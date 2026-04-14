from contextlib import asynccontextmanager
from datetime import datetime
from logging import getLogger

import joblib
from fastapi import Depends, FastAPI, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession

from app.config import settings
from app.models import culling as culling_model
from app.models import mastitis as mastitis_model
from app.schemas import (
    CullingRiskPrediction,
    CullingRiskRequest,
    CullingRiskResponse,
    HealthReport,
    MastitisRiskPrediction,
    MastitisRiskRequest,
    MastitisRiskResponse,
    TrainRequest,
    TrainResponse,
)
from app.services.data_loader import (
    check_connection,
    get_session,
    load_culling_features,
    load_mastitis_features,
)

logger = getLogger(__name__)

_model_timestamps: dict[str, str | None] = {}


def _check_model(name: str) -> str | None:
    import os

    filename = f"{name}_xgb.pkl"
    path = os.path.join(settings.model_dir, filename)
    if os.path.exists(path):
        mtime = os.path.getmtime(path)
        return datetime.fromtimestamp(mtime).isoformat()
    return None


@asynccontextmanager
async def lifespan(app: FastAPI):
    _model_timestamps["mastitis"] = _check_model("mastitis")
    _model_timestamps["culling"] = _check_model("culling")
    logger.info("Analytics ML service started. Model timestamps: %s", _model_timestamps)
    yield


app = FastAPI(title="Milk Farm Analytics ML", version="1.0.0", lifespan=lifespan)


@app.get("/health")
async def health():
    db_ok = await check_connection()
    return HealthReport(
        status="ok" if db_ok else "degraded",
        model_dir=settings.model_dir,
        models={
            "mastitis": _model_timestamps.get("mastitis"),
            "culling": _model_timestamps.get("culling"),
        },
        database_connected=db_ok,
    )


@app.post("/predict/mastitis", response_model=MastitisRiskResponse)
async def predict_mastitis(
    req: MastitisRiskRequest,
    session: AsyncSession = Depends(get_session),
):
    try:
        df = await load_mastitis_features(session)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database error: {e}")

    if df.empty:
        return MastitisRiskResponse(predictions=[])

    if req.animal_id is not None:
        df = df[df["animal_id"] == req.animal_id]
        if df.empty:
            return MastitisRiskResponse(predictions=[])

    try:
        results = mastitis_model.predict(df)
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))

    predictions = [MastitisRiskPrediction(**r) for r in results]
    return MastitisRiskResponse(predictions=predictions)


@app.post("/predict/culling", response_model=CullingRiskResponse)
async def predict_culling(
    req: CullingRiskRequest,
    session: AsyncSession = Depends(get_session),
):
    try:
        df = await load_culling_features(session)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database error: {e}")

    if df.empty:
        return CullingRiskResponse(predictions=[])

    if req.animal_id is not None:
        df = df[df["animal_id"] == req.animal_id]
        if df.empty:
            return CullingRiskResponse(predictions=[])

    try:
        results = culling_model.predict(df)
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))

    predictions = [CullingRiskPrediction(**r) for r in results]
    return CullingRiskResponse(predictions=predictions)


@app.post("/train", response_model=TrainResponse)
async def train_model(
    req: TrainRequest,
    session: AsyncSession = Depends(get_session),
):
    if req.model_name == "mastitis":
        try:
            df = await load_mastitis_features(session)
        except Exception as e:
            raise HTTPException(status_code=503, detail=f"Database error: {e}")
        if df.empty:
            raise HTTPException(status_code=422, detail="No data available for training")
        result = mastitis_model.train(df)
        _model_timestamps["mastitis"] = _check_model("mastitis")
    elif req.model_name == "culling":
        try:
            df = await load_culling_features(session)
        except Exception as e:
            raise HTTPException(status_code=503, detail=f"Database error: {e}")
        if df.empty:
            raise HTTPException(status_code=422, detail="No data available for training")
        result = culling_model.train(df)
        _model_timestamps["culling"] = _check_model("culling")
    else:
        raise HTTPException(status_code=400, detail=f"Unknown model: {req.model_name}")

    return TrainResponse(**result)
