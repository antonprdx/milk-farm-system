from __future__ import annotations

import numpy as np
import pandas as pd
from sklearn.ensemble import RandomForestClassifier, RandomForestRegressor
from sklearn.feature_selection import SelectFromModel


def add_interaction_features(df: pd.DataFrame, feature_cols: list[str]) -> tuple[pd.DataFrame, list[str]]:
    df = df.copy()
    new_cols = []

    interaction_pairs = [
        ("avg_activity_7d", "avg_rumination_7d"),
        ("avg_activity_14d", "avg_rumination_14d"),
        ("activity_ratio_7d", "rumination_ratio_7d"),
        ("avg_milk_7d", "dim_days"),
        ("avg_feed_7d", "dim_days"),
        ("avg_milk_7d", "avg_feed_7d"),
        ("fpr_7d", "dim_days"),
        ("recent_scc", "avg_conductivity"),
    ]

    for col1, col2 in interaction_pairs:
        if col1 in df.columns and col2 in df.columns:
            name = f"{col1}_x_{col2}"
            df[name] = df[col1].fillna(0) * df[col2].fillna(0)
            new_cols.append(name)

    ratio_pairs = [
        ("avg_milk_7d", "avg_feed_7d", "milk_to_feed"),
        ("avg_activity_7d", "avg_rumination_7d", "activity_to_rumination"),
    ]

    for col1, col2, name in ratio_pairs:
        if col1 in df.columns and col2 in df.columns:
            denom = df[col2].fillna(0).replace(0, np.nan)
            df[name] = df[col1].fillna(0) / denom
            df[name] = df[name].fillna(0)
            new_cols.append(name)

    return df, new_cols


def add_seasonal_features(df: pd.DataFrame, date_col: str = "date") -> tuple[pd.DataFrame, list[str]]:
    df = df.copy()
    new_cols = []

    if date_col in df.columns:
        dates = pd.to_datetime(df[date_col], errors="coerce")
        df["month"] = dates.dt.month.fillna(1).astype(int)
        df["day_of_week"] = dates.dt.dayofweek.fillna(0).astype(int)
        df["day_of_year"] = dates.dt.dayofyear.fillna(1).astype(int)
        df["season_sin"] = np.sin(2 * np.pi * df["day_of_year"] / 365.0)
        df["season_cos"] = np.cos(2 * np.pi * df["day_of_year"] / 365.0)
        new_cols.extend(["month", "day_of_week", "day_of_year", "season_sin", "season_cos"])

    return df, new_cols


def add_polynomial_features(df: pd.DataFrame, columns: list[str], degree: int = 2) -> tuple[pd.DataFrame, list[str]]:
    df = df.copy()
    new_cols = []

    for col in columns:
        if col in df.columns:
            for d in range(2, degree + 1):
                name = f"{col}_pow{d}"
                df[name] = df[col].fillna(0) ** d
                new_cols.append(name)

    return df, new_cols


def select_features(
    X: np.ndarray,
    y: np.ndarray,
    feature_names: list[str],
    task: str = "classify",
    threshold: str = "median",
) -> tuple[list[str], np.ndarray]:
    if task == "classify":
        estimator = RandomForestClassifier(n_estimators=50, random_state=42, n_jobs=-1)
    else:
        estimator = RandomForestRegressor(n_estimators=50, random_state=42, n_jobs=-1)

    selector = SelectFromModel(estimator, threshold=threshold)
    selector.fit(X, y)

    selected_mask = selector.get_support()
    selected_names = [name for name, sel in zip(feature_names, selected_mask) if sel]

    if len(selected_names) == 0:
        importances = selector.estimator_.feature_importances_
        top_idx = np.argsort(importances)[-min(5, len(feature_names)):]
        selected_names = [feature_names[i] for i in sorted(top_idx)]
        selected_mask = np.zeros(len(feature_names), dtype=bool)
        for i in top_idx:
            selected_mask[i] = True

    return selected_names, selector.transform(X)


def get_feature_importance(
    X: np.ndarray,
    y: np.ndarray,
    feature_names: list[str],
    task: str = "classify",
) -> list[dict]:
    if task == "classify":
        estimator = RandomForestClassifier(n_estimators=50, random_state=42, n_jobs=-1)
    else:
        estimator = RandomForestRegressor(n_estimators=50, random_state=42, n_jobs=-1)

    estimator.fit(X, y)
    importances = estimator.feature_importances_

    result = []
    for i, name in enumerate(feature_names):
        result.append({
            "feature": name,
            "importance": round(float(importances[i]), 6),
        })

    result.sort(key=lambda x: x["importance"], reverse=True)
    return result
