import os
import tempfile
from unittest.mock import patch

import numpy as np
import pandas as pd
import pytest


def _make_mastitis_df(n=20):
    rng = np.random.RandomState(42)
    return pd.DataFrame({
        "animal_id": range(1, n + 1),
        "animal_name": [f"Cow{i}" for i in range(1, n + 1)],
        "age_years": rng.uniform(2, 8, n),
        "recent_scc": rng.uniform(50000, 400000, n),
        "scc_trend_ratio": rng.uniform(0.5, 3.0, n),
        "avg_conductivity": rng.uniform(30, 70, n),
        "milk_deviation": rng.uniform(-0.3, 0.2, n),
        "dim_days": rng.randint(10, 300, n),
        "avg_rumination_7d": rng.uniform(300, 600, n),
        "avg_activity_7d": rng.uniform(50, 200, n),
        "fat_protein_ratio": rng.uniform(0.8, 1.8, n),
        "cond_asymmetry": rng.uniform(0, 10, n),
    })


def test_mastitis_train_and_predict():
    with tempfile.TemporaryDirectory() as tmpdir:
        with patch("app.models.mastitis.settings") as mock_settings:
            mock_settings.model_dir = tmpdir
            from app.models import mastitis

            df = _make_mastitis_df()
            result = mastitis.train(df)
            assert result["model_name"] == "mastitis"
            assert result["samples"] == 20
            assert "metrics" in result

            model_path = os.path.join(tmpdir, "mastitis_xgb.pkl")
            assert os.path.exists(model_path)

            predictions = mastitis.predict(df)
            assert len(predictions) == 20
            for p in predictions:
                assert "animal_id" in p
                assert "risk_probability" in p
                assert "risk_level" in p
                assert 0 <= p["risk_probability"] <= 1
                assert p["risk_level"] in ("low", "medium", "high")


def test_mastitis_predict_missing_model():
    with tempfile.TemporaryDirectory() as tmpdir:
        with patch("app.models.mastitis.settings") as mock_settings:
            mock_settings.model_dir = tmpdir
            from app.models import mastitis

            df = _make_mastitis_df(5)
            with pytest.raises(FileNotFoundError):
                mastitis.predict(df)


def test_mastitis_predict_with_filter():
    with tempfile.TemporaryDirectory() as tmpdir:
        with patch("app.models.mastitis.settings") as mock_settings:
            mock_settings.model_dir = tmpdir
            from app.models import mastitis

            df = _make_mastitis_df(10)
            mastitis.train(df)

            filtered = df[df["animal_id"] == 1]
            predictions = mastitis.predict(filtered)
            assert len(predictions) == 1
            assert predictions[0]["animal_id"] == 1
