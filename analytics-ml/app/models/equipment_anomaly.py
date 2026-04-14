from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler

from app.config import settings

MODEL_FILENAME = "equipment_anomaly.pkl"

FEATURE_COLUMNS = [
    "avg_conductivity",
    "max_quarter_asymmetry",
    "avg_milk_temperature",
    "std_milk_temperature",
    "avg_milk_yield_per_visit",
    "avg_milk_speed",
    "anomaly_rate_7d",
]


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    X = df[FEATURE_COLUMNS].fillna(0).values

    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X)

    model = IsolationForest(
        n_estimators=100,
        contamination=0.05,
        random_state=42,
    )
    model.fit(X_scaled)

    preds = model.predict(X_scaled)
    n_anomalies = int((preds == -1).sum())

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    os.makedirs(settings.model_dir, exist_ok=True)
    joblib.dump({
        "model": model,
        "scaler": scaler,
        "features": FEATURE_COLUMNS,
        "version": "isolation-forest-v1",
    }, path)

    duration = time.time() - start
    return {
        "model_name": "equipment_anomaly",
        "samples": len(df),
        "metrics": {"anomalies_detected": n_anomalies, "anomaly_pct": round(n_anomalies / len(df) * 100, 2)},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame) -> list[dict]:
    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    if not os.path.exists(path):
        raise FileNotFoundError(f"Model not found: {path}")

    model_data = joblib.load(path)
    model = model_data["model"]
    scaler = model_data["scaler"]
    features = model_data["features"]
    version = model_data["version"]

    X = df[features].fillna(0).values
    X_scaled = scaler.transform(X)

    preds = model.predict(X_scaled)
    scores = model.decision_function(X_scaled)

    results = []
    for i, row in df.iterrows():
        is_anomaly = preds[i] == -1
        score = float(scores[i])

        flags = []
        if row.get("avg_conductivity", 0) > 75:
            flags.append("проводимость↑")
        if row.get("max_quarter_asymmetry", 0) > 10:
            flags.append("асимметрия↑")
        if row.get("avg_milk_temperature", 37) > 38.5:
            flags.append("температура↑")
        if row.get("avg_milk_speed", 2) < 1.0:
            flags.append("скорость↓")

        severity = "critical" if score < -0.3 else "warning" if score < -0.1 else "normal"

        results.append({
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "is_anomaly": is_anomaly,
            "anomaly_score": round(score, 4),
            "severity": severity if is_anomaly else "normal",
            "flags": flags,
            "device_address": int(row.get("device_address", 0)) or None,
            "model_version": version,
        })

    return results
