from contextlib import asynccontextmanager
from datetime import datetime
from logging import getLogger

import joblib
from apscheduler.schedulers.asyncio import AsyncIOScheduler
from apscheduler.triggers.cron import CronTrigger
from fastapi import Depends, FastAPI, HTTPException
from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession

import pandas as pd

from app.config import settings
from app.models import clustering as clustering_model
from app.schemas import DriftStatusEntry, DriftStatusResponse
from app.models import culling as culling_model
from app.models import equipment_anomaly as equipment_model
from app.models import estrus as estrus_model
from app.models import feed_recommendation as feed_rec_model
from app.models import ketosis_warning as ketosis_model
from app.models import mastitis as mastitis_model
from app.models import milk_forecast as forecast_model
from app.schemas import (
    ClusterEntry,
    ClusterRequest,
    ClusterResponse,
    CullingRiskPrediction,
    CullingRiskRequest,
    CullingRiskResponse,
    DriftStatusEntry,
    DriftStatusResponse,
    EquipmentAnomalyEntry,
    EquipmentAnomalyRequest,
    EquipmentAnomalyResponse,
    EstrusPrediction,
    EstrusRequest,
    EstrusResponse,
    FeedRecommendationEntry,
    FeedRecommendationRequest,
    FeedRecommendationResponse,
    HealthReport,
    KetosisWarningEntry,
    KetosisWarningRequest,
    KetosisWarningResponse,
    MastitisRiskPrediction,
    MastitisRiskRequest,
    MastitisRiskResponse,
    MilkForecastDay,
    MilkForecastRequest,
    MilkForecastResponse,
    TrainRequest,
    TrainResponse,
)
from app.services.data_loader import (
    async_session,
    check_connection,
    get_session,
    load_clustering_features,
    load_culling_features,
    load_equipment_anomaly_features,
    load_estrus_features,
    load_feed_recommendation_features,
    load_ketosis_features,
    load_mastitis_features,
    load_milk_timeseries,
)
from app.services.drift_monitor import check_drift, record_predictions

logger = getLogger(__name__)

_model_timestamps: dict[str, str | None] = {}
_scheduler: AsyncIOScheduler | None = None

MODEL_FILES = {
    "mastitis": "mastitis_xgb.pkl",
    "culling": "culling_xgb.pkl",
    "milk_forecast": "milk_forecast_xgb.pkl",
    "cow_clusters": "cow_clusters.pkl",
    "estrus": "estrus_xgb.pkl",
    "equipment_anomaly": "equipment_anomaly.pkl",
    "feed_recommendation": "feed_recommendation_xgb.pkl",
    "ketosis_warning": "ketosis_warning_xgb.pkl",
}


def _check_model(name: str) -> str | None:
    import os

    filename = MODEL_FILES.get(name, f"{name}_xgb.pkl")
    path = os.path.join(settings.model_dir, filename)
    if os.path.exists(path):
        mtime = os.path.getmtime(path)
        return datetime.fromtimestamp(mtime).isoformat()
    return None


async def _scheduled_retrain():
    logger.info("Scheduled retraining started")
    try:
        async with async_session() as session:
            for name in ("mastitis", "culling", "cow_clusters", "milk_forecast", "estrus", "ketosis_warning", "feed_recommendation", "equipment_anomaly"):
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

    for name in MODEL_FILES:
        _model_timestamps[name] = _check_model(name)

    missing = [m for m in ("mastitis", "culling", "milk_forecast", "cow_clusters", "estrus", "ketosis_warning", "feed_recommendation", "equipment_anomaly") if _model_timestamps.get(m) is None]
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


app = FastAPI(title="Milk Farm Analytics ML", version="1.1.0", lifespan=lifespan)


@app.get("/health")
async def health():
    db_ok = await check_connection()
    return HealthReport(
        status="ok" if db_ok else "degraded",
        model_dir=settings.model_dir,
        models=_model_timestamps.copy(),
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
    record_predictions("mastitis", results)
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
    record_predictions("culling", results)
    return CullingRiskResponse(predictions=predictions)


@app.post("/predict/milk-forecast", response_model=MilkForecastResponse)
async def predict_milk_forecast(
    req: MilkForecastRequest,
    session: AsyncSession = Depends(get_session),
):
    try:
        df = await load_milk_timeseries(session, req.animal_id, days=365)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database error: {e}")

    if df.empty:
        return MilkForecastResponse(
            animal_id=req.animal_id,
            animal_name=None,
            current_daily_avg=None,
            forecast=[],
            model_version="no-data",
        )

    animal_name = str(df.iloc[0].get("animal_name", "")) or None

    try:
        result = forecast_model.predict(df, req.days)
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))

    return MilkForecastResponse(
        animal_id=req.animal_id,
        animal_name=animal_name,
        current_daily_avg=result["current_daily_avg"],
        forecast=[MilkForecastDay(**d) for d in result["forecast"]],
        model_version=result["model_version"],
    )


@app.post("/predict/clusters", response_model=ClusterResponse)
async def predict_clusters(
    req: ClusterRequest,
    session: AsyncSession = Depends(get_session),
):
    try:
        df = await load_clustering_features(session, days=req.days)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database error: {e}")

    if df.empty:
        return ClusterResponse(clusters=[], cluster_names={})

    try:
        results = clustering_model.predict(df)
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))

    cluster_names = {str(r["cluster_id"]): r["cluster_name"] for r in results}
    return ClusterResponse(
        clusters=[ClusterEntry(**r) for r in results],
        cluster_names=cluster_names,
    )


@app.post("/predict/estrus", response_model=EstrusResponse)
async def predict_estrus(
    req: EstrusRequest,
    session: AsyncSession = Depends(get_session),
):
    try:
        df = await load_estrus_features(session)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database error: {e}")

    if df.empty:
        return EstrusResponse(predictions=[])

    if req.animal_id is not None:
        df = df[df["animal_id"] == req.animal_id]
        if df.empty:
            return EstrusResponse(predictions=[])

    try:
        results = estrus_model.predict(df)
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))

    estrus_results = [EstrusPrediction(**r) for r in results]
    record_predictions("estrus", results)
    return EstrusResponse(predictions=estrus_results)


@app.post("/predict/equipment-anomaly", response_model=EquipmentAnomalyResponse)
async def predict_equipment(
    req: EquipmentAnomalyRequest,
    session: AsyncSession = Depends(get_session),
):
    try:
        df = await load_equipment_anomaly_features(session)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database error: {e}")

    if df.empty:
        return EquipmentAnomalyResponse(entries=[])

    try:
        results = equipment_model.predict(df)
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))

    equip_results = [EquipmentAnomalyEntry(**r) for r in results]
    record_predictions("equipment_anomaly", results)
    return EquipmentAnomalyResponse(entries=equip_results)


@app.post("/predict/feed-recommendation", response_model=FeedRecommendationResponse)
async def predict_feed(
    req: FeedRecommendationRequest,
    session: AsyncSession = Depends(get_session),
):
    try:
        df = await load_feed_recommendation_features(session)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database error: {e}")

    if df.empty:
        return FeedRecommendationResponse(recommendations=[])

    if req.animal_id is not None:
        df = df[df["animal_id"] == req.animal_id]
        if df.empty:
            return FeedRecommendationResponse(recommendations=[])

    try:
        results = feed_rec_model.predict(df)
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))

    feed_results = [FeedRecommendationEntry(**r) for r in results]
    record_predictions("feed_recommendation", results)
    return FeedRecommendationResponse(recommendations=feed_results)


@app.post("/predict/ketosis-warning", response_model=KetosisWarningResponse)
async def predict_ketosis(
    req: KetosisWarningRequest,
    session: AsyncSession = Depends(get_session),
):
    try:
        df = await load_ketosis_features(session)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database error: {e}")

    if df.empty:
        return KetosisWarningResponse(predictions=[])

    if req.animal_id is not None:
        df = df[df["animal_id"] == req.animal_id]
        if df.empty:
            return KetosisWarningResponse(predictions=[])

    try:
        results = ketosis_model.predict(df)
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))

    ketosis_results = [KetosisWarningEntry(**r) for r in results]
    record_predictions("ketosis_warning", results)
    return KetosisWarningResponse(predictions=ketosis_results)


@app.get("/drift-status", response_model=DriftStatusResponse)
async def drift_status():
    entries = []
    for name in MODEL_FILES:
        info = check_drift(name)
        entries.append(DriftStatusEntry(
            model=info["model"],
            status=info["status"],
            drift_detected=info["drift_detected"],
            z_score=info.get("z_score"),
            recent_mean=info.get("recent_mean"),
            baseline_mean=info.get("baseline_mean"),
            samples=info.get("samples"),
        ))
    return DriftStatusResponse(models=entries)


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
    elif name == "cow_clusters":
        df = await load_clustering_features(session)
        if df.empty:
            raise ValueError("No data for clustering training")
        return clustering_model.train(df)
    elif name == "milk_forecast":
        animals = await session.execute(
            text("SELECT DISTINCT animal_id FROM milk_day_productions WHERE date >= CURRENT_DATE - INTERVAL '365 days' LIMIT 50")
        )
        animal_ids = [r[0] for r in animals.fetchall()]
        if not animal_ids:
            raise ValueError("No animals with milk data for forecast training")
        frames = []
        for aid in animal_ids:
            df = await load_milk_timeseries(session, aid, days=365)
            if not df.empty and len(df) >= 14:
                frames.append(df)
        if not frames:
            raise ValueError("Not enough timeseries data for forecast training")
        combined = pd.concat(frames, ignore_index=True)
        return forecast_model.train(combined)
    elif name == "estrus":
        df = await load_estrus_features(session)
        if df.empty:
            raise ValueError("No data for estrus training")
        return estrus_model.train(df)
    elif name == "ketosis_warning":
        df = await load_ketosis_features(session)
        if df.empty:
            raise ValueError("No data for ketosis training")
        return ketosis_model.train(df)
    elif name == "feed_recommendation":
        df = await load_feed_recommendation_features(session)
        if df.empty:
            raise ValueError("No data for feed recommendation training")
        return feed_rec_model.train(df)
    elif name == "equipment_anomaly":
        df = await load_equipment_anomaly_features(session)
        if df.empty:
            raise ValueError("No data for equipment anomaly training")
        return equipment_model.train(df)
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
