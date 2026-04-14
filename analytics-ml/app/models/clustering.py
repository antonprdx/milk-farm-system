from __future__ import annotations

import os
import time

import joblib
import pandas as pd
from sklearn.cluster import KMeans
from sklearn.preprocessing import StandardScaler

from app.config import settings

MODEL_FILENAME = "cow_clusters.pkl"


def _label_clusters(centers: np.ndarray, feature_names: list[str]) -> dict[int, str]:
    milk_idx = feature_names.index("avg_milk")
    cv_idx = feature_names.index("milk_cv")
    rum_idx = feature_names.index("avg_rumination")

    labels = {}
    for i, center in enumerate(centers):
        milk = center[milk_idx]
        cv = center[cv_idx]
        rum = center[rum_idx]

        if milk > 0.7 and cv < 0.2:
            labels[i] = "Высокопродуктивные"
        elif milk < -0.5:
            labels[i] = "Низкопродуктивные"
        elif cv > 0.5:
            labels[i] = "Нестабильные"
        elif rum > 0.5:
            labels[i] = "Эффективные"
        else:
            labels[i] = "Средние"

    return labels


def _prepare_features(df: pd.DataFrame) -> tuple[pd.DataFrame, list[str]]:
    df = df.copy()
    df["milk_cv"] = df["std_milk"] / df["avg_milk"].replace(0, np.nan).fillna(0)
    df["avg_rumination"] = df["rumination_minutes"].fillna(0)
    df["avg_activity"] = df["activity_counter"].fillna(0)
    df["avg_feed"] = df["feed_amount"].fillna(0)
    df["avg_milk"] = df["avg_milk"].fillna(0)
    df["std_milk"] = df["std_milk"].fillna(0)

    feature_cols = ["avg_milk", "std_milk", "avg_rumination", "avg_activity", "avg_feed", "milk_cv"]
    return df, feature_cols


import numpy as np


def train(df: pd.DataFrame, n_clusters: int = 4) -> dict:
    start = time.time()

    features_df, feature_cols = _prepare_features(df)
    if len(features_df) < n_clusters:
        raise ValueError(f"Not enough cows ({len(features_df)}) for {n_clusters} clusters")

    X = features_df[feature_cols].values

    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X)

    model = KMeans(n_clusters=n_clusters, random_state=42, n_init=10)
    model.fit(X_scaled)

    labels = _label_clusters(model.cluster_centers_, feature_cols)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    os.makedirs(settings.model_dir, exist_ok=True)
    joblib.dump({
        "model": model,
        "scaler": scaler,
        "feature_cols": feature_cols,
        "labels": labels,
        "n_clusters": n_clusters,
        "version": "kmeans-v1",
    }, path)

    duration = time.time() - start
    cluster_sizes = pd.Series(model.labels_).value_counts().to_dict()

    return {
        "model_name": "cow_clusters",
        "samples": len(features_df),
        "metrics": {"n_clusters": n_clusters, "cluster_sizes": {str(k): int(v) for k, v in cluster_sizes.items()}},
        "duration_seconds": round(duration, 2),
    }


def predict(df: pd.DataFrame) -> list[dict]:
    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    if not os.path.exists(path):
        raise FileNotFoundError(f"Model not found: {path}")

    model_data = joblib.load(path)
    model = model_data["model"]
    scaler = model_data["scaler"]
    feature_cols = model_data["feature_cols"]
    labels = model_data["labels"]
    version = model_data["version"]

    features_df, _ = _prepare_features(df)
    X = features_df[feature_cols].values
    X_scaled = scaler.transform(X)

    cluster_ids = model.predict(X_scaled)
    distances = model.transform(X_scaled).min(axis=1)

    results = []
    for i, row in features_df.iterrows():
        cid = int(cluster_ids[i])
        results.append({
            "animal_id": int(row["animal_id"]),
            "animal_name": row["animal_name"],
            "cluster_id": cid,
            "cluster_name": labels.get(cid, f"Кластер {cid}"),
            "avg_milk": round(float(row["avg_milk"]), 2),
            "avg_rumination": round(float(row["avg_rumination"]), 1),
            "distance_to_center": round(float(distances[i]), 3),
            "model_version": version,
        })

    return results
