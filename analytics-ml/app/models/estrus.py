from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score
from xgboost import XGBClassifier

from app.config import settings

MODEL_FILENAME = "estrus_xgb.pkl"

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

    X = df[FEATURE_COLUMNS].fillna(0).values
    y = df["label"].values

    model = XGBClassifier(
        n_estimators=100,
        max_depth=4,
        learning_rate=0.1,
        subsample=0.8,
        colsample_bytree=0.8,
        random_state=42,
        eval_metric="logloss",
    )

    cv = min(5, max(2, len(np.unique(y))))
    cv_scores = cross_val_score(model, X, y, cv=cv, scoring="roc_auc")

    model.fit(X, y)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    os.makedirs(settings.model_dir, exist_ok=True)
    joblib.dump({"model": model, "features": FEATURE_COLUMNS, "version": "xgboost-v1"}, path)

    duration = time.time() - start
    return {
        "model_name": "estrus",
        "samples": len(df),
        "metrics": {"cv_auc_mean": float(cv_scores.mean()), "cv_auc_std": float(cv_scores.std())},
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
    probs = model.predict_proba(X)[:, 1]

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

        results.append({
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "estrus_probability": round(prob, 4),
            "status": status,
            "contributing_signals": contributing,
            "optimal_window": f"{dim}–{min(dim + 3, 150)} DIM" if 35 <= dim <= 130 else None,
            "model_version": version,
        })

    return results
