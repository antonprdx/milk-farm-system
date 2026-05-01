from __future__ import annotations

import os
import logging
import time
import threading
from typing import Any

from app.config import settings

logger = logging.getLogger(__name__)

_lock = threading.Lock()
_model_cache: dict[str, tuple[Any, float, str]] = {}
_onnx_cache: dict[str, tuple[Any, float]] = {}
_feature_cache: dict[str, tuple[Any, float]] = {}

FEATURE_TTL = 60.0


def get_model(name: str, path: str | None = None) -> dict | None:
    import joblib

    if path is None:
        return None
    if not os.path.exists(path):
        return None

    mtime = os.path.getmtime(path)
    with _lock:
        if name in _model_cache:
            cached_data, cached_mtime, cached_path = _model_cache[name]
            if cached_mtime == mtime and cached_path == path:
                return cached_data
        data = joblib.load(path)
        _model_cache[name] = (data, mtime, path)
        logger.debug("Model cache miss: %s (loaded from disk)", name)
        return data


def get_onnx_session(path: str) -> tuple[Any, list[str], str] | None:
    import onnxruntime as ort

    if not os.path.exists(path):
        return None

    mtime = os.path.getmtime(path)
    with _lock:
        if path in _onnx_cache:
            session, cached_mtime = _onnx_cache[path]
            if cached_mtime == mtime:
                return session

    session = ort.InferenceSession(path)
    meta = session.get_modelmeta().custom_metadata_map
    features = meta.get("feature_columns", "").split(",")
    task = meta.get("task", "classify")
    with _lock:
        _onnx_cache[path] = (session, mtime)
    logger.debug("ONNX cache miss: %s", path)
    return session, features, task


def get_features(name: str) -> tuple[Any, float] | None:
    with _lock:
        if name in _feature_cache:
            df, ts = _feature_cache[name]
            if time.time() - ts < FEATURE_TTL:
                return df, ts
    return None


def set_features(name: str, df: Any) -> None:
    with _lock:
        _feature_cache[name] = (df, time.time())


def invalidate_features(name: str | None = None) -> None:
    with _lock:
        if name:
            _feature_cache.pop(name, None)
        else:
            _feature_cache.clear()
