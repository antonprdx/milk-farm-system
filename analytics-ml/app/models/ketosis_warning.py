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

MODEL_FILENAME = "ketosis_warning_xgb.pkl"
ONNX_FILENAME = "ketosis_warning_xgb.onnx"

FEATURE_COLUMNS = [
    "fpr_7d",
    "fpr_14d",
    "fpr_trend",
    "avg_rumination_7d",
    "avg_rumination_14d",
    "rumination_trend",
    "avg_milk_7d",
    "milk_trend",
    "dim_days",
    "lactation_number",
]


def _create_labels(df: pd.DataFrame) -> pd.Series:
    labels = pd.Series(0, index=df.index)
    labels[df["fpr_7d"] > 1.5] = 1
    labels[df["fpr_7d"] < 1.0] = 1
    labels[(df["fpr_7d"] > 1.4) & (df["rumination_trend"] < -0.1)] = 1
    labels[(df["fpr_7d"] < 1.1) & (df["dim_days"] < 60)] = 1
    return labels


def _risk_type(fpr: float) -> str:
    if fpr < 1.0:
        return "ketosis_risk"
    if fpr > 1.5:
        return "acidosis_risk"
    if fpr > 1.4:
        return "acidosis_warning"
    if fpr < 1.1:
        return "ketosis_warning"
    return "normal"


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    df = df.copy()
    df["label"] = _create_labels(df)

    df_filled, medians = fillna_with_medians(df, FEATURE_COLUMNS)
    X = df_filled[FEATURE_COLUMNS].values

    confirmed_mask = None
    confirmed_labels = None
    if "confirmed_ketosis" in df.columns:
        confirmed_mask = df["confirmed_ketosis"].notna().values
        confirmed_labels = df["confirmed_ketosis"].fillna(0).values

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
            mlflow.set_experiment("ketosis_warning")
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
        "model_name": "ketosis_warning",
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
        fpr = float(row.get("fpr_7d", 1.3) or 1.3)
        rtype = _risk_type(fpr)

        contributing = []
        if fpr > 1.4:
            contributing.append("FPR↑")
        if fpr < 1.1:
            contributing.append("FPR↓")
        if row.get("rumination_trend", 0) < -0.1:
            contributing.append("жвачка↓")
        if row.get("milk_trend", 0) < -0.1:
            contributing.append("надой↓")
        dim = row.get("dim_days", 0)
        if dim and dim < 60:
            contributing.append("ранняя лактация")

        severity = "high" if prob >= 0.7 else "medium" if prob >= 0.4 else "low"

        result = {
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "risk_probability": round(prob, 4),
            "risk_type": rtype,
            "severity": severity,
            "fpr_current": round(fpr, 3),
            "fpr_trend": round(float(row.get("fpr_trend", 0) or 0), 4),
            "contributing_factors": contributing,
            "model_version": version,
        }

        if shap_explanations and i < len(shap_explanations):
            result["shap_explanation"] = shap_explanations[i]

        results.append(result)

    return results
