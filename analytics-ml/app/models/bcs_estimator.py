from __future__ import annotations

import logging
import os
import time

import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score

from app.config import settings
from app.services.fillna import fillna_with_medians
from app.services.model_versioning import save_with_version

logger = logging.getLogger(__name__)

MODEL_FILENAME = "bcs_estimator.pkl"

FEATURE_COLUMNS = [
    "dim_days",
    "lactation_number",
    "avg_milk_7d",
    "avg_milk_30d",
    "avg_feed_7d",
    "avg_rumination_7d",
    "avg_activity_7d",
    "milk_feed_ratio",
    "milk_trend_7d",
    "weight_kg",
]


def _estimate_bcs(row) -> float:
    milk = float(row.get("avg_milk_7d", 0) or 0)
    feed = float(row.get("avg_feed_7d", 0) or 0)
    rum = float(row.get("avg_rumination_7d", 0) or 0)
    dim = float(row.get("dim_days", 0) or 0)
    act = float(row.get("avg_activity_7d", 0) or 0)
    weight = float(row.get("weight_kg", 0) or 0)

    bcs = 3.0

    if weight > 0:
        bcs += (weight - 550) / 200.0 * 0.5
    else:
        if milk > 35:
            bcs -= 0.3
        elif milk > 25:
            bcs -= 0.1
        elif milk < 15:
            bcs += 0.2

    if dim < 60:
        bcs -= 0.4
    elif dim < 100:
        bcs -= 0.2
    elif dim > 250:
        bcs += 0.3

    if rum < 400:
        bcs -= 0.2
    elif rum > 550:
        bcs += 0.1

    if feed > 0 and milk > 0:
        ratio = milk / feed
        if ratio > 1.8:
            bcs -= 0.2
        elif ratio < 1.0:
            bcs += 0.2

    return max(1.0, min(5.0, bcs))


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    df_filled, medians = fillna_with_medians(df, FEATURE_COLUMNS)
    X = df_filled[FEATURE_COLUMNS].values

    if "bcs_manual" in df.columns and df["bcs_manual"].notna().sum() >= 10:
        y = df["bcs_manual"].fillna(3.0).values
        label_source = "manual"
    elif "bcs" in df.columns and df["bcs"].notna().sum() >= 10:
        y = df["bcs"].fillna(3.0).values
        label_source = "existing_bcs"
    else:
        y = df.apply(_estimate_bcs, axis=1).values
        label_source = "rule_based"

    from app.services.hyperopt import tune_regressor, get_model_instance
    params, backend = tune_regressor(X, y, n_trials=20, timeout=60)
    backend_str = params.pop("_backend", backend)

    model = get_model_instance(params, backend_str, "regressor")
    model.fit(X, y)

    if settings.mlflow_tracking_uri:
        try:
            import mlflow
            mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
            mlflow.set_experiment("bcs_estimator")
            with mlflow.start_run():
                mlflow.log_params({**params, "backend": backend_str, "label_source": label_source})
                mlflow.log_metrics({
                    "samples": len(df),
                    "duration_seconds": round(time.time() - start, 2),
                })
        except Exception as e:
            logger.warning("MLflow logging failed: %s", e)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    save_with_version(path, {
        "model": model,
        "features": FEATURE_COLUMNS,
        "medians": medians,
        "backend": backend_str,
        "params": params,
        "version": "bcs-v2",
    })

    duration = time.time() - start
    return {
        "model_name": "bcs_estimator",
        "samples": len(df),
        "metrics": {"method": "ml", "backend": backend_str, "label_source": label_source},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame, include_shap: bool = False) -> list[dict]:
    path = os.path.join(settings.model_dir, MODEL_FILENAME)

    from app.services.model_cache import get_model
    model_data = get_model("bcs_estimator", path)

    if model_data is not None:
        model = model_data["model"]
        features = model_data["features"]
        medians = model_data.get("medians", {})
        version = model_data.get("version", "bcs-v1")

        df_filled, _ = fillna_with_medians(df, features, medians=medians)
        X = df_filled[features].values
        preds = model.predict(X)

        if include_shap:
            try:
                from app.services.shap_explain import explain_prediction
                shap_explanations = explain_prediction(model, X, features)
            except Exception:
                shap_explanations = []
        else:
            shap_explanations = []
    else:
        features = FEATURE_COLUMNS
        version = "rule-v1"
        preds = df.apply(_estimate_bcs, axis=1).values
        shap_explanations = []

    animal_ids = df["animal_id"].values
    names = df["animal_name"].values if "animal_name" in df.columns else [""] * len(df)

    results = []
    for i in range(len(df)):
        bcs_est = round(max(1.0, min(5.0, float(preds[i]))), 2)

        if bcs_est < 2.5:
            status = "underconditioned"
        elif bcs_est > 3.75:
            status = "overconditioned"
        else:
            status = "optimal"

        result = {
            "animal_id": int(animal_ids[i]),
            "animal_name": names[i],
            "estimated_bcs": bcs_est,
            "status": status,
            "model_version": version,
        }

        if shap_explanations and i < len(shap_explanations):
            result["shap_explanation"] = shap_explanations[i]

        results.append(result)

    return results
