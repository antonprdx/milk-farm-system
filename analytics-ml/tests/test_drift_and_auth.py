import json
import os
import tempfile
from unittest.mock import AsyncMock, MagicMock, patch

import numpy as np
import pandas as pd
import pytest

from app.auth import verify_api_key
from app.services.drift_monitor import check_drift, record_predictions


def test_record_predictions_writes_jsonl():
    with tempfile.TemporaryDirectory() as tmpdir:
        with patch("app.services.drift_monitor.settings") as mock_settings:
            mock_settings.model_dir = tmpdir
            record_predictions("mastitis", [
                {"animal_id": 1, "risk_probability": 0.8},
                {"animal_id": 2, "risk_probability": 0.3},
            ])
            path = os.path.join(tmpdir, "drift", "mastitis.jsonl")
            assert os.path.exists(path)
            with open(path) as f:
                entry = json.loads(f.readline())
            assert entry["n"] == 2
            assert entry["mean"] == pytest.approx(0.55, abs=0.01)


def test_record_predictions_skips_no_scores():
    with tempfile.TemporaryDirectory() as tmpdir:
        with patch("app.services.drift_monitor.settings") as mock_settings:
            mock_settings.model_dir = tmpdir
            record_predictions("test_model", [
                {"animal_id": 1, "animal_name": "Cow1"},
            ])
            path = os.path.join(tmpdir, "drift", "test_model.jsonl")
            assert not os.path.exists(path)


def test_check_drift_no_data():
    with tempfile.TemporaryDirectory() as tmpdir:
        with patch("app.services.drift_monitor.settings") as mock_settings:
            mock_settings.model_dir = tmpdir
            result = check_drift("nonexistent")
            assert result["status"] == "no_data"
            assert not result["drift_detected"]


def test_check_drift_insufficient_data():
    with tempfile.TemporaryDirectory() as tmpdir:
        drift_dir = os.path.join(tmpdir, "drift")
        os.makedirs(drift_dir, exist_ok=True)
        with open(os.path.join(drift_dir, "test.jsonl"), "w") as f:
            for i in range(5):
                f.write(json.dumps({"mean": 0.5}) + "\n")
        with patch("app.services.drift_monitor.settings") as mock_settings:
            mock_settings.model_dir = tmpdir
            result = check_drift("test")
            assert result["status"] == "insufficient_data"


def test_check_drift_no_drift():
    with tempfile.TemporaryDirectory() as tmpdir:
        drift_dir = os.path.join(tmpdir, "drift")
        os.makedirs(drift_dir, exist_ok=True)
        with open(os.path.join(drift_dir, "test.jsonl"), "w") as f:
            for i in range(20):
                f.write(json.dumps({"mean": 0.5 + i * 0.001}) + "\n")
        with patch("app.services.drift_monitor.settings") as mock_settings:
            mock_settings.model_dir = tmpdir
            result = check_drift("test")
            assert result["status"] == "ok"
            assert not result["drift_detected"]


def test_check_drift_detected():
    with tempfile.TemporaryDirectory() as tmpdir:
        drift_dir = os.path.join(tmpdir, "drift")
        os.makedirs(drift_dir, exist_ok=True)
        with open(os.path.join(drift_dir, "test.jsonl"), "w") as f:
            for i in range(20):
                f.write(json.dumps({"mean": 0.1}) + "\n")
            for i in range(20):
                f.write(json.dumps({"mean": 0.9}) + "\n")
        with patch("app.services.drift_monitor.settings") as mock_settings:
            mock_settings.model_dir = tmpdir
            result = check_drift("test", threshold=1.0)
            assert result["drift_detected"]
            assert result["status"] == "drift_detected"


@pytest.mark.asyncio
async def test_verify_api_key_no_key_configured():
    from fastapi import Request
    with patch("app.auth.settings") as mock_settings:
        mock_settings.api_key = ""
        result = await verify_api_key(MagicMock(), None)
        assert result is None


@pytest.mark.asyncio
async def test_verify_api_key_valid():
    from fastapi import Request
    with patch("app.auth.settings") as mock_settings:
        mock_settings.api_key = "test-secret"
        result = await verify_api_key(MagicMock(), "test-secret")
        assert result is None


@pytest.mark.asyncio
async def test_verify_api_key_invalid():
    from fastapi import HTTPException, Request
    with patch("app.auth.settings") as mock_settings:
        mock_settings.api_key = "test-secret"
        with pytest.raises(HTTPException) as exc_info:
            await verify_api_key(MagicMock(), "wrong-key")
        assert exc_info.value.status_code == 401


@pytest.mark.asyncio
async def test_verify_api_key_missing():
    from fastapi import HTTPException, Request
    with patch("app.auth.settings") as mock_settings:
        mock_settings.api_key = "test-secret"
        with pytest.raises(HTTPException) as exc_info:
            await verify_api_key(MagicMock(), None)
        assert exc_info.value.status_code == 401
