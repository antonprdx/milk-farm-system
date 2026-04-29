from __future__ import annotations

import os

import numpy as np
import pandas as pd


def compute_medians(df: pd.DataFrame, columns: list[str]) -> dict[str, float]:
    medians = {}
    for col in columns:
        if col in df.columns:
            vals = df[col].dropna()
            medians[col] = float(vals.median()) if len(vals) > 0 else 0.0
        else:
            medians[col] = 0.0
    return medians


def apply_fillna(
    df: pd.DataFrame,
    columns: list[str],
    medians: dict[str, float],
) -> pd.DataFrame:
    df = df.copy()
    for col in columns:
        if col in df.columns:
            fill_value = medians.get(col, 0.0)
            df[col] = df[col].fillna(fill_value)
        else:
            df[col] = medians.get(col, 0.0)
    return df


def fillna_with_medians(
    df: pd.DataFrame,
    columns: list[str],
    medians: dict[str, float] | None = None,
) -> tuple[pd.DataFrame, dict[str, float]]:
    if medians is None:
        medians = compute_medians(df, columns)
    filled = apply_fillna(df, columns, medians)
    return filled, medians
