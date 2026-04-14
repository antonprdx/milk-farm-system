from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score
from xgboost import XGBClassifier

from app.config import settings

MODEL_FILENAME = "ketosis_warning_xgb.pkl"

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
        "model_name": "ketosis_warning",
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

        results.append({
            "animal_id": int(row["animal_id"]),
            "animal_name": row.get("animal_name"),
            "risk_probability": round(prob, 4),
            "risk_type": rtype,
            "severity": severity,
            "fpr_current": round(fpr, 3),
            "fpr_trend": round(float(row.get("fpr_trend", 0) or 0), 4),
            "contributing_factors": contributing,
            "model_version": version,
        })

    return results
