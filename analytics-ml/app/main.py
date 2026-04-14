from contextlib import asynccontextmanager
from datetime import datetime
from logging import getLogger

import joblib
from apscheduler.schedulers.asyncio import AsyncIOScheduler
from apscheduler.triggers.cron import CronTrigger
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
    async_session,
    check_connection,
    get_session,
    load_culling_features,
    load_mastitis_features,
)

logger = getLogger(__name__)

_model_timestamps: dict[str, str | None] = {}
_scheduler: AsyncIOScheduler | None = None


def _check_model(name: str) -> str | None:
    import os

    filename = f"{name}_xgb.pkl"
    path = os.path.join(settings.model_dir, filename)
    if os.path.exists(path):
        mtime = os.path.getmtime(path)
        return datetime.fromtimestamp(mtime).isoformat()
    return None


async def _scheduled_retrain():
    logger.info("Scheduled retraining started")
    try:
        async with async_session() as session:
            for name in ("mastitis", "culling"):
                try:
                    result = await _train_model(name, session)
                    _model_timestamps[name] = _check_model(name)
                    logger.info("Retrained %s: %s", name, result)
                except Exception as e:
                    logger.warning("Retrain %s failed: %s", name, e)
    except Exception as e:
        logger.warning("Scheduled retrain DB error: %s", e)


@asynccontextmanager
async def lifespan(app: FastAPI):
    global _scheduler

    _model_timestamps["mastitis"] = _check_model("mastitis")
    _model_timestamps["culling"] = _check_model("culling")

    missing = [m for m in ("mastitis", "culling") if _model_timestamps[m] is None]
    if missing:
        logger.info("Auto-training missing models: %s", missing)
        try:
            async with async_session() as session:
                for name in missing:
                    try:
                        result = await _train_model(name, session)
                        _model_timestamps[name] = _check_model(name)
                        logger.info("Auto-trained %s: %s", name, result)
                    except Exception as e:
                        logger.warning("Auto-train %s failed: %s", name, e)
        except Exception as e:
            logger.warning("Auto-train DB connection failed: %s", e)

    day_map = {
        "mon": "mon", "tue": "tue", "wed": "wed", "thu": "thu",
        "fri": "fri", "sat": "sat", "sun": "sun",
    }
    dow = day_map.get(settings.retrain_day_of_week.lower()[:3], "mon")
    _scheduler = AsyncIOScheduler()
    _scheduler.add_job(
        _scheduled_retrain,
        CronTrigger(day_of_week=dow, hour=settings.retrain_hour, minute=0),
        id="retrain_models",
        replace_existing=True,
    )
    _scheduler.start()
    logger.info(
        "Scheduler started. Retrain: every %s at %02d:00. Models: %s",
        dow, settings.retrain_hour, _model_timestamps,
    )
    yield
    if _scheduler:
        _scheduler.shutdown(wait=False)


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


async def _train_model(name: str, session: AsyncSession) -> dict:
    if name == "mastitis":
        df = await load_mastitis_features(session)
        if df.empty:
            raise ValueError("No data for mastitis training")
        return mastitis_model.train(df)
    elif name == "culling":
        df = await load_culling_features(session)
        if df.empty:
            raise ValueError("No data for culling training")
        return culling_model.train(df)
    else:
        raise ValueError(f"Unknown model: {name}")


@app.post("/train", response_model=TrainResponse)
async def train_model(
    req: TrainRequest,
    session: AsyncSession = Depends(get_session),
):
    try:
        result = await _train_model(req.model_name, session)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database error: {e}")

    _model_timestamps[req.model_name] = _check_model(req.model_name)
    return TrainResponse(**result)
