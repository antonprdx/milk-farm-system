from __future__ import annotations

import json
import logging
import os
from datetime import datetime, timezone

import numpy as np
import pandas as pd

from app.config import settings

logger = logging.getLogger(__name__)


def _drift_dir() -> str:
    path = os.path.join(settings.model_dir, "drift")
    os.makedirs(path, exist_ok=True)
    return path


def record_predictions(model_name: str, predictions: list[dict]) -> None:
    scores = []
    for p in predictions:
        for key in ("risk_probability", "estrus_probability", "risk_score", "anomaly_score"):
            if key in p:
                scores.append(float(p[key]))
                break

    if not scores:
        return

    path = os.path.join(_drift_dir(), f"{model_name}.jsonl")
    entry = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "n": len(scores),
        "mean": float(np.mean(scores)),
        "std": float(np.std(scores)),
        "min": float(np.min(scores)),
        "max": float(np.max(scores)),
        "p25": float(np.percentile(scores, 25)),
        "p50": float(np.percentile(scores, 50)),
        "p75": float(np.percentile(scores, 75)),
    }
    with open(path, "a") as f:
        f.write(json.dumps(entry) + "\n")


def record_features(model_name: str, features_df: pd.DataFrame) -> None:
    if features_df.empty:
        return

    numeric_cols = features_df.select_dtypes(include=[np.number]).columns.tolist()
    exclude = {"animal_id", "id"}
    cols = [c for c in numeric_cols if c not in exclude]
    if not cols:
        return

    path = os.path.join(_drift_dir(), f"{model_name}_features.jsonl")
    stats = {}
    for col in cols:
        series = features_df[col].dropna()
        if len(series) > 0:
            stats[col] = {
                "mean": float(series.mean()),
                "std": float(series.std()) if len(series) > 1 else 0.0,
                "min": float(series.min()),
                "max": float(series.max()),
                "q25": float(series.quantile(0.25)),
                "q50": float(series.quantile(0.50)),
                "q75": float(series.quantile(0.75)),
            }

    entry = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "n": len(features_df),
        "features": stats,
    }
    with open(path, "a") as f:
        f.write(json.dumps(entry) + "\n")


def check_drift(model_name: str, window: int = 100, threshold: float = 2.0) -> dict:
    path = os.path.join(_drift_dir(), f"{model_name}.jsonl")
    if not os.path.exists(path):
        return {"model": model_name, "status": "no_data", "drift_detected": False}

    entries = []
    with open(path) as f:
        for line in f:
            try:
                entries.append(json.loads(line.strip()))
            except (json.JSONDecodeError, ValueError):
                continue

    if len(entries) < 10:
        return {"model": model_name, "status": "insufficient_data", "drift_detected": False}

    recent = entries[-min(window, len(entries)):]
    baseline = entries[:max(len(entries) // 2, 10)]

    recent_mean = np.mean([e["mean"] for e in recent])
    baseline_mean = np.mean([e["mean"] for e in baseline])
    baseline_std = np.std([e["mean"] for e in baseline]) or 0.001

    z_score = abs(recent_mean - baseline_mean) / baseline_std
    drift = z_score > threshold

    return {
        "model": model_name,
        "status": "drift_detected" if drift else "ok",
        "drift_detected": drift,
        "z_score": round(float(z_score), 4),
        "recent_mean": round(float(recent_mean), 4),
        "baseline_mean": round(float(baseline_mean), 4),
    }


def _ks_test(baseline_vals: list[float], recent_vals: list[float]) -> dict:
    try:
        from scipy.stats import ks_2samp
        stat, p_value = ks_2samp(baseline_vals, recent_vals)
        return {"ks_statistic": round(float(stat), 6), "ks_pvalue": round(float(p_value), 6)}
    except ImportError:
        baseline_arr = np.array(baseline_vals)
        recent_arr = np.array(recent_vals)
        baseline_sorted = np.sort(baseline_arr)
        recent_sorted = np.sort(recent_arr)
        n1 = len(baseline_sorted)
        n2 = len(recent_sorted)
        all_vals = np.sort(np.concatenate([baseline_sorted, recent_sorted]))
        cdf1 = np.searchsorted(baseline_sorted, all_vals, side="right") / n1
        cdf2 = np.searchsorted(recent_sorted, all_vals, side="right") / n2
        ks_stat = float(np.max(np.abs(cdf1 - cdf2)))
        return {"ks_statistic": round(ks_stat, 6), "ks_pvalue": None}
    except Exception as e:
        return {"ks_statistic": None, "ks_pvalue": None, "error": str(e)}


def _mmd(X: np.ndarray, Y: np.ndarray, gamma: float | None = None) -> float:
    from sklearn.metrics.pairwise import rbf_kernel

    if gamma is None:
        median_dist = np.median(np.linalg.norm(X[:100] - Y[:100], axis=1)) if len(X) > 100 and len(Y) > 100 else 1.0
        gamma = 1.0 / (2 * median_dist ** 2 + 1e-8)

    K_XX = rbf_kernel(X, X, gamma=gamma)
    K_YY = rbf_kernel(Y, Y, gamma=gamma)
    K_XY = rbf_kernel(X, Y, gamma=gamma)

    mmd = float(np.mean(K_XX) + np.mean(K_YY) - 2 * np.mean(K_XY))
    return max(mmd, 0.0)


def check_feature_drift(model_name: str, window: int = 50, threshold: float = 2.5, ks_alpha: float = 0.05) -> dict:
    path = os.path.join(_drift_dir(), f"{model_name}_features.jsonl")
    if not os.path.exists(path):
        return {"model": model_name, "status": "no_data", "drift_detected": False, "features": {}}

    entries = []
    with open(path) as f:
        for line in f:
            try:
                entries.append(json.loads(line.strip()))
            except (json.JSONDecodeError, ValueError):
                continue

    if len(entries) < 4:
        return {"model": model_name, "status": "insufficient_data", "drift_detected": False, "features": {}}

    recent = entries[-min(window, len(entries)):]
    baseline = entries[:max(len(entries) // 2, 2)]

    drifted = {}
    all_features = set()
    for e in recent + baseline:
        if "features" in e:
            all_features.update(e["features"].keys())

    for feat in all_features:
        recent_vals = [e["features"].get(feat, {}).get("mean", 0) for e in recent if feat in e.get("features", {})]
        baseline_vals = [e["features"].get(feat, {}).get("mean", 0) for e in baseline if feat in e.get("features", {})]

        if len(recent_vals) < 2 or len(baseline_vals) < 2:
            continue

        r_mean = float(np.mean(recent_vals))
        b_mean = float(np.mean(baseline_vals))
        b_std = float(np.std(baseline_vals)) or 0.001
        z = abs(r_mean - b_mean) / b_std

        ks_result = _ks_test(baseline_vals, recent_vals)

        feature_drifted = z > threshold
        if ks_result.get("ks_pvalue") is not None:
            feature_drifted = feature_drifted or ks_result["ks_pvalue"] < ks_alpha

        if feature_drifted:
            drifted[feat] = {
                "z_score": round(z, 4),
                "ks_statistic": ks_result.get("ks_statistic"),
                "ks_pvalue": ks_result.get("ks_pvalue"),
                "recent_mean": round(r_mean, 4),
                "baseline_mean": round(b_mean, 4),
            }

    return {
        "model": model_name,
        "status": "feature_drift" if drifted else "ok",
        "drift_detected": len(drifted) > 0,
        "drifted_features": drifted,
        "total_features_monitored": len(all_features),
        "samples": len(entries),
    }


def check_mmd_drift(
    model_name: str,
    recent_features: pd.DataFrame,
    baseline_features: pd.DataFrame,
    threshold: float = 0.1,
) -> dict:
    numeric_cols = recent_features.select_dtypes(include=[np.number]).columns.tolist()
    exclude = {"animal_id", "id"}
    cols = [c for c in numeric_cols if c not in exclude]

    if not cols:
        return {"model": model_name, "status": "no_numeric_features", "mmd_drift_detected": False}

    X_recent = recent_features[cols].fillna(0).values.astype(np.float64)
    X_baseline = baseline_features[cols].fillna(0).values.astype(np.float64)

    if len(X_recent) < 2 or len(X_baseline) < 2:
        return {"model": model_name, "status": "insufficient_data", "mmd_drift_detected": False}

    mmd_value = _mmd(X_baseline[:min(500, len(X_baseline))], X_recent[:min(500, len(X_recent))])
    drift_detected = mmd_value > threshold

    return {
        "model": model_name,
        "status": "mmd_drift" if drift_detected else "ok",
        "mmd_drift_detected": drift_detected,
        "mmd_value": round(mmd_value, 6),
        "threshold": threshold,
        "n_features": len(cols),
        "n_recent": len(X_recent),
        "n_baseline": len(X_baseline),
    }


def comprehensive_drift_check(model_name: str, recent_features: pd.DataFrame | None = None) -> dict:
    prediction_drift = check_drift(model_name)
    feature_drift = check_feature_drift(model_name)

    result = {
        "model": model_name,
        "prediction_drift": prediction_drift,
        "feature_drift": feature_drift,
        "overall_drift_detected": prediction_drift.get("drift_detected", False) or feature_drift.get("drift_detected", False),
    }

    if recent_features is not None:
        baseline_path = os.path.join(_drift_dir(), f"{model_name}_baseline_features.pkl")
        try:
            if os.path.exists(baseline_path):
                import joblib
                baseline_features = joblib.load(baseline_path)
                mmd_result = check_mmd_drift(model_name, recent_features, baseline_features)
                result["mmd_drift"] = mmd_result
                result["overall_drift_detected"] = result["overall_drift_detected"] or mmd_result.get("mmd_drift_detected", False)
            else:
                import joblib
                joblib.dump(recent_features, baseline_path)
                result["mmd_drift"] = {"status": "baseline_saved", "mmd_drift_detected": False}
        except Exception as e:
            result["mmd_drift"] = {"status": "error", "error": str(e), "mmd_drift_detected": False}

    return result
