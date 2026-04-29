import pytest
from fastapi.testclient import TestClient
from unittest.mock import patch, AsyncMock


@pytest.fixture
def client():
    with patch("app.main.async_session") as mock_session:
        mock_session_instance = AsyncMock()
        mock_session.return_value.__aenter__ = AsyncMock(return_value=mock_session_instance)
        mock_session.return_value.__aexit__ = AsyncMock(return_value=None)
        from app.main import app
        with TestClient(app) as c:
            yield c


def test_health_endpoint(client):
    with patch("app.main.check_connection", new_callable=AsyncMock, return_value=True):
        resp = client.get("/health")
        assert resp.status_code == 200
        data = resp.json()
        assert data["status"] == "ok"
        assert "models" in data


def test_health_endpoint_degraded(client):
    with patch("app.main.check_connection", new_callable=AsyncMock, return_value=False):
        resp = client.get("/health")
        assert resp.status_code == 200
        data = resp.json()
        assert data["status"] == "degraded"


def test_api_key_required(client):
    with patch("app.auth.settings") as mock_settings:
        mock_settings.api_key = "secret-key"
        with patch("app.main.check_connection", new_callable=AsyncMock, return_value=True):
            resp = client.get("/health")
            assert resp.status_code == 401


def test_api_key_valid(client):
    with patch("app.auth.settings") as mock_settings:
        mock_settings.api_key = "secret-key"
        with patch("app.main.check_connection", new_callable=AsyncMock, return_value=True):
            resp = client.get("/health", headers={"X-API-Key": "secret-key"})
            assert resp.status_code == 200


def test_api_key_invalid(client):
    with patch("app.auth.settings") as mock_settings:
        mock_settings.api_key = "secret-key"
        with patch("app.main.check_connection", new_callable=AsyncMock, return_value=True):
            resp = client.get("/health", headers={"X-API-Key": "wrong-key"})
            assert resp.status_code == 401
