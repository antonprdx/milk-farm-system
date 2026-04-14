from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score
from xgboost import XGBRegressor

from app.config import settings

MODEL_FILENAME = "culling_xgb.pkl"

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

    cv_scores = cross_val_score(model, X, y, cv=min(5, max(2, len(df) // 10)), scoring="r2")

    model.fit(X, y)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    os.makedirs(settings.model_dir, exist_ok=True)
    joblib.dump({"model": model, "features": FEATURE_COLUMNS, "version": "xgboost-v1"}, path)

    duration = time.time() - start
    return {
        "model_name": "culling",
        "samples": len(df),
        "metrics": {"cv_r2_mean": float(cv_scores.mean()), "cv_r2_std": float(cv_scores.std())},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame, model_data: dict | None = None) -> list[dict]:
    if model_data is None:
        path = os.path.join(settings.model_dir, MODEL_FILENAME)
        if not os.path.exists(path):
            raise FileNotFoundError(f"Model not found: {path}")
        model_data = joblib.load(path)

    model = model_data["model"]
    features = model_data["features"]
    version = model_data["version"]

    X = df[features].fillna(0).values
    predicted_days = model.predict(X)

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

        results.append({
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "risk_probability": round(risk_prob, 4),
            "expected_days_remaining": max(int(expected_days), 0),
            "risk_factors": risk_factors,
            "model_version": version,
        })

    return results
