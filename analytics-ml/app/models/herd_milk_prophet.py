from __future__ import annotations

import logging
import os
import time

import numpy as np
import pandas as pd

from app.config import settings
from app.services.model_versioning import save_with_version

logger = logging.getLogger(__name__)

MODEL_FILENAME = "herd_milk_prophet.pkl"


def train(df: pd.DataFrame) -> dict:
    start = time.time()
    try:
        from prophet import Prophet

        prophet_df = df.rename(columns={"ds": "ds", "y": "y"})
        if "ds" not in prophet_df.columns or "y" not in prophet_df.columns:
            raise ValueError("DataFrame must have 'ds' and 'y' columns")

        model = Prophet(
            yearly_seasonality=True,
            weekly_seasonality=True,
            daily_seasonality=False,
            uncertainty_samples=500,
        )

        if "temperature" in df.columns:
            model.add_regressor("temperature")
        if "humidity" in df.columns:
            model.add_regressor("humidity")
        if "milking_cows_count" in df.columns:
            model.add_regressor("milking_cows_count")

        model.fit(prophet_df)

        path = os.path.join(settings.model_dir, MODEL_FILENAME)
        regressors = [c for c in ["temperature", "humidity", "milking_cows_count"] if c in df.columns]
        save_with_version(path, {
            "model": model,
            "regressors": regressors,
            "version": "prophet-v2",
        })

        if settings.mlflow_tracking_uri:
            try:
                import mlflow
                mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
                mlflow.set_experiment("herd_milk_prophet")
                with mlflow.start_run():
                    mlflow.log_params({
                        "yearly_seasonality": True,
                        "weekly_seasonality": True,
                        "regressors": str(regressors),
                    })
                    mlflow.log_metrics({
                        "samples": len(df),
                        "duration_seconds": round(time.time() - start, 2),
                    })
            except Exception as e:
                logger.warning("MLflow logging failed: %s", e)

        duration = time.time() - start
        return {
            "model_name": "herd_milk_prophet",
            "samples": len(df),
            "metrics": {"method": "prophet", "regressors": regressors},
            "duration_seconds": round(duration, 2),
        }
    except ImportError:
        logger.warning("prophet not installed, skipping training")
        return {
            "model_name": "herd_milk_prophet",
            "samples": len(df),
            "metrics": {"method": "prophet", "error": "not_installed"},
            "duration_seconds": 0.0,
        }


def predict(df: pd.DataFrame, periods: int = 30, include_shap: bool = False) -> dict:
    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    model_data = None

    from app.services.model_cache import get_model
    model_data = get_model("herd_milk_prophet", path)

    if model_data is not None:
        model = model_data["model"]
        regressors = model_data.get("regressors", [])
        version = model_data.get("version", "prophet-v2")
    else:
        try:
            from prophet import Prophet
        except ImportError:
            raise FileNotFoundError("prophet not installed and no saved model found")

        prophet_df = df.rename(columns={"ds": "ds", "y": "y"})
        if "ds" not in prophet_df.columns or "y" not in prophet_df.columns:
            raise ValueError("DataFrame must have 'ds' and 'y' columns")

        model = Prophet(
            yearly_seasonality=True,
            weekly_seasonality=True,
            daily_seasonality=False,
            uncertainty_samples=500,
        )
        model.fit(prophet_df)
        regressors = []
        version = "prophet-v1-fallback"

    future = model.make_future_dataframe(periods=periods)

    if regressors and model_data is not None:
        for reg in regressors:
            if reg in df.columns:
                reg_vals = df[reg].values
                if len(reg_vals) >= periods:
                    future[reg] = np.nan
                    future.loc[future.index[-len(df):], reg] = reg_vals[:len(future) - len(df) + periods] if len(reg_vals) > periods else reg_vals

    forecast = model.predict(future)

    result = forecast.tail(periods)[["ds", "yhat", "yhat_lower", "yhat_upper"]].copy()
    result["ds"] = result["ds"].dt.strftime("%Y-%m-%d")

    trend_pct = 0.0
    if len(forecast) > 14:
        recent = forecast["yhat"].iloc[-7:].mean()
        earlier = forecast["yhat"].iloc[-14:-7].mean()
        if earlier > 0:
            trend_pct = ((recent - earlier) / earlier) * 100.0

    direction = (
        "significant_up" if trend_pct > 5
        else "up" if trend_pct > 2
        else "significant_down" if trend_pct < -5
        else "down" if trend_pct < -2
        else "stable"
    )

    forecast_dates = result["ds"].values
    yhat = result["yhat"].values
    yhat_lower = result["yhat_lower"].values
    yhat_upper = result["yhat_upper"].values

    forecast_days = []
    for i in range(len(result)):
        forecast_days.append({
            "date": forecast_dates[i],
            "predicted": round(max(float(yhat[i]), 0), 2),
            "lower": round(max(float(yhat_lower[i]), 0), 2),
            "upper": round(max(float(yhat_upper[i]), 0), 2),
        })

    return {
        "forecast": forecast_days,
        "trend_direction": direction,
        "trend_percent": round(trend_pct, 2),
        "model_version": version,
    }
