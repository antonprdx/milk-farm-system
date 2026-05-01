from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score

from app.config import settings
from app.services.fillna import fillna_with_medians
from app.services.model_versioning import save_with_version

MODEL_FILENAME = "mastitis_xgb.pkl"
ONNX_FILENAME = "mastitis_xgb.onnx"

FEATURE_COLUMNS = [
    "age_years",
    "recent_scc",
    "scc_trend_ratio",
    "avg_conductivity",
    "milk_deviation",
    "dim_days",
    "avg_rumination_7d",
    "avg_activity_7d",
    "fat_protein_ratio",
    "cond_asymmetry",
    "avg_lactose_7d",
    "lactose_trend",
    "weather_temp",
    "weather_humidity",
    "thi",
    "mastitis_treatments_90d",
    "days_since_mastitis_tx",
    "vet_tx_count_180d",
    "days_since_any_tx",
    "holstein_percentage",
]


def _create_labels(df: pd.DataFrame) -> pd.Series:
    labels = pd.Series(0, index=df.index)
    labels[df["recent_scc"] > 300000] = 1
    labels[(df["recent_scc"] > 200000) & (df["scc_trend_ratio"] > 1.5)] = 1
    labels[(df["recent_scc"] > 150000) & (df["avg_conductivity"] > 55)] = 1
    labels[(df["milk_deviation"] < -0.15) & (df["recent_scc"] > 100000)] = 1
    labels[(df["cond_asymmetry"] > 5) & (df["recent_scc"] > 100000)] = 1
    labels[(df["fat_protein_ratio"] > 0) & (df["fat_protein_ratio"] < 1.0) & (df["recent_scc"] > 100000)] = 1
    if "avg_lactose_7d" in df.columns:
        labels[(df["avg_lactose_7d"] > 0) & (df["avg_lactose_7d"] < 4.2) & (df["recent_scc"] > 100000)] = 1
    if "lactose_trend" in df.columns:
        labels[(df["lactose_trend"] < -0.03) & (df["recent_scc"] > 100000)] = 1
    if "mastitis_treatments_90d" in df.columns:
        labels[df["mastitis_treatments_90d"] > 0] = 1
    return labels


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    df = df.copy()
    df["label"] = _create_labels(df)

    df_filled, medians = fillna_with_medians(df, FEATURE_COLUMNS)
    X = df_filled[FEATURE_COLUMNS].values

    confirmed_mask = None
    confirmed_labels = None
    if "confirmed_mastitis" in df.columns:
        confirmed_mask = df["confirmed_mastitis"].notna().values
        confirmed_labels = df["confirmed_mastitis"].fillna(0).values

    from app.services.pu_learning import merge_real_labels, pu_adjust_labels
    labels, weights = merge_real_labels(df["label"].values, confirmed_labels, confirmed_mask)

    from app.services.hyperopt import tune_classifier, get_model_instance
    params, backend = tune_classifier(X, labels, n_trials=30, timeout=120)
    backend_str = params.pop("_backend", backend)

    model = get_model_instance(params, backend_str, "classifier")
    cv_scores = cross_val_score(model, X, labels, cv=min(5, max(2, len(np.unique(labels)))), scoring="roc_auc")

    model.fit(X, labels, sample_weight=weights)

    if settings.mlflow_tracking_uri:
        try:
            import mlflow
            mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
            mlflow.set_experiment("mastitis")
            with mlflow.start_run():
                mlflow.log_params({**params, "backend": backend_str})
                mlflow.log_metrics({
                    "cv_auc_mean": float(cv_scores.mean()),
                    "cv_auc_std": float(cv_scores.std()),
                    "samples": len(df),
                    "confirmed_samples": int(confirmed_mask.sum()) if confirmed_mask is not None else 0,
                    "duration_seconds": round(time.time() - start, 2),
                })
        except Exception as e:
            import logging
            logging.getLogger(__name__).warning("MLflow logging failed: %s", e)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    save_with_version(path, {
        "model": model,
        "features": FEATURE_COLUMNS,
        "medians": medians,
        "backend": backend_str,
        "params": params,
        "version": "xgboost-v3",
    })

    try:
        from app.services.onnx_utils import save_model_onnx
        onnx_path = os.path.join(settings.model_dir, ONNX_FILENAME)
        save_model_onnx(model, FEATURE_COLUMNS, onnx_path, task="classify")
    except Exception as e:
        import logging
        logging.getLogger(__name__).warning("ONNX export failed: %s", e)

    duration = time.time() - start
    return {
        "model_name": "mastitis",
        "samples": len(df),
        "metrics": {"cv_auc_mean": float(cv_scores.mean()), "cv_auc_std": float(cv_scores.std())},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame, model_data: dict | None = None, include_shap: bool = False) -> list[dict]:
    features = FEATURE_COLUMNS
    version = "xgboost-v3"
    medians = {}

    onnx_path = os.path.join(settings.model_dir, ONNX_FILENAME)
    if model_data is None and os.path.exists(onnx_path):
        try:
            from app.services.model_cache import get_onnx_session
            result = get_onnx_session(onnx_path)
            if result is not None:
                session, features, task = result
                from app.services.onnx_utils import predict_onnx
                df_filled, _ = fillna_with_medians(df, features)
                X = df_filled[features].values
                probs = predict_onnx(session, X)[:, 1]
                shap_explanations = _compute_shap(None, X, features) if include_shap else []
                return _build_results(df, probs, features, version, shap_explanations)
        except Exception:
            pass

    if model_data is None:
        path = os.path.join(settings.model_dir, MODEL_FILENAME)
        from app.services.model_cache import get_model
        model_data = get_model("mastitis", path)
        if model_data is None:
            raise FileNotFoundError(f"Model not found: {path}")

    model = model_data["model"]
    features = model_data["features"]
    medians = model_data.get("medians", {})
    version = model_data.get("version", "xgboost-v2")

    df_filled, _ = fillna_with_medians(df, features, medians=medians)
    X = df_filled[features].values
    probs = model.predict_proba(X)[:, 1]

    shap_explanations = _compute_shap(model, X, features) if include_shap else []
    return _build_results(df, probs, features, version, shap_explanations)


def _compute_shap(model, X, features):
    if model is None:
        return []
    try:
        from app.services.shap_explain import explain_prediction
        return explain_prediction(model, X, features)
    except Exception:
        return []


def _build_results(df, probs, features, version, shap_explanations=None):
    animal_ids = df["animal_id"].values
    names = df["animal_name"].values if "animal_name" in df.columns else [""] * len(df)
    recent_scc = df["recent_scc"].values if "recent_scc" in df.columns else np.zeros(len(df))
    scc_trend = df["scc_trend_ratio"].values if "scc_trend_ratio" in df.columns else np.ones(len(df))
    conductivity = df["avg_conductivity"].values if "avg_conductivity" in df.columns else np.zeros(len(df))
    milk_dev = df["milk_deviation"].values if "milk_deviation" in df.columns else np.zeros(len(df))
    dim_days = df["dim_days"].values if "dim_days" in df.columns else np.zeros(len(df))
    cond_asym = df["cond_asymmetry"].values if "cond_asymmetry" in df.columns else np.zeros(len(df))
    fpr = df["fat_protein_ratio"].values if "fat_protein_ratio" in df.columns else np.full(len(df), 1.5)

    results = []
    for i in range(len(df)):
        prob = float(probs[i])
        contributing = []
        if recent_scc[i] > 300000:
            contributing.append("SCC>300k")
        if scc_trend[i] > 2:
            contributing.append("SCC↑↑")
        elif scc_trend[i] > 1.5:
            contributing.append("SCC↑")
        if conductivity[i] > 60:
            contributing.append("conductivity↑")
        if milk_dev[i] < -0.15:
            contributing.append("milk↓")
        if dim_days[i] < 30:
            contributing.append("early_lactation")
        if cond_asym[i] > 5:
            contributing.append("quarter_asymmetry↑")
        if 0 < fpr[i] < 1.0:
            contributing.append("FPR↓(ketosis_risk)")

        risk_level = "high" if prob >= 0.6 else "medium" if prob >= 0.3 else "low"

        result = {
            "animal_id": int(animal_ids[i]),
            "animal_name": str(names[i]) if names[i] else None,
            "risk_probability": round(prob, 4),
            "risk_level": risk_level,
            "contributing_features": contributing,
            "model_version": version,
        }

        if shap_explanations and i < len(shap_explanations):
            result["shap_explanation"] = shap_explanations[i]

        results.append(result)

    return results
