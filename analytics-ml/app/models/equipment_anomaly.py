from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.ensemble import IsolationForest
from sklearn.isotonic import IsotonicRegression
from sklearn.preprocessing import StandardScaler

from app.config import settings
from app.services.model_versioning import save_with_version

MODEL_FILENAME = "equipment_anomaly.pkl"
ONNX_FILENAME = "equipment_anomaly.onnx"

FEATURE_COLUMNS = [
    "avg_conductivity",
    "max_quarter_asymmetry",
    "avg_milk_temperature",
    "std_milk_temperature",
    "avg_milk_yield_per_visit",
    "avg_milk_speed",
    "anomaly_rate_7d",
]


def _estimate_contamination(scores: np.ndarray) -> float:
    q1 = np.percentile(scores, 25)
    q3 = np.percentile(scores, 75)
    iqr = q3 - q1
    lower_bound = q1 - 1.5 * iqr
    anomaly_ratio = float(np.mean(scores < lower_bound))
    return max(min(anomaly_ratio, 0.2), 0.01)


def _calibrate_scores(scores: np.ndarray, labels: np.ndarray | None = None) -> IsotonicRegression | None:
    if labels is None:
        return None

    if len(np.unique(labels)) < 2:
        return None

    ir = IsotonicRegression(out_of_bounds="clip")
    ir.fit(scores, labels.astype(float))
    return ir


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    X = df[FEATURE_COLUMNS].fillna(0).values

    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X)

    preliminary_model = IsolationForest(
        n_estimators=100,
        contamination=0.1,
        random_state=42,
    )
    preliminary_model.fit(X_scaled)
    prelim_scores = preliminary_model.decision_function(X_scaled)

    contamination = _estimate_contamination(prelim_scores)

    model = IsolationForest(
        n_estimators=200,
        contamination=contamination,
        random_state=42,
    )
    model.fit(X_scaled)

    scores = model.decision_function(X_scaled)
    preds = model.predict(X_scaled)

    confirmed_labels = None
    if "confirmed_anomaly" in df.columns:
        confirmed_labels = df["confirmed_anomaly"].fillna(0).values

    calibrator = _calibrate_scores(scores, confirmed_labels)

    n_anomalies = int((preds == -1).sum())

    if settings.mlflow_tracking_uri:
        try:
            import mlflow
            mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
            mlflow.set_experiment("equipment_anomaly")
            with mlflow.start_run():
                mlflow.log_params({
                    "n_estimators": 200,
                    "contamination": round(contamination, 4),
                    "calibrated": calibrator is not None,
                })
                mlflow.log_metrics({
                    "anomalies_detected": n_anomalies,
                    "anomaly_pct": round(n_anomalies / len(df) * 100, 2),
                    "samples": len(df),
                    "duration_seconds": round(time.time() - start, 2),
                })
        except Exception as e:
            import logging
            logging.getLogger(__name__).warning("MLflow logging failed: %s", e)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    save_with_version(path, {
        "model": model,
        "scaler": scaler,
        "calibrator": calibrator,
        "features": FEATURE_COLUMNS,
        "contamination": contamination,
        "version": "isolation-forest-v2",
    })

    duration = time.time() - start
    return {
        "model_name": "equipment_anomaly",
        "samples": len(df),
        "metrics": {
            "anomalies_detected": n_anomalies,
            "anomaly_pct": round(n_anomalies / len(df) * 100, 2),
            "contamination": round(contamination, 4),
            "calibrated": calibrator is not None,
        },
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame) -> list[dict]:
    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    if not os.path.exists(path):
        raise FileNotFoundError(f"Model not found: {path}")

    model_data = joblib.load(path)
    model = model_data["model"]
    scaler = model_data["scaler"]
    calibrator = model_data.get("calibrator")
    features = model_data["features"]
    version = model_data.get("version", "isolation-forest-v1")

    X = df[features].fillna(0).values
    X_scaled = scaler.transform(X)

    preds = model.predict(X_scaled)
    scores = model.decision_function(X_scaled)

    if calibrator is not None:
        calibrated_probs = calibrator.transform(scores)
    else:
        score_min = scores.min()
        score_max = scores.max()
        score_range = score_max - score_min or 1.0
        calibrated_probs = 1.0 - (scores - score_min) / score_range

    results = []
    for i, row in df.iterrows():
        is_anomaly = preds[i] == -1
        score = float(scores[i])
        prob = float(calibrated_probs[i])

        flags = []
        if row.get("avg_conductivity", 0) > 75:
            flags.append("проводимость↑")
        if row.get("max_quarter_asymmetry", 0) > 10:
            flags.append("асимметрия↑")
        if row.get("avg_milk_temperature", 37) > 38.5:
            flags.append("температура↑")
        if row.get("avg_milk_speed", 2) < 1.0:
            flags.append("скорость↓")

        severity = "critical" if prob >= 0.8 else "warning" if prob >= 0.5 else "normal"

        results.append({
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "is_anomaly": is_anomaly,
            "anomaly_score": round(score, 4),
            "anomaly_probability": round(prob, 4),
            "severity": severity if is_anomaly else "normal",
            "flags": flags,
            "device_address": int(row.get("device_address", 0)) or None,
            "model_version": version,
        })

    return results
