from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.model_selection import cross_val_score

from app.config import settings
from app.services.fillna import fillna_with_medians
from app.services.model_versioning import save_with_version

MODEL_FILENAME = "milk_forecast_xgb.pkl"
ONNX_FILENAME = "milk_forecast_xgb.onnx"

FORECAST_FEATURES = [
    "milk_lag1", "milk_lag7", "milk_roll7", "milk_roll30",
    "milk_diff1", "milk_diff7", "feed_amount", "rumination", "activity",
    "month", "day_of_week",
]

DIRECT_HORIZONS = [1, 7, 14, 30]


def _build_features(df: pd.DataFrame) -> tuple[pd.DataFrame, pd.Series]:
    df = df.sort_values("date").copy()

    features = pd.DataFrame(index=df.index)
    features["milk_lag1"] = df["milk_amount"].shift(1)
    features["milk_lag7"] = df["milk_amount"].shift(7)
    features["milk_roll7"] = df["milk_amount"].rolling(7, min_periods=1).mean()
    features["milk_roll30"] = df["milk_amount"].rolling(30, min_periods=1).mean()
    features["milk_diff1"] = df["milk_amount"].diff(1)
    features["milk_diff7"] = df["milk_amount"].diff(7)
    features["feed_amount"] = df["feed_amount"].fillna(0)
    features["rumination"] = df["rumination_minutes"].fillna(0)
    features["activity"] = df["activity_counter"].fillna(0)

    if "date" in df.columns:
        features["month"] = pd.to_datetime(df["date"]).dt.month
        features["day_of_week"] = pd.to_datetime(df["date"]).dt.dayofweek
    else:
        features["month"] = 1
        features["day_of_week"] = 0

    valid = features.dropna(subset=["milk_lag1"])
    target = df.loc[valid.index, "milk_amount"]

    return features.loc[valid.index], target


def _build_direct_target(df: pd.DataFrame, horizon: int) -> pd.Series | None:
    df_sorted = df.sort_values("date").copy()
    values = df_sorted["milk_amount"].values
    if len(values) <= horizon:
        return None
    target = pd.Series(np.nan, index=df_sorted.index)
    target.iloc[:-horizon] = values[horizon:]
    return target


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    X_all, y_all = _build_features(df)
    if len(X_all) < 10:
        raise ValueError(f"Not enough data: {len(X_all)} rows")

    from app.services.hyperopt import tune_regressor, get_model_instance
    base_params, backend = tune_regressor(X_all.values, y_all.values, n_trials=30, timeout=120)
    backend_str = base_params.pop("_backend", backend)

    base_params_copy = {k: v for k, v in base_params.items()}

    cv = min(5, max(2, len(X_all) // 20))
    cv_scores = cross_val_score(
        get_model_instance(dict(base_params_copy), backend_str, "regressor"),
        X_all.values, y_all.values, cv=cv, scoring="neg_mean_absolute_error",
    )

    model = get_model_instance(dict(base_params_copy), backend_str, "regressor")
    model.fit(X_all.values, y_all.values)

    q10_params = dict(base_params_copy)
    q90_params = dict(base_params_copy)

    from app.services.hyperopt import get_model_instance as _gi

    q10 = _gi(q10_params, backend_str, "regressor")
    q90 = _gi(q90_params, backend_str, "regressor")

    if backend_str == "xgboost":
        try:
            from xgboost import XGBRegressor
            q10 = XGBRegressor(**{**q10_params, "objective": "reg:quantileerror", "quantile_alpha": 0.1})
            q90 = XGBRegressor(**{**q90_params, "objective": "reg:quantileerror", "quantile_alpha": 0.9})
        except TypeError:
            q10 = _gi(q10_params, backend_str, "regressor")
            q90 = _gi(q90_params, backend_str, "regressor")

    q10.fit(X_all.values, y_all.values)
    q90.fit(X_all.values, y_all.values)

    direct_models = {}
    for horizon in DIRECT_HORIZONS:
        y_direct = _build_direct_target(df, horizon)
        if y_direct is None:
            continue
        valid_mask = y_direct.notna()
        X_h = X_all.loc[valid_mask]
        y_h = y_direct[valid_mask]
        if len(X_h) < 10:
            continue
        m = _gi(dict(base_params_copy), backend_str, "regressor")
        m.fit(X_h.values, y_h.values)
        direct_models[horizon] = m

    medians = {}
    for col in FORECAST_FEATURES:
        if col in X_all.columns:
            vals = X_all[col].dropna()
            medians[col] = float(vals.median()) if len(vals) > 0 else 0.0

    if settings.mlflow_tracking_uri:
        try:
            import mlflow
            mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
            mlflow.set_experiment("milk_forecast")
            with mlflow.start_run():
                mlflow.log_params({
                    **base_params_copy,
                    "quantile_models": "p10_p90",
                    "backend": backend_str,
                    "direct_horizons": str(list(direct_models.keys())),
                })
                mlflow.log_metrics({
                    "cv_mae_mean": float(-cv_scores.mean()),
                    "cv_mae_std": float(cv_scores.std()),
                    "samples": len(X_all),
                    "duration_seconds": round(time.time() - start, 2),
                })
        except Exception as e:
            import logging
            logging.getLogger(__name__).warning("MLflow logging failed: %s", e)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    save_with_version(path, {
        "model": model,
        "q10": q10,
        "q90": q90,
        "direct_models": direct_models,
        "medians": medians,
        "backend": backend_str,
        "params": base_params_copy,
        "version": "xgboost-v3-multistep",
    })

    try:
        from app.services.onnx_utils import save_model_onnx
        onnx_path = os.path.join(settings.model_dir, ONNX_FILENAME)
        save_model_onnx(model, FORECAST_FEATURES, onnx_path, task="regress")
    except Exception as e:
        import logging
        logging.getLogger(__name__).warning("ONNX export failed: %s", e)

    duration = time.time() - start
    return {
        "model_name": "milk_forecast",
        "samples": len(X_all),
        "metrics": {"cv_mae_mean": float(-cv_scores.mean()), "cv_mae_std": float(cv_scores.std())},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame, days: int = 30) -> dict:
    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    if not os.path.exists(path):
        raise FileNotFoundError(f"Model not found: {path}")

    model_data = joblib.load(path)
    model = model_data["model"]
    q10 = model_data.get("q10")
    q90 = model_data.get("q90")
    direct_models = model_data.get("direct_models", {})
    version = model_data.get("version", "xgboost-v1")
    model_n_features = model.n_features_in_ if hasattr(model, "n_features_in_") else len(FORECAST_FEATURES)
    use_time_features = model_n_features > 9

    from app.services.feature_forecast import forecast_features
    feature_forecasts = forecast_features(df, days)

    df = df.sort_values("date").copy()
    current_avg = df["milk_amount"].mean()

    recent_7_vals = df["milk_amount"].tail(7).tolist()
    recent_30_vals = df["milk_amount"].tail(30).tolist()
    last_milk = float(df["milk_amount"].iloc[-1]) if len(df) > 0 else float(current_avg or 0)

    feed_forecast = feature_forecasts["feed_amount"]
    rum_forecast = feature_forecasts["rumination"]
    act_forecast = feature_forecasts["activity"]

    direct_preds = {}
    for horizon, dmodel in direct_models.items():
        if horizon <= days:
            row = _build_current_features(
                last_milk, recent_7_vals, recent_30_vals, current_avg,
                feed_forecast, rum_forecast, act_forecast,
                df, 0, use_time_features,
            )
            direct_preds[horizon] = float(dmodel.predict(row)[0])

    forecast = []
    for offset in range(1, days + 1):
        lag1 = forecast[-1]["predicted_milk"] if forecast else last_milk
        lag7 = (
            forecast[-7]["predicted_milk"]
            if len(forecast) >= 7
            else (recent_7_vals[-(7 - len(forecast))] if len(forecast) < 7 and len(recent_7_vals) >= (7 - len(forecast)) else current_avg)
        )

        all_recent = [f["predicted_milk"] for f in forecast[-6:]] + [last_milk]
        roll7 = float(np.mean(all_recent))

        history_for_30 = recent_30_vals + [f["predicted_milk"] for f in forecast]
        roll30 = float(np.mean(history_for_30[-30:])) if history_for_30 else float(current_avg or 0)

        prev1 = forecast[-1]["predicted_milk"] if forecast else last_milk
        diff1 = prev1 - (forecast[-2]["predicted_milk"] if len(forecast) >= 2 else last_milk)
        diff7 = prev1 - (forecast[-7]["predicted_milk"] if len(forecast) >= 7 else (recent_7_vals[0] if recent_7_vals else last_milk))

        decay_weight = min(offset / 30.0, 1.0) * 0.3

        feature_list = [lag1, lag7, roll7, roll30, diff1, diff7,
                        feed_forecast[offset - 1], rum_forecast[offset - 1],
                        act_forecast[offset - 1]]
        if use_time_features:
            feature_list += [1, 0]
        feature_row = np.array([feature_list])
        pred = float(model.predict(feature_row)[0])

        pred = pred * (1 - decay_weight) + current_avg * decay_weight

        if offset in direct_preds:
            blend_weight = 0.6
            pred = pred * (1 - blend_weight) + direct_preds[offset] * blend_weight

        if q10 is not None and q90 is not None:
            lower = float(q10.predict(feature_row)[0])
            upper = float(q90.predict(feature_row)[0])
            if lower > pred:
                lower = pred
            if upper < pred:
                upper = pred
        else:
            std_est = abs(pred) * 0.1
            lower = pred - 1.96 * std_est
            upper = pred + 1.96 * std_est

        forecast.append({
            "day_offset": offset,
            "predicted_milk": round(max(pred, 0), 2),
            "lower_bound": round(max(lower, 0), 2),
            "upper_bound": round(max(upper, 0), 2),
        })

    shap_explanation = _compute_shap(model, feature_row, use_time_features)

    return {
        "current_daily_avg": round(current_avg, 2) if current_avg else None,
        "forecast": forecast,
        "model_version": version,
        "shap_explanation": shap_explanation,
    }


def _compute_shap(model, feature_row, use_time_features):
    if model is None:
        return None
    try:
        from app.services.shap_explain import explain_prediction
        feature_names = list(FORECAST_FEATURES[:9])
        if use_time_features:
            feature_names += ["month", "day_of_week"]
        explanations = explain_prediction(model, feature_row, feature_names, top_k=5)
        if explanations:
            return explanations[0]
    except Exception:
        pass
    return None


def _build_current_features(
    last_milk, recent_7_vals, recent_30_vals, current_avg,
    feed_forecast, rum_forecast, act_forecast,
    df, offset, use_time_features=True,
):
    lag1 = last_milk
    lag7 = recent_7_vals[-1] if recent_7_vals else current_avg
    roll7 = float(np.mean(recent_7_vals)) if recent_7_vals else current_avg
    roll30 = float(np.mean(recent_30_vals)) if recent_30_vals else current_avg
    diff1 = 0.0
    diff7 = 0.0

    feature_list = [lag1, lag7, roll7, roll30, diff1, diff7,
                     feed_forecast[offset], rum_forecast[offset], act_forecast[offset]]
    if use_time_features:
        feature_list += [1, 0]
    return np.array([feature_list])
