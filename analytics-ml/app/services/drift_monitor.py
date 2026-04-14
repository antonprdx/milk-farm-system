from __future__ import annotations

import json
import os
import time
from datetime import datetime

import numpy as np
import pandas as pd

from app.config import settings


def _drift_dir() -> str:
    path = os.path.join(settings.model_dir, "drift")
    os.makedirs(path, exist_ok=True)
    return path


def record_predictions(model_name: str, predictions: list[dict]) -> None:
    scores = []
    for p in predictions:
        for key in ("risk_probability", "estrus_probability", "risk_score", "anomaly_score"):
            if key in p:
                scores.append(float(p[key]))
                break

    if not scores:
        return

    path = os.path.join(_drift_dir(), f"{model_name}.jsonl")
    entry = {
        "ts": datetime.utcnow().isoformat(),
        "n": len(scores),
        "mean": float(np.mean(scores)),
        "std": float(np.std(scores)),
        "min": float(np.min(scores)),
        "max": float(np.max(scores)),
    }
    with open(path, "a") as f:
        f.write(json.dumps(entry) + "\n")


def check_drift(model_name: str, window: int = 100, threshold: float = 2.0) -> dict:
    path = os.path.join(_drift_dir(), f"{model_name}.jsonl")
    if not os.path.exists(path):
        return {"model": model_name, "status": "no_data", "drift_detected": False}

    entries = []
    with open(path) as f:
        for line in f:
            try:
                entries.append(json.loads(line.strip()))
            except (json.JSONDecodeError, ValueError):
                continue

    if len(entries) < 10:
        return {"model": model_name, "status": "insufficient_data", "drift_detected": False}

    recent = entries[-min(window, len(entries)):]
    baseline = entries[:max(len(entries) // 2, 10)]

    recent_mean = np.mean([e["mean"] for e in recent])
    baseline_mean = np.mean([e["mean"] for e in baseline])
    baseline_std = np.std([e["mean"] for e in baseline]) or 0.001

    z_score = abs(recent_mean - baseline_mean) / baseline_std
    drift = z_score > threshold

    return {
        "model": model_name,
        "status": "drift_detected" if drift else "ok",
        "drift_detected": drift,
        "z_score": round(float(z_score), 4),
        "recent_mean": round(float(recent_mean), 4),
        "baseline_mean": round(float(baseline_mean), 4),
        "samples": len(entries),
    }
