from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score
from xgboost import XGBRegressor

from app.config import settings

MODEL_FILENAME = "feed_recommendation_xgb.pkl"

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


def _create_target(df: pd.DataFrame) -> pd.Series:
    return (
        df["avg_milk_7d"] * 0.45
        + df["avg_rumination_7d"] * 0.005
        + df["dim_days"].clip(0, 100) * 0.02
        + 5.0
    )


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    X = df[FEATURE_COLUMNS].fillna(0).values
    y = _create_target(df).values

    model = XGBRegressor(
        n_estimators=100,
        max_depth=4,
        learning_rate=0.1,
        subsample=0.8,
        colsample_bytree=0.8,
        random_state=42,
    )

    cv = min(5, max(2, len(df) // 20))
    cv_scores = cross_val_score(model, X, y, cv=cv, scoring="neg_mean_absolute_error")

    model.fit(X, y)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    os.makedirs(settings.model_dir, exist_ok=True)
    joblib.dump({"model": model, "features": FEATURE_COLUMNS, "version": "xgboost-v1"}, path)

    duration = time.time() - start
    return {
        "model_name": "feed_recommendation",
        "samples": len(df),
        "metrics": {"cv_mae_mean": float(-cv_scores.mean()), "cv_mae_std": float(cv_scores.std())},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame) -> list[dict]:
    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    if not os.path.exists(path):
        raise FileNotFoundError(f"Model not found: {path}")

    model_data = joblib.load(path)
    model = model_data["model"]
    features = model_data["features"]
    version = model_data["version"]

    X = df[features].fillna(0).values
    preds = model.predict(X)

    results = []
    for i, row in df.iterrows():
        recommended = round(max(float(preds[i]), 0), 2)
        current = float(row.get("avg_feed_7d", 0) or 0)
        diff = round(recommended - current, 2)

        results.append({
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "current_feed_avg": round(current, 2),
            "recommended_feed": recommended,
            "difference_kg": diff,
            "suggestion": "increase" if diff > 1.0 else "decrease" if diff < -1.0 else "maintain",
            "dim_days": int(row.get("dim_days", 0) or 0),
            "lactation_number": int(row.get("lactation_number", 0) or 0),
            "model_version": version,
        })

    return results
