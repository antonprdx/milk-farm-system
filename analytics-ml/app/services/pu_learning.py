from __future__ import annotations

import numpy as np


def pu_adjust_labels(y: np.ndarray, positive_fraction: float = 0.3) -> np.ndarray:
    n_pos = int((y == 1).sum())
    n_total = len(y)
    if n_pos == 0 or n_total == 0:
        return y.copy()

    observed_pos_rate = n_pos / n_total
    if observed_pos_rate <= 0:
        return y.copy()

    prior = min(positive_fraction, 0.9)
    n_unlabeled = n_total - n_pos

    adjusted = y.copy().astype(float)

    for i in range(n_total):
        if y[i] == 1:
            adjusted[i] = 1.0
        else:
            p_unlabeled_pos = prior * (1 - observed_pos_rate)
            p_unlabeled_total = max(1 - observed_pos_rate, 1e-8)
            adjusted[i] = min(p_unlabeled_pos / p_unlabeled_total, 1.0)

    return adjusted


def compute_sample_weights(y: np.ndarray, pu_labels: np.ndarray) -> np.ndarray:
    weights = np.ones(len(y), dtype=float)
    for i in range(len(y)):
        if y[i] == 1:
            weights[i] = 1.0 / max(pu_labels[i], 0.1)
        else:
            weights[i] = 1.0 / max(1.0 - pu_labels[i], 0.1)
    total = weights.sum()
    if total > 0:
        weights = weights / total * len(y)
    return weights


def merge_real_labels(
    rule_labels: np.ndarray,
    confirmed_labels: np.ndarray | None = None,
    confirmed_mask: np.ndarray | None = None,
) -> tuple[np.ndarray, np.ndarray]:
    if confirmed_labels is None or confirmed_mask is None:
        return rule_labels, np.ones(len(rule_labels), dtype=float)

    labels = rule_labels.copy()
    weights = np.ones(len(rule_labels), dtype=float)

    for i in range(len(labels)):
        if confirmed_mask[i]:
            labels[i] = confirmed_labels[i]
            weights[i] = 3.0
        else:
            weights[i] = 1.0

    return labels, weights
