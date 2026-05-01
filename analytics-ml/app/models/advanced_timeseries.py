from __future__ import annotations

import numpy as np
import pandas as pd


def auto_arima_select(
    series: np.ndarray,
    max_p: int = 3,
    max_d: int = 2,
    max_q: int = 3,
    period: int = 7,
) -> dict:
    best_aic = float("inf")
    best_order = (0, 1, 1)

    for d in range(max_d + 1):
        differenced = series.copy()
        for _ in range(d):
            differenced = np.diff(differenced)
            if len(differenced) < 5:
                break
        if len(differenced) < 5:
            continue

        n = len(differenced)
        train_size = int(n * 0.8)
        if train_size < 3:
            continue
        train = differenced[:train_size]

        mean = np.mean(train)
        var = np.var(train) if np.var(train) > 1e-10 else 1.0

        for p in range(max_p + 1):
            for q in range(max_q + 1):
                k = p + q + 1
                if k >= train_size:
                    continue
                log_lik = -0.5 * train_size * np.log(2 * np.pi * var) - 0.5 * np.sum(
                    (train - mean) ** 2
                ) / var
                aic = 2 * k - 2 * log_lik
                if aic < best_aic:
                    best_aic = aic
                    best_order = (p, d, q)

    return {"order": best_order, "aic": best_aic}


def fit_arima(series: np.ndarray, order: tuple[int, int, int]) -> dict:
    p, d, q = order

    for _ in range(d):
        series = np.diff(series)
    n = len(series)
    if n < 3:
        return {"fitted": series, "residuals": np.zeros_like(series), "drift": 0.0}

    ar_coeffs = np.zeros(p)
    for i in range(p):
        y = series[p:]
        X = np.column_stack([series[p - i - 1 : n - i - 1]]) if p > 0 else np.ones((n - p, 1))
        if len(y) == len(X):
            ar_coeffs[i] = np.linalg.lstsq(X.reshape(-1, 1), y, rcond=None)[0][0]

    fitted = np.convolve(series, np.r_[1, -ar_coeffs], mode="full")[:n]
    residuals = series - fitted
    ma_coeff = 0.3
    if q > 0 and len(residuals) > 1:
        ma_coeff = np.corrcoef(residuals[:-1], residuals[1:])[0, 1]
        if np.isnan(ma_coeff):
            ma_coeff = 0.0

    drift = np.mean(series) if d > 0 else 0.0

    return {
        "fitted": fitted,
        "residuals": residuals,
        "drift": drift,
        "ar_coeffs": ar_coeffs,
        "ma_coeff": ma_coeff,
    }


def forecast_arima(model: dict, steps: int, original_series: np.ndarray) -> np.ndarray:
    fitted = model["fitted"]
    drift = model["drift"]
    ar_coeffs = model.get("ar_coeffs", np.zeros(0))
    ma_coeff = model.get("ma_coeff", 0.0)
    p = len(ar_coeffs)
    residuals = model.get("residuals", np.zeros(1))

    history = list(fitted)
    resid_history = list(residuals[-5:]) if len(residuals) > 0 else [0.0]
    forecasts = []

    for _ in range(steps):
        ar_part = sum(
            ar_coeffs[i] * history[-(i + 1)] for i in range(min(p, len(history)))
        )
        ma_part = ma_coeff * resid_history[-1] if resid_history else 0.0
        val = ar_part + drift + ma_part
        forecasts.append(val)
        history.append(val)
        resid_history.append(0.0)

    return np.array(forecasts)


def nbets_forecast(
    series: np.ndarray,
    forecast_horizon: int,
    n_blocks: int = 3,
    hidden_size: int = 16,
    epochs: int = 100,
    learning_rate: float = 0.001,
    period: int = 7,
) -> dict:
    try:
        import torch
        import torch.nn as nn
    except ImportError:
        return _nbets_fallback(series, forecast_horizon, period)

    class NBeatsBlock(nn.Module):
        def __init__(self, input_size, hidden_size, output_size):
            super().__init__()
            self.fc1 = nn.Linear(input_size, hidden_size)
            self.fc2 = nn.Linear(hidden_size, hidden_size)
            self.fc3 = nn.Linear(hidden_size, output_size)

        def forward(self, x):
            x = torch.relu(self.fc1(x))
            x = torch.relu(self.fc2(x))
            return self.fc3(x)

    class NBeatsStack(nn.Module):
        def __init__(self, input_size, hidden_size, output_size, n_blocks):
            super().__init__()
            self.blocks = nn.ModuleList(
                [NBeatsBlock(input_size, hidden_size, output_size) for _ in range(n_blocks)]
            )

        def forward(self, x):
            residual = x
            forecasts = []
            for block in self.blocks:
                backcast = block(residual)
                forecast = block(residual)
                residual = residual - backcast
                forecasts.append(forecast)
            return residual, torch.stack(forecasts).sum(dim=0)

    n = len(series)
    if n < period * 2:
        return _nbets_fallback(series, forecast_horizon, period)

    input_size = min(n, period * 4)
    X = torch.FloatTensor(series[:-forecast_horizon][-input_size:]).unsqueeze(0)
    y = torch.FloatTensor(series[-forecast_horizon:]).unsqueeze(0)

    if X.shape[1] < 4 or y.shape[1] < 1:
        return _nbets_fallback(series, forecast_horizon, period)

    model = NBeatsStack(X.shape[1], hidden_size, y.shape[1], n_blocks)
    optimizer = torch.optim.Adam(model.parameters(), lr=learning_rate)
    criterion = nn.MSELoss()

    model.train()
    for _ in range(epochs):
        optimizer.zero_grad()
        _, forecast = model(X)
        loss = criterion(forecast, y)
        loss.backward()
        optimizer.step()

    model.eval()
    with torch.no_grad():
        full_input = torch.FloatTensor(series[-input_size:]).unsqueeze(0)
        _, forecast = model(full_input)

    trend = np.polyfit(np.arange(n), series, 1)
    seasonal = np.zeros(period)
    for i in range(min(period, n)):
        vals = series[i::period]
        if len(vals) > 0:
            seasonal[i] = np.mean(vals) - np.mean(series)

    result = forecast.numpy().flatten()
    if len(result) < forecast_horizon:
        trend_ext = trend[0] * np.arange(n, n + forecast_horizon) + trend[1]
        season_ext = np.tile(seasonal, (forecast_horizon // period) + 1)[:forecast_horizon]
        result = trend_ext + season_ext

    return {
        "forecast": result,
        "model_type": "n-beats",
        "trend": trend.tolist(),
        "seasonal": seasonal.tolist(),
    }


def _nbets_fallback(series: np.ndarray, steps: int, period: int) -> dict:
    n = len(series)
    trend = np.polyfit(np.arange(n), series, 1)
    trend_ext = trend[0] * np.arange(n, n + steps) + trend[1]

    seasonal = np.zeros(period)
    for i in range(min(period, n)):
        vals = series[i::period]
        if len(vals) > 0:
            seasonal[i] = np.mean(vals) - np.mean(series)

    season_ext = np.tile(seasonal, (steps // period) + 1)[:steps]
    forecast = trend_ext + season_ext

    return {
        "forecast": forecast,
        "model_type": "n-beats-fallback",
        "trend": trend.tolist(),
        "seasonal": seasonal.tolist(),
    }


def compute_tsfresh_features(series: np.ndarray) -> dict:
    n = len(series)
    if n < 3:
        return {}

    diff = np.diff(series)
    features = {
        "mean": float(np.mean(series)),
        "std": float(np.std(series)),
        "cv": float(np.std(series) / max(abs(np.mean(series)), 1e-10)),
        "skew": float(_skewness(series)),
        "kurtosis": float(_kurtosis(series)),
        "entropy": float(_approx_entropy(series)),
        "acf_lag1": float(np.corrcoef(series[:-1], series[1:])[0, 1])
        if n > 2
        else 0.0,
        "acf_lag7": float(np.corrcoef(series[:-7], series[7:])[0, 1])
        if n > 8
        else 0.0,
        "diff_mean": float(np.mean(diff)),
        "diff_std": float(np.std(diff)),
        "trend_slope": float(np.polyfit(np.arange(n), series, 1)[0]),
        "max_drawdown": float(_max_drawdown(series)),
        "stl_trend_strength": float(1 - np.var(series - _moving_avg(series, 7)) / max(np.var(series), 1e-10)),
        "stl_season_strength": float(
            1 - np.var(series - _seasonal_extract(series, 7)) / max(np.var(series), 1e-10)
        ),
    }

    return {k: 0.0 if np.isnan(v) or np.isinf(v) else v for k, v in features.items()}


def _skewness(x):
    n = len(x)
    m = np.mean(x)
    s = np.std(x)
    if s < 1e-10:
        return 0.0
    return float(np.mean(((x - m) / s) ** 3))


def _kurtosis(x):
    n = len(x)
    m = np.mean(x)
    s = np.std(x)
    if s < 1e-10:
        return 0.0
    return float(np.mean(((x - m) / s) ** 4) - 3)


def _approx_entropy(x, m=2, r=None):
    n = len(x)
    if n < m + 1:
        return 0.0
    if r is None:
        r = 0.2 * np.std(x)
    if r < 1e-10:
        return 0.0

    def _phi(m_val):
        counts = []
        for i in range(n - m_val + 1):
            template = x[i : i + m_val]
            dists = np.max(np.abs(np.lib.stride_tricks.sliding_window_view(x, m_val) - template), axis=1)
            counts.append(np.sum(dists <= r) / (n - m_val + 1))
        return np.mean(np.log(np.array(counts) + 1e-10))

    return abs(_phi(m) - _phi(m + 1))


def _max_drawdown(x):
    peak = x[0]
    max_dd = 0.0
    for val in x:
        if val > peak:
            peak = val
        dd = (peak - val) / max(abs(peak), 1e-10)
        if dd > max_dd:
            max_dd = dd
    return max_dd


def _moving_avg(x, window):
    result = np.copy(x)
    for i in range(len(x)):
        start = max(0, i - window // 2)
        end = min(len(x), i + window // 2 + 1)
        result[i] = np.mean(x[start:end])
    return result


def _seasonal_extract(x, period):
    seasonal = np.zeros(period)
    for i in range(period):
        vals = x[i::period]
        if len(vals) > 0:
            seasonal[i] = np.mean(vals)
    seasonal -= np.mean(seasonal)
    return np.tile(seasonal, (len(x) // period) + 1)[: len(x)]


def lstm_global_forecast(
    series: np.ndarray,
    forecast_horizon: int,
    lookback: int = 14,
    hidden_size: int = 64,
    num_layers: int = 2,
    epochs: int = 150,
    learning_rate: float = 0.001,
) -> dict:
    try:
        import torch
        import torch.nn as nn
    except ImportError:
        return _lstm_fallback(series, forecast_horizon)

    n = len(series)
    if n < lookback + 5:
        return _lstm_fallback(series, forecast_horizon)

    mean = np.mean(series)
    std = max(np.std(series), 1e-8)
    normed = (series - mean) / std

    X_list, y_list = [], []
    for i in range(lookback, n):
        X_list.append(normed[i - lookback : i])
        y_list.append(normed[i])
    if len(X_list) < 3:
        return _lstm_fallback(series, forecast_horizon)

    X_arr = np.array(X_list, dtype=np.float32)
    y_arr = np.array(y_list, dtype=np.float32)

    class _LSTMModel(nn.Module):
        def __init__(self, input_size, hidden_size, num_layers):
            super().__init__()
            self.lstm = nn.LSTM(input_size, hidden_size, num_layers, batch_first=True)
            self.fc = nn.Linear(hidden_size, 1)

        def forward(self, x):
            out, _ = self.lstm(x)
            return self.fc(out[:, -1, :])

    X_tensor = torch.FloatTensor(X_arr).unsqueeze(-1)
    y_tensor = torch.FloatTensor(y_arr)

    model = _LSTMModel(1, hidden_size, num_layers)
    optimizer = torch.optim.Adam(model.parameters(), lr=learning_rate)
    criterion = nn.MSELoss()

    model.train()
    for _ in range(epochs):
        optimizer.zero_grad()
        pred = model(X_tensor).squeeze(-1)
        loss = criterion(pred, y_tensor)
        loss.backward()
        optimizer.step()

    model.eval()
    current = normed[-lookback:].tolist()
    forecasts = []
    with torch.no_grad():
        for _ in range(forecast_horizon):
            inp = torch.FloatTensor(current[-lookback:]).unsqueeze(0).unsqueeze(-1)
            val = model(inp).item()
            forecasts.append(val)
            current.append(val)

    result = np.array(forecasts) * std + mean
    return {
        "forecast": result,
        "model_type": "lstm-global",
    }


def _lstm_fallback(series: np.ndarray, steps: int) -> dict:
    n = len(series)
    if n < 4:
        last_val = series[-1] if n > 0 else 0.0
        return {"forecast": np.full(steps, last_val), "model_type": "lstm-fallback"}
    trend = np.polyfit(np.arange(n), series, 1)
    trend_ext = trend[0] * np.arange(n, n + steps) + trend[1]
    ma = np.convolve(series, np.ones(7) / 7, mode="same")
    seasonal_dev = series[-min(n, 7) :] - ma[-min(n, 7) :]
    if len(seasonal_dev) > 0:
        season_ext = np.tile(seasonal_dev, (steps // len(seasonal_dev)) + 1)[:steps]
    else:
        season_ext = np.zeros(steps)
    return {"forecast": trend_ext + season_ext, "model_type": "lstm-fallback"}
