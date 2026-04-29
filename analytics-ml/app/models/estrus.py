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

MODEL_FILENAME = "estrus_xgb.pkl"
ONNX_FILENAME = "estrus_xgb.onnx"

FEATURE_COLUMNS = [
    "activity_ratio_7d",
    "rumination_ratio_7d",
    "milk_ratio_7d",
    "dim_days",
    "lactation_number",
    "days_since_last_heat",
    "avg_activity_14d",
    "avg_rumination_14d",
]


def _create_labels(df: pd.DataFrame) -> pd.Series:
    labels = pd.Series(0, index=df.index)
    labels[df["activity_ratio_7d"] > 1.4] = 1
    labels[(df["activity_ratio_7d"] > 1.2) & (df["rumination_ratio_7d"] < 0.85)] = 1
    labels[(df["activity_ratio_7d"] > 1.15) & (df["dim_days"] >= 40) & (df["dim_days"] <= 120)] = 1
    return labels


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    df = df.copy()
    df["label"] = _create_labels(df)

    df_filled, medians = fillna_with_medians(df, FEATURE_COLUMNS)
    X = df_filled[FEATURE_COLUMNS].values

    confirmed_mask = None
    confirmed_labels = None
    if "confirmed_estrus" in df.columns:
        confirmed_mask = df["confirmed_estrus"].notna().values
        confirmed_labels = df["confirmed_estrus"].fillna(0).values

    from app.services.pu_learning import merge_real_labels, pu_adjust_labels
    labels, weights = merge_real_labels(df["label"].values, confirmed_labels, confirmed_mask)

    from app.services.hyperopt import tune_classifier, get_model_instance
    params, backend = tune_classifier(X, labels, n_trials=30, timeout=120)
    backend_str = params.pop("_backend", backend)

    model = get_model_instance(params, backend_str, "classifier")
    cv = min(5, max(2, len(np.unique(labels))))
    cv_scores = cross_val_score(model, X, labels, cv=cv, scoring="roc_auc")

    model.fit(X, labels, sample_weight=weights)

    if settings.mlflow_tracking_uri:
        try:
            import mlflow
            mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
            mlflow.set_experiment("estrus")
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
        "version": "xgboost-v2",
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
        "model_name": "estrus",
        "samples": len(df),
        "metrics": {"cv_auc_mean": float(cv_scores.mean()), "cv_auc_std": float(cv_scores.std())},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame) -> list[dict]:
    onnx_path = os.path.join(settings.model_dir, ONNX_FILENAME)
    if os.path.exists(onnx_path):
        try:
            from app.services.onnx_utils import load_model_onnx, predict_onnx
            session, features, task = load_model_onnx(onnx_path)
            df_filled, _ = fillna_with_medians(df, features)
            X = df_filled[features].values
            probs = predict_onnx(session, X)[:, 1]
            shap_explanations = _compute_shap(None, X, features)
            return _build_results(df, probs, "xgboost-v2", shap_explanations)
        except Exception:
            pass

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    if not os.path.exists(path):
        raise FileNotFoundError(f"Model not found: {path}")

    model_data = joblib.load(path)
    model = model_data["model"]
    features = model_data["features"]
    medians = model_data.get("medians", {})
    version = model_data.get("version", "xgboost-v1")

    df_filled, _ = fillna_with_medians(df, features, medians=medians)
    X = df_filled[features].values
    probs = model.predict_proba(X)[:, 1]

    shap_explanations = _compute_shap(model, X, features)
    return _build_results(df, probs, version, shap_explanations)


def _compute_shap(model, X, features):
    if model is None:
        return []
    try:
        from app.services.shap_explain import explain_prediction
        return explain_prediction(model, X, features)
    except Exception:
        return []


def _build_results(df, probs, version, shap_explanations=None):
    results = []
    for i, row in df.iterrows():
        prob = float(probs[i])
        contributing = []
        if row.get("activity_ratio_7d", 1) > 1.3:
            contributing.append("активность↑")
        if row.get("rumination_ratio_7d", 1) < 0.85:
            contributing.append("жвачка↓")
        if row.get("milk_ratio_7d", 1) < 0.9:
            contributing.append("надой↓")
        dim = row.get("dim_days", 0)
        if 40 <= dim <= 120:
            contributing.append("DIM в окне")

        if prob >= 0.7:
            status = "in_heat"
        elif prob >= 0.4:
            status = "approaching"
        else:
            status = "not_in_heat"

        result = {
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "estrus_probability": round(prob, 4),
            "status": status,
            "contributing_signals": contributing,
            "optimal_window": f"{dim}–{min(dim + 3, 150)} DIM" if 35 <= dim <= 130 else None,
            "model_version": version,
        }

        if shap_explanations and i < len(shap_explanations):
            result["shap_explanation"] = shap_explanations[i]

        results.append(result)

    return results
