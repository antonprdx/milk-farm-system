import pandas as pd
from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker

from app.config import settings

engine = create_async_engine(settings.database_url, pool_size=5, max_overflow=10)
async_session = sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)


async def get_session() -> AsyncSession:
    async with async_session() as session:
        yield session


async def check_connection() -> bool:
    try:
        async with async_session() as session:
            await session.execute(text("SELECT 1"))
        return True
    except Exception:
        return False


async def load_mastitis_features(session: AsyncSession) -> pd.DataFrame:
    query = text("""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            EXTRACT(YEAR FROM AGE(CURRENT_DATE, a.birth_date))::float8 as age_years,
            COALESCE(scc_latest.scc, 0) as recent_scc,
            COALESCE(scc_trend.ratio, 1) as scc_trend_ratio,
            COALESCE(cond.avg_cond, 0) as avg_conductivity,
            COALESCE(milk_dev.dev, 0) as milk_deviation,
            COALESCE(dim.days, 0) as dim_days,
            COALESCE(rum_7d.rum, 0) as avg_rumination_7d,
            COALESCE(act_7d.act, 0) as avg_activity_7d
        FROM animals a
        LEFT JOIN LATERAL (
            SELECT AVG(q.scc)::float8 as scc FROM milk_quality q
            WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
        ) scc_latest ON true
        LEFT JOIN LATERAL (
            SELECT (recent.scc / NULLIF(baseline.scc, 0))::float8 as ratio
            FROM (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days') recent,
                 (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days' AND q.date < CURRENT_DATE - INTERVAL '7 days') baseline
        ) scc_trend ON true
        LEFT JOIN LATERAL (
            SELECT AVG((v.lf_conductivity + v.lr_conductivity + v.rf_conductivity + v.rr_conductivity)::float8 / 4.0) as avg_cond
            FROM milk_visit_quality v WHERE v.animal_id = a.id AND v.visit_datetime >= CURRENT_DATE - INTERVAL '7 days'
        ) cond ON true
        LEFT JOIN LATERAL (
            SELECT (recent.milk / NULLIF(baseline.milk, 0) - 1)::float8 as dev
            FROM (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days') recent,
                 (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days' AND m.date < CURRENT_DATE - INTERVAL '7 days') baseline
        ) milk_dev ON true
        LEFT JOIN LATERAL (
            SELECT (CURRENT_DATE - c.calving_date)::int8 as days
            FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
        ) dim ON true
        LEFT JOIN LATERAL (
            SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r
            WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '7 days'
        ) rum_7d ON true
        LEFT JOIN LATERAL (
            SELECT AVG(act.activity_counter)::float8 as act FROM activities act
            WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - INTERVAL '7 days'
        ) act_7d ON true
        WHERE a.active = true AND a.gender = 'female'
    """)
    result = await session.execute(query)
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_culling_features(session: AsyncSession) -> pd.DataFrame:
    query = text("""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            EXTRACT(YEAR FROM AGE(CURRENT_DATE, a.birth_date))::float8 as age_years,
            COALESCE(latest_milk.milk, 0) as avg_milk_30d,
            COALESCE(avg_scc.scc, 0) as avg_scc_90d,
            COALESCE(ci.interval, 0) as calving_interval,
            COALESCE(lac_count.lacs, 0) as lactation_count,
            COALESCE(rum_30d.rum, 0) as avg_rumination_30d,
            COALESCE(milk_7d.milk, 0) as avg_milk_7d,
            COALESCE(act_30d.act, 0) as avg_activity_30d,
            COALESCE(dim.days, 0) as current_dim
        FROM animals a
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days') latest_milk ON true
        LEFT JOIN LATERAL (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days') avg_scc ON true
        LEFT JOIN LATERAL (
            SELECT AVG((c2.calving_date - c1.calving_date))::float8 as interval
            FROM calvings c1 JOIN calvings c2 ON c1.animal_id = c2.animal_id AND c2.calving_date > c1.calving_date
            WHERE c1.animal_id = a.id
            AND NOT EXISTS (SELECT 1 FROM calvings c3 WHERE c3.animal_id = c1.animal_id AND c3.calving_date > c1.calving_date AND c3.calving_date < c2.calving_date)
        ) ci ON true
        LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as lacs FROM calvings c WHERE c.animal_id = a.id) lac_count ON true
        LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '30 days') rum_30d ON true
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days') milk_7d ON true
        LEFT JOIN LATERAL (SELECT AVG(act.activity_counter)::float8 as act FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - INTERVAL '30 days') act_30d ON true
        LEFT JOIN LATERAL (SELECT (CURRENT_DATE - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
        WHERE a.active = true AND a.gender = 'female'
    """)
    result = await session.execute(query)
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_milk_timeseries(session: AsyncSession, animal_id: int, days: int = 365) -> pd.DataFrame:
    query = text("""
        SELECT
            m.date,
            m.milk_amount,
            COALESCE(f.total, 0) as feed_amount,
            COALESCE(r.rumination_minutes, 0) as rumination_minutes,
            COALESCE(act.activity_counter, 0) as activity_counter
        FROM milk_day_productions m
        LEFT JOIN feed_day_amounts f ON f.animal_id = m.animal_id AND f.feed_date = m.date
        LEFT JOIN ruminations r ON r.animal_id = m.animal_id AND r.date = m.date
        LEFT JOIN LATERAL (
            SELECT AVG(activity_counter)::float8 as activity_counter
            FROM activities WHERE animal_id = m.animal_id AND activity_datetime::date = m.date
        ) act ON true
        WHERE m.animal_id = :animal_id AND m.date >= CURRENT_DATE - (:days || ' days')::interval
        ORDER BY m.date
    """)
    result = await session.execute(query, {"animal_id": animal_id, "days": days})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)
