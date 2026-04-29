from __future__ import annotations

import json
import logging
import os
import time
from datetime import datetime, timezone

import numpy as np

from app.config import settings

logger = logging.getLogger(__name__)


def _eval_dir() -> str:
    path = os.path.join(settings.model_dir, "evaluations")
    os.makedirs(path, exist_ok=True)
    return path


def evaluate_model(
    model_name: str,
    y_true: np.ndarray,
    y_pred: np.ndarray,
    y_proba: np.ndarray | None = None,
    backend: str = "xgboost",
    version: str = "unknown",
    sample_size: int = 0,
) -> dict:
    from sklearn.metrics import (
        accuracy_score,
        f1_score,
        mean_absolute_error,
        mean_squared_error,
        precision_score,
        r2_score,
        recall_score,
        roc_auc_score,
    )

    results = {
        "model_name": model_name,
        "backend": backend,
        "version": version,
        "sample_size": sample_size,
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }

    if y_proba is not None and len(np.unique(y_true)) == 2:
        try:
            results["roc_auc"] = round(float(roc_auc_score(y_true, y_proba)), 6)
        except ValueError:
            pass
        results["precision"] = round(float(precision_score(y_true, y_pred, zero_division=0)), 6)
        results["recall"] = round(float(recall_score(y_true, y_pred, zero_division=0)), 6)
        results["f1"] = round(float(f1_score(y_true, y_pred, zero_division=0)), 6)
        results["accuracy"] = round(float(accuracy_score(y_true, y_pred)), 6)
    else:
        results["mae"] = round(float(mean_absolute_error(y_true, y_pred)), 6)
        results["rmse"] = round(float(np.sqrt(mean_squared_error(y_true, y_pred))), 6)
        results["r2"] = round(float(r2_score(y_true, y_pred)), 6)

    path = os.path.join(_eval_dir(), f"{model_name}.jsonl")
    with open(path, "a") as f:
        f.write(json.dumps(results) + "\n")

    return results


def champion_challenger(
    model_name: str,
    challenger_metrics: dict,
    metric_key: str = "roc_auc",
    higher_is_better: bool = True,
    min_improvement: float = 0.01,
) -> dict:
    path = os.path.join(_eval_dir(), f"{model_name}.jsonl")
    if not os.path.exists(path):
        return {"decision": "promote", "reason": "no_previous_champion"}

    entries = []
    with open(path) as f:
        for line in f:
            try:
                entries.append(json.loads(line.strip()))
            except (json.JSONDecodeError, ValueError):
                continue

    if not entries:
        return {"decision": "promote", "reason": "no_previous_champion"}

    champion = entries[-1]
    champ_val = champion.get(metric_key)
    chall_val = challenger_metrics.get(metric_key)

    if champ_val is None or chall_val is None:
        return {"decision": "keep_champion", "reason": f"metric_{metric_key}_not_found"}

    diff = chall_val - champ_val
    improvement_pct = abs(diff) / max(abs(champ_val), 1e-8)

    if higher_is_better:
        if diff > min_improvement * abs(champ_val):
            return {
                "decision": "promote",
                "reason": f"challenger_better_by_{improvement_pct:.4f}",
                "champion_val": champ_val,
                "challenger_val": chall_val,
                "improvement": round(float(diff), 6),
            }
        else:
            return {
                "decision": "keep_champion",
                "reason": f"challenger_not_better_enough",
                "champion_val": champ_val,
                "challenger_val": chall_val,
                "improvement": round(float(diff), 6),
            }
    else:
        if diff < -min_improvement * abs(champ_val):
            return {
                "decision": "promote",
                "reason": f"challenger_better_by_{improvement_pct:.4f}",
                "champion_val": champ_val,
                "challenger_val": chall_val,
                "improvement": round(float(diff), 6),
            }
        else:
            return {
                "decision": "keep_champion",
                "reason": f"challenger_not_better_enough",
                "champion_val": champ_val,
                "challenger_val": chall_val,
                "improvement": round(float(diff), 6),
            }


def get_evaluation_history(model_name: str, limit: int = 20) -> list[dict]:
    path = os.path.join(_eval_dir(), f"{model_name}.jsonl")
    if not os.path.exists(path):
        return []

    entries = []
    with open(path) as f:
        for line in f:
            try:
                entries.append(json.loads(line.strip()))
            except (json.JSONDecodeError, ValueError):
                continue

    return entries[-limit:]


def check_performance_alert(
    model_name: str,
    metric_key: str = "roc_auc",
    threshold: float = 0.6,
    window: int = 5,
) -> dict:
    history = get_evaluation_history(model_name, limit=window)
    if not history:
        return {"model": model_name, "alert": False, "reason": "no_data"}

    recent = [h.get(metric_key) for h in history if metric_key in h]
    if not recent:
        return {"model": model_name, "alert": False, "reason": "metric_not_found"}

    avg_metric = float(np.mean(recent))
    alert = avg_metric < threshold

    return {
        "model": model_name,
        "alert": alert,
        "metric": metric_key,
        "average_value": round(avg_metric, 6),
        "threshold": threshold,
        "window_size": len(recent),
        "reason": "below_threshold" if alert else "ok",
    }
