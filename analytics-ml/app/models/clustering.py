from __future__ import annotations

import os
import time

import joblib
import numpy as np
import pandas as pd
from sklearn.cluster import KMeans
from sklearn.metrics import silhouette_score
from sklearn.preprocessing import StandardScaler

from app.config import settings

MODEL_FILENAME = "cow_clusters.pkl"

FEATURE_COLS = [
    "avg_milk",
    "std_milk",
    "avg_rumination",
    "avg_activity",
    "avg_feed",
    "milk_cv",
    "dim_days",
    "lactation_number",
]

PRODUCTIVITY_LABELS = [
    "Низкопродуктивные",
    "Ниже среднего",
    "Средние",
    "Выше среднего",
    "Высокопродуктивные",
    "Высокой продуктивности",
    "Очень высокая",
    "Элитные",
]


def _prepare_features(df: pd.DataFrame) -> tuple[pd.DataFrame, list[str]]:
    df = df.copy()
    df["avg_milk"] = df["avg_milk"].fillna(0)
    df["std_milk"] = df["std_milk"].fillna(0)
    df["avg_rumination"] = df["rumination_minutes"].fillna(0)
    df["avg_activity"] = df["activity_counter"].fillna(0)
    df["avg_feed"] = df["feed_amount"].fillna(0)
    df["dim_days"] = df["dim_days"].fillna(0)
    df["lactation_number"] = df["lactation_number"].fillna(0)

    avg = df["avg_milk"].replace(0, np.nan)
    df["milk_cv"] = (df["std_milk"] / avg).fillna(0)

    return df, FEATURE_COLS


def _find_best_k(X_scaled: np.ndarray, k_min: int = 2, k_max: int = 8) -> int:
    n_samples = X_scaled.shape[0]
    k_max = min(k_max, n_samples - 1)

    if k_max < k_min:
        return k_min

    best_k = k_min
    best_score = -1.0

    for k in range(k_min, k_max + 1):
        km = KMeans(n_clusters=k, random_state=42, n_init=10)
        labels = km.fit_predict(X_scaled)
        if len(np.unique(labels)) < 2:
            continue
        score = silhouette_score(X_scaled, labels, sample_size=min(5000, n_samples))
        if score > best_score:
            best_score = score
            best_k = k

    return best_k


def _label_clusters(centers: np.ndarray, feature_names: list[str], n_clusters: int) -> dict[int, str]:
    milk_idx = feature_names.index("avg_milk")
    cv_idx = feature_names.index("milk_cv")
    rum_idx = feature_names.index("avg_rumination")

    sorted_ids = sorted(range(n_clusters), key=lambda i: centers[i][milk_idx])

    labels = {}
    for rank, cid in enumerate(sorted_ids):
        center = centers[cid]
        milk = center[milk_idx]
        cv = center[cv_idx]
        rum = center[rum_idx]

        base = PRODUCTIVITY_LABELS[min(rank, len(PRODUCTIVITY_LABELS) - 1)]

        modifiers = []
        if cv > 0.5:
            modifiers.append("нестабильные")
        if rum > 0.5 and milk > 0:
            modifiers.append("эффективные")

        if modifiers:
            labels[cid] = f"{base} ({', '.join(modifiers)})"
        else:
            labels[cid] = base

    return labels


def train(df: pd.DataFrame) -> dict:
    start = time.time()

    features_df, feature_cols = _prepare_features(df)
    if len(features_df) < 2:
        raise ValueError(f"Not enough cows ({len(features_df)}) for clustering")

    X = features_df[feature_cols].values
    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X)

    n_clusters = _find_best_k(X_scaled, k_min=2, k_max=min(8, len(features_df) - 1))

    model = KMeans(n_clusters=n_clusters, random_state=42, n_init=10)
    model.fit(X_scaled)

    sil_score = silhouette_score(X_scaled, model.labels_, sample_size=min(5000, len(X_scaled)))
    labels = _label_clusters(model.cluster_centers_, feature_cols, n_clusters)

    path = os.path.join(settings.model_dir, MODEL_FILENAME)
    os.makedirs(settings.model_dir, exist_ok=True)
    joblib.dump({
        "model": model,
        "scaler": scaler,
        "feature_cols": feature_cols,
        "labels": labels,
        "n_clusters": n_clusters,
        "version": "kmeans-v2",
    }, path)

    duration = time.time() - start
    cluster_sizes = pd.Series(model.labels_).value_counts().to_dict()

    return {
        "model_name": "cow_clusters",
        "samples": len(features_df),
        "metrics": {
            "n_clusters": n_clusters,
            "silhouette_score": round(sil_score, 4),
            "cluster_sizes": {str(k): int(v) for k, v in cluster_sizes.items()},
        },
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
