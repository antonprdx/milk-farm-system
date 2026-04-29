from __future__ import annotations

import logging

import numpy as np

logger = logging.getLogger(__name__)

_SHAP_AVAILABLE = False
try:
    import shap
    _SHAP_AVAILABLE = True
except ImportError:
    pass


def is_available() -> bool:
    return _SHAP_AVAILABLE


def explain_prediction(model, X: np.ndarray, feature_names: list[str], top_k: int = 3) -> list[dict]:
    if not _SHAP_AVAILABLE:
        return _fallback_explanation(X, feature_names, top_k)

    try:
        X_float = X.astype(np.float32) if X.dtype != np.float32 else X

        try:
            explainer = shap.TreeExplainer(model)
            shap_values = explainer.shap_values(X_float)
        except Exception:
            explainer = shap.Explainer(model, X_float[:min(50, len(X_float))])
            result = explainer(X_float)
            shap_values = result.values

        if isinstance(shap_values, list):
            shap_values = shap_values[1] if len(shap_values) > 1 else shap_values[0]

        explanations = []
        for i in range(len(X)):
            row_shap = shap_values[i] if shap_values.ndim > 1 else shap_values
            contributions = []
            for j, fname in enumerate(feature_names):
                if j < len(row_shap):
                    contributions.append({
                        "feature": fname,
                        "value": float(X_float[i, j]) if X_float.ndim > 1 else float(X_float[i]),
                        "shap_value": round(float(row_shap[j]), 6),
                    })
            contributions.sort(key=lambda x: abs(x["shap_value"]), reverse=True)

            base_val = float(explainer.expected_value) if hasattr(explainer, "expected_value") else 0.0
            if isinstance(base_val, (list, np.ndarray)):
                base_val = float(base_val[-1])

            explanations.append({
                "top_features": contributions[:top_k],
                "base_value": round(base_val, 4),
            })

        return explanations

    except Exception as e:
        logger.warning("SHAP explanation failed: %s, using fallback", e)
        return _fallback_explanation(X, feature_names, top_k)


def compute_global_importance(model, X: np.ndarray, feature_names: list[str]) -> dict[str, float]:
    if not _SHAP_AVAILABLE:
        return {}

    try:
        explainer = shap.TreeExplainer(model)
        shap_values = explainer.shap_values(X.astype(np.float32))

        if isinstance(shap_values, list):
            shap_values = shap_values[1] if len(shap_values) > 1 else shap_values[0]

        mean_abs_shap = np.abs(shap_values).mean(axis=0)
        total = mean_abs_shap.sum() or 1.0

        importance = {}
        for i, fname in enumerate(feature_names):
            if i < len(mean_abs_shap):
                importance[fname] = round(float(mean_abs_shap[i] / total), 6)

        return dict(sorted(importance.items(), key=lambda x: x[1], reverse=True))

    except Exception as e:
        logger.warning("Global SHAP importance failed: %s", e)
        return {}


def _fallback_explanation(X: np.ndarray, feature_names: list[str], top_k: int) -> list[dict]:
    explanations = []
    for i in range(len(X)):
        row = X[i] if X.ndim > 1 else X
        deviations = []
        for j, fname in enumerate(feature_names):
            if j < len(row):
                deviations.append({
                    "feature": fname,
                    "value": float(row[j]),
                    "shap_value": 0.0,
                })
        deviations.sort(key=lambda x: abs(x["value"]), reverse=True)
        explanations.append({"top_features": deviations[:top_k], "base_value": 0.0})
    return explanations
