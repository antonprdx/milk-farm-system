from __future__ import annotations

import os
import time

import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score

from app.config import settings
from app.services.fillna import fillna_with_medians
from app.services.model_versioning import save_with_version

MODEL_FILENAME = "feed_recommendation_xgb.pkl"
ONNX_FILENAME = "feed_recommendation_xgb.onnx"

FEATURE_COLUMNS = [
    "dim_days",
    "lactation_number",
    "avg_milk_7d",
    "avg_feed_7d",
    "avg_rumination_7d",
    "avg_activity_7d",
    "milk_feed_ratio",
    "avg_scc_30d",
    "milk_trend_7d",
]


def _compute_nrc_target(row) -> float:
    dim = float(row.get("dim_days", 0) or 0)
    lac = int(row.get("lactation_number", 0) or 0)
    milk = float(row.get("avg_milk_7d", 0) or 0)
    rumination = float(row.get("avg_rumination_7d", 0) or 0)
    bw = 550.0 + lac * 25.0

    maintenance_ne = 0.08 * bw ** 0.75
    milk_ne = milk * 0.74

    dim_factor = 1.0
    if dim < 21:
        dim_factor = 1.2 - (dim / 21) * 0.25
    elif dim < 60:
        dim_factor = 1.05
    elif dim > 250:
        dim_factor = 0.92

    lactation_factor = 1.0 + max(lac - 1, 0) * 0.03

    total_ne = (maintenance_ne + milk_ne) * dim_factor * lactation_factor

    feed_dm = total_ne / 1.5

    if rumination > 0 and rumination < 400:
        feed_dm *= 0.95
    elif rumination > 550:
        feed_dm *= 1.02

    feed_as_fed = feed_dm * 0.55

    return round(max(feed_as_fed, 0), 2)


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    if "recommended_feed" in df.columns and df["recommended_feed"].notna().sum() >= 10:
        y = df["recommended_feed"].values
    else:
        y = df.apply(_compute_nrc_target, axis=1).values

    df_filled, medians = fillna_with_medians(df, FEATURE_COLUMNS)
    X = df_filled[FEATURE_COLUMNS].values

    from app.services.hyperopt import tune_regressor
    params, backend = tune_regressor(X, y, n_trials=30, timeout=120)
    backend_str = params.pop("_backend", backend)

    from app.services.hyperopt import get_model_instance
    model = get_model_instance(params, backend_str, "regressor")

    cv_scores = cross_val_score(
        model, X, y, cv=min(5, max(2, len(df) // 10)), scoring="neg_mean_absolute_error",
    )
    model.fit(X, y)

    if settings.mlflow_tracking_uri:
        try:
            import mlflow
            mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
            mlflow.set_experiment("feed_recommendation")
            with mlflow.start_run():
                mlflow.log_params({**params, "backend": backend_str})
                mlflow.log_metrics({
                    "cv_mae_mean": float(-cv_scores.mean()),
                    "cv_mae_std": float(cv_scores.std()),
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
        "version": "ml-v1",
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
        "model_name": "feed_recommendation",
        "samples": len(df),
        "metrics": {"cv_mae_mean": float(-cv_scores.mean()), "cv_mae_std": float(cv_scores.std()), "backend": backend_str},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame, include_shap: bool = False) -> list[dict]:
    path = os.path.join(settings.model_dir, MODEL_FILENAME)

    from app.services.model_cache import get_model
    model_data = get_model("feed_recommendation", path)

    if model_data is not None:
        model = model_data["model"]
        features = model_data["features"]
        medians = model_data.get("medians", {})
        version = model_data.get("version", "formula-v1")

        df_filled, _ = fillna_with_medians(df, features, medians=medians)
        X = df_filled[features].values
        preds = model.predict(X)
    else:
        features = FEATURE_COLUMNS
        version = "formula-v1"
        preds = df.apply(_compute_nrc_target, axis=1).values

    animal_ids = df["animal_id"].values
    names = df["animal_name"].values if "animal_name" in df.columns else [""] * len(df)
    avg_feed_7d = df["avg_feed_7d"].values if "avg_feed_7d" in df.columns else np.full(len(df), 0.0)
    dim_values = df["dim_days"].values if "dim_days" in df.columns else np.full(len(df), 0)
    lac_values = df["lactation_number"].values if "lactation_number" in df.columns else np.full(len(df), 0)

    results = []
    for i in range(len(df)):
        recommended = round(max(float(preds[i]), 0), 2)
        current = float(avg_feed_7d[i]) if avg_feed_7d[i] == avg_feed_7d[i] else 0.0
        diff = round(recommended - current, 2)

        results.append({
            "animal_id": int(animal_ids[i]),
            "animal_name": names[i],
            "current_feed_avg": round(current, 2),
            "recommended_feed": recommended,
            "difference_kg": diff,
            "suggestion": "increase" if diff > 1.0 else "decrease" if diff < -1.0 else "maintain",
            "dim_days": int(dim_values[i]),
            "lactation_number": int(lac_values[i]),
            "model_version": version,
        })

    return results
