"""
Train all ML models via the analytics-ml service API.

Usage:
    ML_URL=http://localhost:8001 python train_all.py
"""
from __future__ import annotations

import json
import os
import sys
import time

import httpx

MODELS = [
    "mastitis",
    "ketosis_warning",
    "estrus",
    "culling",
    "milk_forecast",
    "cow_clusters",
    "equipment_anomaly",
    "multi_task_health",
]


def main() -> None:
    base_url = os.environ.get("ML_URL", "http://localhost:8001")
    api_key = os.environ.get("ML_API_KEY", "")

    headers = {"Content-Type": "application/json"}
    if api_key:
        headers["X-API-Key"] = api_key

    client = httpx.Client(base_url=base_url, headers=headers, timeout=300)

    health = client.get("/health")
    if health.status_code != 200:
        print(f"ML service not healthy: {health.status_code}")
        sys.exit(1)
    print(f"ML service: {health.json()}")

    results = {}
    for model_name in MODELS:
        print(f"\nTraining {model_name}...")
        start = time.time()
        try:
            resp = client.post("/train", json={"model_name": model_name})
            elapsed = time.time() - start
            if resp.status_code == 200:
                data = resp.json()
                results[model_name] = {"status": "ok", "elapsed": round(elapsed, 1), "data": data}
                print(f"  OK ({elapsed:.1f}s) - {json.dumps(data.get('metrics', data.get('model_name', '')), indent=2)}")
            else:
                results[model_name] = {"status": "error", "elapsed": round(elapsed, 1), "code": resp.status_code, "body": resp.text[:200]}
                print(f"  ERROR {resp.status_code}: {resp.text[:200]}")
        except Exception as e:
            elapsed = time.time() - start
            results[model_name] = {"status": "exception", "elapsed": round(elapsed, 1), "error": str(e)[:200]}
            print(f"  EXCEPTION: {e}")

    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    ok = sum(1 for r in results.values() if r["status"] == "ok")
    err = sum(1 for r in results.values() if r["status"] != "ok")
    for name, res in results.items():
        status = res["status"].upper()
        elapsed = res.get("elapsed", 0)
        print(f"  {name:25s} {status:12s} {elapsed:6.1f}s")
    print(f"\n  Total: {ok} ok, {err} errors out of {len(MODELS)} models")


if __name__ == "__main__":
    main()
