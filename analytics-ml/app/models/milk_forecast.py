from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score
from xgboost import XGBRegressor

from app.config import settings

MODEL_FILENAME = "milk_forecast_xgb.pkl"


def _build_features(df: pd.DataFrame) -> tuple[pd.DataFrame, pd.Series]:
    df = df.sort_values("date").copy()
    df["day_idx"] = range(len(df))

    features = pd.DataFrame()
    features["day_idx"] = df["day_idx"]
    features["milk_lag1"] = df["milk_amount"].shift(1)
    features["milk_lag7"] = df["milk_amount"].shift(7)
    features["milk_roll7"] = df["milk_amount"].rolling(7, min_periods=1).mean()
    features["milk_roll30"] = df["milk_amount"].rolling(30, min_periods=1).mean()
    features["feed_amount"] = df["feed_amount"].fillna(0)
    features["rumination"] = df["rumination_minutes"].fillna(0)
    features["activity"] = df["activity_counter"].fillna(0)
    features["milk_feed_ratio"] = features["milk_roll7"] / features["feed_amount"].replace(0, np.nan).fillna(1)

    valid = features.dropna(subset=["milk_lag1"])
    target = df.loc[valid.index, "milk_amount"]

    return features.loc[valid.index], target


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    X, y = _build_features(df)
    if len(X) < 10:
        raise ValueError(f"Not enough data: {len(X)} rows")

    model = XGBRegressor(
        n_estimators=100,
        max_depth=4,
        learning_rate=0.1,
        subsample=0.8,
        colsample_bytree=0.8,
        random_state=42,
    )

    cv = min(5, max(2, len(X) // 20))
    cv_scores = cross_val_score(model, X.values, y.values, cv=cv, scoring="neg_mean_absolute_error")

    model.fit(X.values, y.values)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    os.makedirs(settings.model_dir, exist_ok=True)
    joblib.dump({"model": model, "version": "xgboost-v1"}, path)

    duration = time.time() - start
    return {
        "model_name": "milk_forecast",
        "samples": len(X),
        "metrics": {"cv_mae_mean": float(-cv_scores.mean()), "cv_mae_std": float(cv_scores.std())},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame, days: int = 30) -> dict:
    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    if not os.path.exists(path):
        raise FileNotFoundError(f"Model not found: {path}")

    model_data = joblib.load(path)
    model = model_data["model"]
    version = model_data["version"]

    df = df.sort_values("date").copy()
    current_avg = df["milk_amount"].mean()

    forecast = []
    last_row = df.iloc[-1].copy() if len(df) > 0 else None

    for offset in range(1, days + 1):
        feature_row = {
            "day_idx": [len(df) + offset - 1],
            "milk_lag1": [forecast[-1]["predicted_milk"] if forecast else (current_avg or 0)],
            "milk_lag7": [forecast[-7]["predicted_milk"] if len(forecast) >= 7 else (current_avg or 0)],
            "milk_roll7": [
                np.mean([f["predicted_milk"] for f in forecast[-6:]] + [current_avg or 0])
                if forecast else (current_avg or 0)
            ],
            "milk_roll30": [current_avg or 0],
            "feed_amount": [df["feed_amount"].mean() or 0],
            "rumination": [df["rumination_minutes"].mean() or 0],
            "activity": [df["activity_counter"].mean() or 0],
            "milk_feed_ratio": [1.0],
        }
        X_pred = pd.DataFrame(feature_row)
        pred = float(model.predict(X_pred.values)[0])

        std_est = abs(pred) * 0.1
        forecast.append({
            "day_offset": offset,
            "predicted_milk": round(max(pred, 0), 2),
            "lower_bound": round(max(pred - 1.96 * std_est, 0), 2),
            "upper_bound": round(pred + 1.96 * std_est, 2),
        })

    return {
        "current_daily_avg": round(current_avg, 2) if current_avg else None,
        "forecast": forecast,
        "model_version": version,
    }
