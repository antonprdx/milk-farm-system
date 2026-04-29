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

MODEL_FILENAME = "culling_xgb.pkl"
ONNX_FILENAME = "culling_xgb.onnx"

FEATURE_COLUMNS = [
    "age_years",
    "avg_milk_30d",
    "avg_scc_90d",
    "calving_interval",
    "lactation_count",
    "avg_rumination_30d",
    "avg_milk_7d",
    "avg_activity_30d",
    "current_dim",
]

BASE_DAYS = 730.0


def _create_target(df: pd.DataFrame) -> pd.Series:
    risk = pd.Series(0.0, index=df.index)
    risk[df["age_years"] >= 10] += 0.4
    risk[(df["age_years"] >= 8) & (df["age_years"] < 10)] += 0.25
    risk[(df["age_years"] >= 6) & (df["age_years"] < 8)] += 0.1
    risk[df["avg_milk_30d"] < 15] += 0.3
    risk[(df["avg_milk_30d"] >= 15) & (df["avg_milk_30d"] < 20)] += 0.1
    risk[df["avg_scc_90d"] > 300000] += 0.25
    risk[(df["avg_scc_90d"] > 200000) & (df["avg_scc_90d"] <= 300000)] += 0.1
    risk[df["calving_interval"] > 450] += 0.2
    risk[(df["calving_interval"] > 400) & (df["calving_interval"] <= 450)] += 0.1
    risk[df["lactation_count"] >= 6] += 0.1
    risk = risk.clip(upper=1.0)

    expected_days = BASE_DAYS * (1 - risk)
    return expected_days.clip(lower=0)


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    df_filled, medians = fillna_with_medians(df, FEATURE_COLUMNS)
    X = df_filled[FEATURE_COLUMNS].values
    y = _create_target(df).values

    from app.services.hyperopt import tune_regressor, get_model_instance
    params, backend = tune_regressor(X, y, n_trials=30, timeout=120)
    backend_str = params.pop("_backend", backend)

    model = get_model_instance(params, backend_str, "regressor")
    cv_scores = cross_val_score(model, X, y, cv=min(5, max(2, len(df) // 10)), scoring="r2")
    model.fit(X, y)

    if settings.mlflow_tracking_uri:
        try:
            import mlflow
            mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
            mlflow.set_experiment("culling")
            with mlflow.start_run():
                mlflow.log_params({**params, "backend": backend_str})
                mlflow.log_metrics({
                    "cv_r2_mean": float(cv_scores.mean()),
                    "cv_r2_std": float(cv_scores.std()),
                    "samples": len(df),
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
        save_model_onnx(model, FEATURE_COLUMNS, onnx_path, task="regress")
    except Exception as e:
        import logging
        logging.getLogger(__name__).warning("ONNX export failed: %s", e)

    duration = time.time() - start
    return {
        "model_name": "culling",
        "samples": len(df),
        "metrics": {"cv_r2_mean": float(cv_scores.mean()), "cv_r2_std": float(cv_scores.std())},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame, model_data: dict | None = None) -> list[dict]:
    onnx_path = os.path.join(settings.model_dir, ONNX_FILENAME)
    if model_data is None and os.path.exists(onnx_path):
        try:
            from app.services.onnx_utils import load_model_onnx, predict_onnx
            session, features, task = load_model_onnx(onnx_path)
            df_filled, _ = fillna_with_medians(df, features)
            X = df_filled[features].values
            predicted_days = predict_onnx(session, X).flatten()
            shap_explanations = _compute_shap(None, X, features)
            return _build_results(df, predicted_days, "xgboost-v2", shap_explanations)
        except Exception:
            pass

    if model_data is None:
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
    predicted_days = model.predict(X)

    shap_explanations = _compute_shap(model, X, features)
    return _build_results(df, predicted_days, version, shap_explanations)


def _compute_shap(model, X, features):
    if model is None:
        return []
    try:
        from app.services.shap_explain import explain_prediction
        return explain_prediction(model, X, features)
    except Exception:
        return []


def _build_results(df, predicted_days, version, shap_explanations=None):
    results = []
    for i, row in df.iterrows():
        expected_days = float(predicted_days[i])
        risk_prob = 1.0 - min(expected_days / BASE_DAYS, 1.0)

        risk_factors = []
        if row.get("age_years", 0) >= 8:
            risk_factors.append(f"age>={int(row['age_years'])}yr")
        if row.get("avg_milk_30d", 0) < 20:
            risk_factors.append("milk<20L")
        if row.get("avg_scc_90d", 0) > 200000:
            risk_factors.append("SCC>200k")
        if row.get("calving_interval", 0) > 400:
            risk_factors.append("interval>400d")
        if row.get("lactation_count", 0) >= 6:
            risk_factors.append("lac>=6")

        risk_level = "high" if risk_prob >= 0.6 else "medium" if risk_prob >= 0.3 else "low"

        result = {
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "risk_probability": round(risk_prob, 4),
            "expected_days_remaining": max(int(expected_days), 0),
            "risk_factors": risk_factors,
            "model_version": version,
        }

        if shap_explanations and i < len(shap_explanations):
            result["shap_explanation"] = shap_explanations[i]

        results.append(result)

    return results
