from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score
from xgboost import XGBClassifier

from app.config import settings

MODEL_FILENAME = "mastitis_xgb.pkl"

FEATURE_COLUMNS = [
    "age_years",
    "recent_scc",
    "scc_trend_ratio",
    "avg_conductivity",
    "milk_deviation",
    "dim_days",
    "avg_rumination_7d",
    "avg_activity_7d",
]


def _create_labels(df: pd.DataFrame) -> pd.Series:
    labels = pd.Series(0, index=df.index)
    labels[df["recent_scc"] > 300000] = 1
    labels[(df["recent_scc"] > 200000) & (df["scc_trend_ratio"] > 1.5)] = 1
    labels[(df["recent_scc"] > 150000) & (df["avg_conductivity"] > 55)] = 1
    labels[(df["milk_deviation"] < -0.15) & (df["recent_scc"] > 100000)] = 1
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

    cv_scores = cross_val_score(model, X, y, cv=min(5, max(2, len(np.unique(y)))), scoring="roc_auc")

    model.fit(X, y)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    os.makedirs(settings.model_dir, exist_ok=True)
    joblib.dump({"model": model, "features": FEATURE_COLUMNS, "version": "xgboost-v1"}, path)

    duration = time.time() - start
    return {
        "model_name": "mastitis",
        "samples": len(df),
        "metrics": {"cv_auc_mean": float(cv_scores.mean()), "cv_auc_std": float(cv_scores.std())},
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
    probs = model.predict_proba(X)[:, 1]

    results = []
    for i, row in df.iterrows():
        prob = float(probs[i])
        contributing = []
        if row.get("recent_scc", 0) > 300000:
            contributing.append("SCC>300k")
        if row.get("scc_trend_ratio", 1) > 2:
            contributing.append("SCC↑↑")
        elif row.get("scc_trend_ratio", 1) > 1.5:
            contributing.append("SCC↑")
        if row.get("avg_conductivity", 0) > 60:
            contributing.append("conductivity↑")
        if row.get("milk_deviation", 0) < -0.15:
            contributing.append("milk↓")
        if row.get("dim_days", 0) < 30:
            contributing.append("early_lactation")

        risk_level = "high" if prob >= 0.6 else "medium" if prob >= 0.3 else "low"

        results.append({
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "risk_probability": round(prob, 4),
            "risk_level": risk_level,
            "contributing_features": contributing,
            "model_version": version,
        })

    return results
