from __future__ import annotations

import numpy as np
import pandas as pd


def forecast_series(values: list[float], periods: int) -> list[float]:
    if not values:
        return [0.0] * periods

    arr = np.array(values, dtype=float)
    n = len(arr)

    if n < 7:
        return [float(np.mean(arr))] * periods

    recent_7 = arr[-7:]
    recent_30 = arr[-min(30, n):]

    level = float(np.mean(recent_7))
    trend = (float(np.mean(recent_7)) - float(np.mean(recent_30[:7]))) / 7.0 if n >= 14 else 0.0

    seasonality = np.zeros(7)
    if n >= 14:
        for i in range(7):
            vals = arr[i::7]
            if len(vals) > 1:
                seasonality[i] = float(np.mean(vals[-3:]) - np.mean(arr))
            else:
                seasonality[i] = 0.0

    result = []
    for h in range(periods):
        s = seasonality[(n + h) % 7]
        pred = level + (h + 1) * trend * 0.3 + s
        result.append(max(float(pred), 0.0))

    return result


def forecast_features(df: pd.DataFrame, periods: int) -> dict[str, list[float]]:
    df = df.sort_values("date").copy()

    feed_vals = df["feed_amount"].dropna().tolist() if "feed_amount" in df.columns else []
    rum_vals = df["rumination_minutes"].dropna().tolist() if "rumination_minutes" in df.columns else []
    act_vals = df["activity_counter"].dropna().tolist() if "activity_counter" in df.columns else []

    return {
        "feed_amount": forecast_series(feed_vals, periods),
        "rumination": forecast_series(rum_vals, periods),
        "activity": forecast_series(act_vals, periods),
    }
