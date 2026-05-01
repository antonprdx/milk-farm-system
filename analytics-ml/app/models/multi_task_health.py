from __future__ import annotations

import os
import time

import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score
from sklearn.multioutput import MultiOutputClassifier

from app.config import settings
from app.services.fillna import fillna_with_medians
from app.services.model_versioning import save_with_version

MODEL_FILENAME = "multi_task_health.pkl"

FEATURE_COLUMNS = [
    "age_years",
    "dim_days",
    "lactation_number",
    "recent_scc",
    "scc_trend_ratio",
    "avg_conductivity",
    "milk_deviation",
    "avg_rumination_7d",
    "avg_activity_7d",
    "fat_protein_ratio",
    "cond_asymmetry",
    "avg_lactose_7d",
    "lactose_trend",
    "weather_temp",
    "thi",
    "vet_tx_count_180d",
    "days_since_any_tx",
    "holstein_percentage",
    "activity_ratio_7d",
    "rumination_ratio_7d",
    "milk_ratio_7d",
    "fpr_7d",
    "fpr_trend",
]


def _create_labels(df: pd.DataFrame) -> pd.DataFrame:
    mastitis = pd.Series(0, index=df.index)
    mastitis[df["recent_scc"] > 300000] = 1
    mastitis[(df["recent_scc"] > 200000) & (df["scc_trend_ratio"] > 1.5)] = 1
    mastitis[(df["recent_scc"] > 150000) & (df["avg_conductivity"] > 55)] = 1
    if "avg_lactose_7d" in df.columns:
        mastitis[(df["avg_lactose_7d"] > 0) & (df["avg_lactose_7d"] < 4.2) & (df["recent_scc"] > 100000)] = 1

    ketosis = pd.Series(0, index=df.index)
    if "fpr_7d" in df.columns:
        ketosis[df["fpr_7d"] > 1.5] = 1
        ketosis[(df["fpr_7d"] < 1.0) & (df["dim_days"] < 60)] = 1
    if "fpr_trend" in df.columns:
        ketosis[(df["fpr_trend"] < -0.1) & (df["dim_days"] < 60)] = 1

    estrus = pd.Series(0, index=df.index)
    if "activity_ratio_7d" in df.columns:
        estrus[df["activity_ratio_7d"] > 1.4] = 1
        estrus[(df["activity_ratio_7d"] > 1.2) & (df["rumination_ratio_7d"] < 0.85)] = 1
    estrus[(df["dim_days"] > 30) & (df["dim_days"] < 150)] = estrus[(df["dim_days"] > 30) & (df["dim_days"] < 150)]

    return pd.DataFrame({"mastitis": mastitis, "ketosis": ketosis, "estrus": estrus})


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    available_features = [f for f in FEATURE_COLUMNS if f in df.columns]
    df_filled, medians = fillna_with_medians(df, available_features)
    X = df_filled[available_features].values
    Y = _create_labels(df)

    from app.services.hyperopt import tune_classifier, get_model_instance
    params, backend = tune_classifier(X, Y.iloc[:, 0].values, n_trials=30, timeout=120)
    backend_str = params.pop("_backend", backend)

    base_model = get_model_instance(params, backend_str, "classifier")
    model = MultiOutputClassifier(base_model)

    model.fit(X, Y.values)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    save_with_version(path, {
        "model": model,
        "features": available_features,
        "medians": medians,
        "backend": backend_str,
        "params": params,
        "version": "multitask-v1",
        "targets": ["mastitis", "ketosis", "estrus"],
    })

    duration = time.time() - start
    return {
        "model_name": "multi_task_health",
        "samples": len(df),
        "metrics": {"targets": ["mastitis", "ketosis", "estrus"]},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame, model_data: dict | None = None, include_shap: bool = False) -> list[dict]:
    if model_data is None:
        path = os.path.join(settings.model_dir, MODEL_FILENAME)
        from app.services.model_cache import get_model
        model_data = get_model("multi_task_health", path)
        if model_data is None:
            raise FileNotFoundError(f"Model not found: {path}")

    model = model_data["model"]
    features = model_data["features"]
    medians = model_data.get("medians", {})
    targets = model_data.get("targets", ["mastitis", "ketosis", "estrus"])

    available = [f for f in features if f in df.columns]
    df_filled, _ = fillna_with_medians(df, available, medians=medians)
    X = df_filled[available].values

    probs = model.predict_proba(X)

    animal_ids = df["animal_id"].values
    names = df["animal_name"].values if "animal_name" in df.columns else [""] * len(df)

    results = []
    for i in range(len(df)):
        entry = {
            "animal_id": int(animal_ids[i]),
            "animal_name": names[i],
        }
        for j, target in enumerate(targets):
            if j < len(probs) and probs[j] is not None and probs[j].shape[1] > 1:
                entry[f"{target}_risk"] = round(float(probs[j][i, 1]), 4)
            else:
                entry[f"{target}_risk"] = 0.0
        entry["model_version"] = model_data.get("version", "multitask-v1")
        results.append(entry)
    return results
