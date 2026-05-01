import logging

import pandas as pd
from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker

from app.config import settings

logger = logging.getLogger(__name__)

engine = create_async_engine(settings.database_url, pool_size=10, max_overflow=20)
async_session = sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)

_REF_DATE = "get_ref_date()"

_WEATHER_SQL = """
    COALESCE(w.temp_c, 0) as weather_temp,
    COALESCE(w.humidity, 0) as weather_humidity,
    COALESCE(
        CASE WHEN w.temp_c IS NOT NULL AND w.humidity IS NOT NULL
            THEN (1.8 * w.temp_c + 32) - (0.55 - 0.0055 * w.humidity) * ((1.8 * w.temp_c + 32) - 58)
            ELSE 0 END, 0
    )::float8 as thi
"""

_VET_MASTITIS_SQL = """
    COALESCE(vmast.cnt, 0) as mastitis_treatments_90d,
    COALESCE(vmast.days_since, 999) as days_since_mastitis_tx
"""

_VET_HISTORY_SQL = """
    COALESCE(vh.tx_count, 0) as vet_tx_count_180d,
    COALESCE(vh.days_since_any, 999) as days_since_any_tx
"""

_GENETICS_SQL = """
    COALESCE(breed.hol_pct, 0) as holstein_percentage
"""

_LACTOSE_SQL = """
    COALESCE(lac_avg.avg_lactose, 0) as avg_lactose_7d,
    COALESCE(lac_tr.lac_trend, 0) as lactose_trend
"""

_COMMON_JOINS_WEATHER = f"""
    LEFT JOIN LATERAL (
        SELECT temp_c, humidity FROM weather_cache
        WHERE date = {_REF_DATE} - INTERVAL '1 day' LIMIT 1
    ) w ON true
"""

_COMMON_JOINS_VET_MASTITIS = f"""
    LEFT JOIN LATERAL (
        SELECT COUNT(*)::int8 as cnt,
               ({_REF_DATE} - MAX(event_date))::int8 as days_since
        FROM vet_records WHERE animal_id = a.id
        AND record_type = 'treatment' AND confirmed = true
        AND event_date >= {_REF_DATE} - INTERVAL '90 days'
    ) vmast ON true
"""

_COMMON_JOINS_VET_HISTORY = f"""
    LEFT JOIN LATERAL (
        SELECT COUNT(*)::int8 as tx_count,
               ({_REF_DATE} - MAX(event_date))::int8 as days_since_any
        FROM vet_records WHERE animal_id = a.id
        AND record_type IN ('treatment','disease','surgery')
        AND event_date >= {_REF_DATE} - INTERVAL '180 days'
    ) vh ON true
"""

_COMMON_JOINS_GENETICS = """
    LEFT JOIN LATERAL (
        SELECT COALESCE(SUM(CASE WHEN blood_type_code = 'HOL' THEN percentage ELSE 0 END), 0)::float8 as hol_pct
        FROM bloodlines WHERE animal_id = a.id
    ) breed ON true
"""

_COMMON_JOINS_LACTOSE = f"""
    LEFT JOIN LATERAL (
        SELECT AVG(q.lactose_percentage)::float8 as avg_lactose FROM milk_quality q
        WHERE q.animal_id = a.id AND q.date >= {_REF_DATE} - INTERVAL '7 days'
    ) lac_avg ON true
    LEFT JOIN LATERAL (
        SELECT (recent.lac / NULLIF(baseline.lac, 0) - 1)::float8 as lac_trend
        FROM (SELECT AVG(lactose_percentage)::float8 as lac FROM milk_quality WHERE animal_id = a.id AND date >= {_REF_DATE} - INTERVAL '7 days') recent,
             (SELECT AVG(lactose_percentage)::float8 as lac FROM milk_quality WHERE animal_id = a.id AND date >= {_REF_DATE} - INTERVAL '14 days' AND date < {_REF_DATE} - INTERVAL '7 days') baseline
    ) lac_tr ON true
"""


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


async def load_culling_events(session: AsyncSession) -> pd.DataFrame:
    query = text("""
        SELECT ce.animal_id, ce.culling_date as event_date, ce.reason, ce.details
        FROM culling_events ce
        ORDER BY ce.culling_date DESC
    """)
    result = await session.execute(query)
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_confirmed_vet_labels(session: AsyncSession, record_type: str) -> pd.DataFrame:
    query = text("""
        SELECT animal_id, event_date, diagnosis_code, confirmed
        FROM vet_records
        WHERE confirmed = true AND record_type = :rtype
        ORDER BY event_date DESC
    """)
    result = await session.execute(query, {"rtype": record_type})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_confirmed_heats(session: AsyncSession) -> pd.DataFrame:
    query = text("""
        SELECT animal_id, heat_date, confirmed, confirmation_method
        FROM heats WHERE confirmed = true
        ORDER BY heat_date DESC
    """)
    result = await session.execute(query)
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_mastitis_features(session: AsyncSession, animal_id: int | None = None) -> pd.DataFrame:
    query = text(f"""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            EXTRACT(YEAR FROM AGE({_REF_DATE}, a.birth_date))::float8 as age_years,
            COALESCE(scc_latest.scc, 0) as recent_scc,
            COALESCE(scc_trend.ratio, 1) as scc_trend_ratio,
            COALESCE(cond.avg_cond, 0) as avg_conductivity,
            COALESCE(milk_dev.dev, 0) as milk_deviation,
            COALESCE(dim.days, 0) as dim_days,
            COALESCE(rum_7d.rum, 0) as avg_rumination_7d,
            COALESCE(act_7d.act, 0) as avg_activity_7d,
            COALESCE(fpr.ratio, 0) as fat_protein_ratio,
            COALESCE(cond_asym.asym, 0) as cond_asymmetry,
            {_LACTOSE_SQL},
            {_WEATHER_SQL},
            {_VET_MASTITIS_SQL},
            {_VET_HISTORY_SQL},
            {_GENETICS_SQL}
        FROM animals a
        LEFT JOIN LATERAL (
            SELECT AVG(q.scc)::float8 as scc FROM milk_quality q
            WHERE q.animal_id = a.id AND q.date >= {_REF_DATE} - INTERVAL '7 days'
        ) scc_latest ON true
        LEFT JOIN LATERAL (
            SELECT (recent.scc / NULLIF(baseline.scc, 0))::float8 as ratio
            FROM (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= {_REF_DATE} - INTERVAL '7 days') recent,
                 (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= {_REF_DATE} - INTERVAL '90 days' AND q.date < {_REF_DATE} - INTERVAL '7 days') baseline
        ) scc_trend ON true
        LEFT JOIN LATERAL (
            SELECT AVG((v.lf_conductivity + v.lr_conductivity + v.rf_conductivity + v.rr_conductivity)::float8 / 4.0) as avg_cond
            FROM milk_visit_quality v WHERE v.animal_id = a.id AND v.visit_datetime >= {_REF_DATE} - INTERVAL '7 days'
        ) cond ON true
        LEFT JOIN LATERAL (
            SELECT (recent.milk / NULLIF(baseline.milk, 0) - 1)::float8 as dev
            FROM (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '7 days') recent,
                 (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '30 days' AND m.date < {_REF_DATE} - INTERVAL '7 days') baseline
        ) milk_dev ON true
        LEFT JOIN LATERAL (
            SELECT ({_REF_DATE} - c.calving_date)::int8 as days
            FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
        ) dim ON true
        LEFT JOIN LATERAL (
            SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r
            WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - INTERVAL '7 days'
        ) rum_7d ON true
        LEFT JOIN LATERAL (
            SELECT AVG(act.activity_counter)::float8 as act FROM activities act
            WHERE act.animal_id = a.id AND act.activity_datetime >= {_REF_DATE} - INTERVAL '7 days'
        ) act_7d ON true
        LEFT JOIN LATERAL (
            SELECT (AVG(q.fat_percentage) / NULLIF(AVG(q.protein_percentage), 0))::float8 as ratio
            FROM milk_quality q
            WHERE q.animal_id = a.id AND q.date >= {_REF_DATE} - INTERVAL '7 days'
        ) fpr ON true
        LEFT JOIN LATERAL (
            SELECT GREATEST(
                ABS(sub.lf - sub.avg4),
                ABS(sub.lr - sub.avg4),
                ABS(sub.rf - sub.avg4),
                ABS(sub.rr - sub.avg4)
            )::float8 as asym
            FROM (
                SELECT
                    AVG(v.lf_conductivity)::float8 as lf,
                    AVG(v.lr_conductivity)::float8 as lr,
                    AVG(v.rf_conductivity)::float8 as rf,
                    AVG(v.rr_conductivity)::float8 as rr,
                    (AVG(v.lf_conductivity) + AVG(v.lr_conductivity) + AVG(v.rf_conductivity) + AVG(v.rr_conductivity))::float8 / 4.0 as avg4
                FROM milk_visit_quality v
                WHERE v.animal_id = a.id AND v.visit_datetime >= {_REF_DATE} - INTERVAL '7 days'
            ) sub
        ) cond_asym ON true
        {_COMMON_JOINS_LACTOSE}
        {_COMMON_JOINS_WEATHER}
        {_COMMON_JOINS_VET_MASTITIS}
        {_COMMON_JOINS_VET_HISTORY}
        {_COMMON_JOINS_GENETICS}
        WHERE a.active = true AND a.gender = 'female'
        {(" AND a.id = :aid" if animal_id is not None else "")}
    """)
    result = await session.execute(query, {"aid": animal_id} if animal_id is not None else {})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_culling_features(session: AsyncSession, animal_id: int | None = None) -> pd.DataFrame:
    query = text(f"""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            EXTRACT(YEAR FROM AGE({_REF_DATE}, a.birth_date))::float8 as age_years,
            COALESCE(latest_milk.milk, 0) as avg_milk_30d,
            COALESCE(avg_scc.scc, 0) as avg_scc_90d,
            COALESCE(ci.interval, 0) as calving_interval,
            COALESCE(lac_count.lacs, 0) as lactation_count,
            COALESCE(rum_30d.rum, 0) as avg_rumination_30d,
            COALESCE(milk_7d.milk, 0) as avg_milk_7d,
            COALESCE(act_30d.act, 0) as avg_activity_30d,
            COALESCE(dim.days, 0) as current_dim,
            {_WEATHER_SQL},
            {_VET_HISTORY_SQL},
            {_GENETICS_SQL},
            COALESCE(ce.was_culled, false) as was_culled,
            COALESCE(ce.days_to_culling, -1) as days_to_culling
        FROM animals a
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '30 days') latest_milk ON true
        LEFT JOIN LATERAL (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= {_REF_DATE} - INTERVAL '90 days') avg_scc ON true
        LEFT JOIN LATERAL (
            SELECT AVG((c2.calving_date - c1.calving_date))::float8 as interval
            FROM calvings c1 JOIN calvings c2 ON c1.animal_id = c2.animal_id AND c2.calving_date > c1.calving_date
            WHERE c1.animal_id = a.id
            AND NOT EXISTS (SELECT 1 FROM calvings c3 WHERE c3.animal_id = c1.animal_id AND c3.calving_date > c1.calving_date AND c3.calving_date < c2.calving_date)
        ) ci ON true
        LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as lacs FROM calvings c WHERE c.animal_id = a.id) lac_count ON true
        LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - INTERVAL '30 days') rum_30d ON true
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '7 days') milk_7d ON true
        LEFT JOIN LATERAL (SELECT AVG(act.activity_counter)::float8 as act FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= {_REF_DATE} - INTERVAL '30 days') act_30d ON true
        LEFT JOIN LATERAL (SELECT ({_REF_DATE} - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
        {_COMMON_JOINS_WEATHER}
        {_COMMON_JOINS_VET_HISTORY}
        {_COMMON_JOINS_GENETICS}
        LEFT JOIN LATERAL (
            SELECT true as was_culled,
                   (ce.culling_date - {_REF_DATE})::int8 as days_to_culling
            FROM culling_events ce WHERE ce.animal_id = a.id LIMIT 1
        ) ce ON true
        WHERE a.active = true AND a.gender = 'female'
        {(" AND a.id = :aid" if animal_id is not None else "")}
    """)
    result = await session.execute(query, {"aid": animal_id} if animal_id is not None else {})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_milk_timeseries(session: AsyncSession, animal_id: int, days: int = 365) -> pd.DataFrame:
    query = text(f"""
        SELECT
            a.name as animal_name,
            m.date,
            m.milk_amount,
            COALESCE(f.total, 0) as feed_amount,
            COALESCE(r.rumination_minutes, 0) as rumination_minutes,
            COALESCE(act.activity_counter, 0) as activity_counter,
            COALESCE(w.temp_c, 0) as temp_c,
            COALESCE(w.humidity, 0) as humidity,
            COALESCE(
                CASE WHEN w.temp_c IS NOT NULL AND w.humidity IS NOT NULL
                    THEN (1.8 * w.temp_c + 32) - (0.55 - 0.0055 * w.humidity) * ((1.8 * w.temp_c + 32) - 58)
                    ELSE 0 END, 0
            )::float8 as thi
        FROM milk_day_productions m
        JOIN animals a ON a.id = m.animal_id
        LEFT JOIN feed_day_amounts f ON f.animal_id = m.animal_id AND f.feed_date = m.date
        LEFT JOIN ruminations r ON r.animal_id = m.animal_id AND r.date = m.date
        LEFT JOIN LATERAL (
            SELECT AVG(activity_counter)::float8 as activity_counter
            FROM activities WHERE animal_id = m.animal_id AND activity_datetime::date = m.date
        ) act ON true
        LEFT JOIN LATERAL (
            SELECT temp_c, humidity FROM weather_cache WHERE date = m.date LIMIT 1
        ) w ON true
        WHERE m.animal_id = :animal_id AND m.date >= {_REF_DATE} - make_interval(days => :days)
        ORDER BY m.date
    """)
    result = await session.execute(query, {"animal_id": animal_id, "days": days})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_clustering_features(session: AsyncSession, animal_id: int | None = None, days: int = 90) -> pd.DataFrame:
    query = text(f"""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            m.avg_milk,
            COALESCE(m.std_milk, 0) as std_milk,
            COALESCE(r.rum, 0) as rumination_minutes,
            COALESCE(act.act, 0) as activity_counter,
            COALESCE(f.feed, 0) as feed_amount,
            COALESCE(dim.days, 0) as dim_days,
            COALESCE(lac.n, 0) as lactation_number
        FROM animals a
        JOIN LATERAL (
            SELECT AVG(m.milk_amount)::float8 as avg_milk,
                   STDDEV(m.milk_amount)::float8 as std_milk
            FROM milk_day_productions m
            WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - make_interval(days => :days)
        ) m ON true
        LEFT JOIN LATERAL (
            SELECT AVG(r.rumination_minutes)::float8 as rum
            FROM ruminations r WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - make_interval(days => :days)
        ) r ON true
        LEFT JOIN LATERAL (
            SELECT AVG(act.activity_counter)::float8 as act
            FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= {_REF_DATE} - make_interval(days => :days)
        ) act ON true
        LEFT JOIN LATERAL (
            SELECT SUM(f.total)::float8 / NULLIF(COUNT(DISTINCT f.feed_date)::float8, 0) as feed
            FROM feed_day_amounts f WHERE f.animal_id = a.id AND f.feed_date >= {_REF_DATE} - make_interval(days => :days)
        ) f ON true
        LEFT JOIN LATERAL (
            SELECT ({_REF_DATE} - c.calving_date)::int8 as days
            FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
        ) dim ON true
        LEFT JOIN LATERAL (
            SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id
        ) lac ON true
        WHERE a.active = true AND a.gender = 'female' AND m.avg_milk IS NOT NULL
        {(" AND a.id = :aid" if animal_id is not None else "")}
        ORDER BY a.name
    """)
    result = await session.execute(query, {"days": days} | ({"aid": animal_id} if animal_id is not None else {}))
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    df = pd.DataFrame(rows)
    df["animal_name"] = df["animal_name"].fillna("")
    return df


async def load_estrus_features(session: AsyncSession, animal_id: int | None = None) -> pd.DataFrame:
    query = text(f"""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            COALESCE(act_r.ratio, 1) as activity_ratio_7d,
            COALESCE(rum_r.ratio, 1) as rumination_ratio_7d,
            COALESCE(milk_r.ratio, 1) as milk_ratio_7d,
            COALESCE(dim.days, 0) as dim_days,
            COALESCE(lac.n, 0) as lactation_number,
            COALESCE(dsh.days, 999) as days_since_last_heat,
            COALESCE(act_14.avg_act, 0) as avg_activity_14d,
            COALESCE(rum_14.avg_rum, 0) as avg_rumination_14d,
            {_WEATHER_SQL},
            {_GENETICS_SQL}
        FROM animals a
        LEFT JOIN LATERAL (
            SELECT (recent.act / NULLIF(baseline.act, 0))::float8 as ratio FROM (
                SELECT AVG(activity_counter)::float8 as act FROM activities
                WHERE animal_id = a.id AND activity_datetime >= {_REF_DATE} - INTERVAL '7 days'
            ) recent, (
                SELECT AVG(activity_counter)::float8 as act FROM activities
                WHERE animal_id = a.id AND activity_datetime >= {_REF_DATE} - INTERVAL '30 days'
                AND activity_datetime < {_REF_DATE} - INTERVAL '7 days'
            ) baseline
        ) act_r ON true
        LEFT JOIN LATERAL (
            SELECT (recent.rum / NULLIF(baseline.rum, 0))::float8 as ratio FROM (
                SELECT AVG(rumination_minutes)::float8 as rum FROM ruminations
                WHERE animal_id = a.id AND date >= {_REF_DATE} - INTERVAL '7 days'
            ) recent, (
                SELECT AVG(rumination_minutes)::float8 as rum FROM ruminations
                WHERE animal_id = a.id AND date >= {_REF_DATE} - INTERVAL '30 days'
                AND date < {_REF_DATE} - INTERVAL '7 days'
            ) baseline
        ) rum_r ON true
        LEFT JOIN LATERAL (
            SELECT (recent.milk / NULLIF(baseline.milk, 0))::float8 as ratio FROM (
                SELECT AVG(milk_amount)::float8 as milk FROM milk_day_productions
                WHERE animal_id = a.id AND date >= {_REF_DATE} - INTERVAL '7 days'
            ) recent, (
                SELECT AVG(milk_amount)::float8 as milk FROM milk_day_productions
                WHERE animal_id = a.id AND date >= {_REF_DATE} - INTERVAL '30 days'
                AND date < {_REF_DATE} - INTERVAL '7 days'
            ) baseline
        ) milk_r ON true
        LEFT JOIN LATERAL (
            SELECT ({_REF_DATE} - c.calving_date)::int8 as days
            FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
        ) dim ON true
        LEFT JOIN LATERAL (
            SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id
        ) lac ON true
        LEFT JOIN LATERAL (
            SELECT ({_REF_DATE} - h.heat_date)::int8 as days
            FROM heats h WHERE h.animal_id = a.id ORDER BY h.heat_date DESC LIMIT 1
        ) dsh ON true
        LEFT JOIN LATERAL (
            SELECT AVG(act.activity_counter)::float8 as avg_act
            FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= {_REF_DATE} - INTERVAL '14 days'
        ) act_14 ON true
        LEFT JOIN LATERAL (
            SELECT AVG(r.rumination_minutes)::float8 as avg_rum
            FROM ruminations r WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - INTERVAL '14 days'
        ) rum_14 ON true
        {_COMMON_JOINS_WEATHER}
        {_COMMON_JOINS_GENETICS}
        WHERE a.active = true AND a.gender = 'female'
        {(" AND a.id = :aid" if animal_id is not None else "")}
        ORDER BY a.name
    """)
    result = await session.execute(query, {"aid": animal_id} if animal_id is not None else {})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_equipment_anomaly_features(session: AsyncSession, animal_id: int | None = None) -> pd.DataFrame:
    query = text(f"""
        SELECT
            agg.animal_id,
            agg.animal_name,
            agg.device_address,
            agg.avg_conductivity,
            asym.max_quarter_asymmetry,
            agg.avg_milk_temperature,
            agg.std_milk_temperature,
            agg.avg_milk_yield_per_visit,
            agg.avg_milk_speed,
            COALESCE(anom.rate, 0) as anomaly_rate_7d
        FROM (
            SELECT
                v.animal_id,
                a.name as animal_name,
                v.device_address,
                AVG((v.lf_conductivity + v.lr_conductivity + v.rf_conductivity + v.rr_conductivity)::float8 / 4.0) as avg_conductivity,
                AVG(v.lf_conductivity)::float8 as avg_lf,
                AVG(v.lr_conductivity)::float8 as avg_lr,
                AVG(v.rf_conductivity)::float8 as avg_rf,
                AVG(v.rr_conductivity)::float8 as avg_rr,
                AVG(v.milk_temperature)::float8 as avg_milk_temperature,
                STDDEV(v.milk_temperature)::float8 as std_milk_temperature,
                AVG(v.milk_yield)::float8 as avg_milk_yield_per_visit,
                AVG(rmd.milk_speed)::float8 as avg_milk_speed
            FROM milk_visit_quality v
            JOIN animals a ON a.id = v.animal_id
            LEFT JOIN robot_milk_data rmd ON rmd.animal_id = v.animal_id AND rmd.milking_date = v.visit_datetime
            WHERE v.visit_datetime >= {_REF_DATE} - INTERVAL '7 days'
            GROUP BY v.animal_id, a.name, v.device_address
        ) agg
        LEFT JOIN LATERAL (
            SELECT GREATEST(
                ABS(agg.avg_lf - agg.avg_conductivity),
                ABS(agg.avg_lr - agg.avg_conductivity),
                ABS(agg.avg_rf - agg.avg_conductivity),
                ABS(agg.avg_rr - agg.avg_conductivity)
            )::float8 as max_quarter_asymmetry
        ) asym ON true
        LEFT JOIN LATERAL (
            SELECT (COUNT(*) FILTER (WHERE v3.success_milking = false OR v3.lf_colour_code IN ('W','Y') OR v3.lr_colour_code IN ('W','Y') OR v3.rf_colour_code IN ('W','Y') OR v3.rr_colour_code IN ('W','Y')))::float8 / NULLIF(COUNT(*)::float8, 0) as rate
            FROM milk_visit_quality v3 WHERE v3.animal_id = agg.animal_id AND v3.visit_datetime >= {_REF_DATE} - INTERVAL '7 days'
        ) anom ON true
        WHERE 1=1{(" AND agg.animal_id = :aid" if animal_id is not None else "")}
        ORDER BY agg.animal_name
    """)
    result = await session.execute(query, {"aid": animal_id} if animal_id is not None else {})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_feed_recommendation_features(session: AsyncSession, animal_id: int | None = None) -> pd.DataFrame:
    query = text(f"""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            COALESCE(dim.days, 0) as dim_days,
            COALESCE(lac.n, 0) as lactation_number,
            COALESCE(m7.milk, 0) as avg_milk_7d,
            COALESCE(f7.feed, 0) as avg_feed_7d,
            COALESCE(r7.rum, 0) as avg_rumination_7d,
            COALESCE(act7.act, 0) as avg_activity_7d,
            CASE WHEN f7.feed > 0 THEN m7.milk / f7.feed ELSE 0 END::float8 as milk_feed_ratio,
            COALESCE(scc30.scc, 0) as avg_scc_30d,
            COALESCE(m_trend.trend, 0) as milk_trend_7d
        FROM animals a
        LEFT JOIN LATERAL (SELECT ({_REF_DATE} - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
        LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac ON true
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '7 days') m7 ON true
        LEFT JOIN LATERAL (SELECT SUM(f.total)::float8 / NULLIF(COUNT(DISTINCT f.feed_date)::float8, 0) as feed FROM feed_day_amounts f WHERE f.animal_id = a.id AND f.feed_date >= {_REF_DATE} - INTERVAL '7 days') f7 ON true
        LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - INTERVAL '7 days') r7 ON true
        LEFT JOIN LATERAL (SELECT AVG(act.activity_counter)::float8 as act FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= {_REF_DATE} - INTERVAL '7 days') act7 ON true
        LEFT JOIN LATERAL (SELECT AVG(mq.scc)::float8 as scc FROM milk_quality mq WHERE mq.animal_id = a.id AND mq.date >= {_REF_DATE} - INTERVAL '30 days') scc30 ON true
        LEFT JOIN LATERAL (
            SELECT (recent.milk / NULLIF(baseline.milk, 0) - 1)::float8 as trend
            FROM (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '7 days') recent,
                 (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '14 days' AND m.date < {_REF_DATE} - INTERVAL '7 days') baseline
        ) m_trend ON true
        WHERE a.active = true AND a.gender = 'female' AND m7.milk IS NOT NULL
        {(" AND a.id = :aid" if animal_id is not None else "")}
        ORDER BY a.name
    """)
    result = await session.execute(query, {"aid": animal_id} if animal_id is not None else {})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_bcs_features(session: AsyncSession, animal_id: int | None = None) -> pd.DataFrame:
    query = text(f"""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            COALESCE(dim.days, 0) as dim_days,
            COALESCE(lac.n, 0) as lactation_number,
            COALESCE(m7.milk, 0) as avg_milk_7d,
            COALESCE(m30.milk, 0) as avg_milk_30d,
            COALESCE(f7.feed, 0) as avg_feed_7d,
            COALESCE(r7.rum, 0) as avg_rumination_7d,
            COALESCE(act7.act, 0) as avg_activity_7d,
            CASE WHEN f7.feed > 0 THEN m7.milk / f7.feed ELSE 0 END::float8 as milk_feed_ratio,
            COALESCE(m_trend.trend, 0) as milk_trend_7d,
            COALESCE(w.weight, 0) as weight_kg
        FROM animals a
        LEFT JOIN LATERAL (SELECT ({_REF_DATE} - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
        LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac ON true
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '7 days') m7 ON true
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '30 days') m30 ON true
        LEFT JOIN LATERAL (SELECT SUM(f.total)::float8 / NULLIF(COUNT(DISTINCT f.feed_date)::float8, 0) as feed FROM feed_day_amounts f WHERE f.animal_id = a.id AND f.feed_date >= {_REF_DATE} - INTERVAL '7 days') f7 ON true
        LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - INTERVAL '7 days') r7 ON true
        LEFT JOIN LATERAL (SELECT AVG(act.activity_counter)::float8 as act FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= {_REF_DATE} - INTERVAL '7 days') act7 ON true
        LEFT JOIN LATERAL (
            SELECT (recent.milk / NULLIF(baseline.milk, 0) - 1)::float8 as trend
            FROM (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '7 days') recent,
                 (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '14 days' AND m.date < {_REF_DATE} - INTERVAL '7 days') baseline
        ) m_trend ON true
        LEFT JOIN LATERAL (SELECT wr.weight_kg::float8 as weight FROM weight_records wr WHERE wr.animal_id = a.id ORDER BY wr.measure_date DESC LIMIT 1) w ON true
        WHERE a.active = true AND a.gender = 'female' AND m7.milk IS NOT NULL
        {(" AND a.id = :aid" if animal_id is not None else "")}
        ORDER BY a.name
    """)
    result = await session.execute(query, {"aid": animal_id} if animal_id is not None else {})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_ketosis_features(session: AsyncSession, animal_id: int | None = None) -> pd.DataFrame:
    query = text(f"""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            COALESCE(dim.days, 0) as dim_days,
            COALESCE(lac.n, 0) as lactation_number,
            COALESCE(mq7.fpr, 1.3) as fpr_7d,
            COALESCE(mq14.fpr, 1.3) as fpr_14d,
            COALESCE(mq_trend.fpr_trend, 0) as fpr_trend,
            COALESCE(r7.rum, 0) as avg_rumination_7d,
            COALESCE(r14.rum, 0) as avg_rumination_14d,
            COALESCE(r_trend.rum_trend, 0) as rumination_trend,
            COALESCE(m7.milk, 0) as avg_milk_7d,
            COALESCE(m_trend.trend, 0) as milk_trend,
            COALESCE(wc.temp, 0) as weather_temp,
            COALESCE(wc.humidity, 0) as weather_humidity,
            COALESCE(wc.thi, 0) as thi,
            0::float8 as vet_tx_count_180d,
            999::float8 as days_since_any_tx,
            COALESCE(lac7.lactose, 0) as avg_lactose_7d,
            COALESCE(lac_trend.lactose_trend, 0) as lactose_trend
        FROM animals a
        LEFT JOIN LATERAL (SELECT ({_REF_DATE} - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
        LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac ON true
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '7 days') m7 ON true
        LEFT JOIN LATERAL (SELECT AVG(mq.fat_percentage / NULLIF(mq.protein_percentage, 0))::float8 as fpr FROM milk_quality mq WHERE mq.animal_id = a.id AND mq.date >= {_REF_DATE} - INTERVAL '7 days') mq7 ON true
        LEFT JOIN LATERAL (SELECT AVG(mq.fat_percentage / NULLIF(mq.protein_percentage, 0))::float8 as fpr FROM milk_quality mq WHERE mq.animal_id = a.id AND mq.date >= {_REF_DATE} - INTERVAL '14 days') mq14 ON true
        LEFT JOIN LATERAL (
            SELECT (recent_fpr.fpr / NULLIF(baseline_fpr.fpr, 0) - 1)::float8 as fpr_trend
            FROM (SELECT AVG(mq.fat_percentage / NULLIF(mq.protein_percentage, 0))::float8 as fpr FROM milk_quality mq WHERE mq.animal_id = a.id AND mq.date >= {_REF_DATE} - INTERVAL '7 days') recent_fpr,
                 (SELECT AVG(mq.fat_percentage / NULLIF(mq.protein_percentage, 0))::float8 as fpr FROM milk_quality mq WHERE mq.animal_id = a.id AND mq.date >= {_REF_DATE} - INTERVAL '14 days' AND mq.date < {_REF_DATE} - INTERVAL '7 days') baseline_fpr
        ) mq_trend ON true
        LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - INTERVAL '7 days') r7 ON true
        LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - INTERVAL '14 days') r14 ON true
        LEFT JOIN LATERAL (
            SELECT (recent_r.rum / NULLIF(baseline_r.rum, 0) - 1)::float8 as rum_trend
            FROM (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - INTERVAL '7 days') recent_r,
                 (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= {_REF_DATE} - INTERVAL '14 days' AND r.date < {_REF_DATE} - INTERVAL '7 days') baseline_r
        ) r_trend ON true
        LEFT JOIN LATERAL (
            SELECT (recent_m.milk / NULLIF(baseline_m.milk, 0) - 1)::float8 as trend
            FROM (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '7 days') recent_m,
                 (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= {_REF_DATE} - INTERVAL '14 days' AND m.date < {_REF_DATE} - INTERVAL '7 days') baseline_m
        ) m_trend ON true
        LEFT JOIN LATERAL (SELECT AVG(mq.lactose_percentage)::float8 as lactose FROM milk_quality mq WHERE mq.animal_id = a.id AND mq.date >= {_REF_DATE} - INTERVAL '7 days') lac7 ON true
        LEFT JOIN LATERAL (
            SELECT (recent_l.lactose / NULLIF(baseline_l.lactose, 0) - 1)::float8 as lactose_trend
            FROM (SELECT AVG(mq.lactose_percentage)::float8 as lactose FROM milk_quality mq WHERE mq.animal_id = a.id AND mq.date >= {_REF_DATE} - INTERVAL '7 days') recent_l,
                 (SELECT AVG(mq.lactose_percentage)::float8 as lactose FROM milk_quality mq WHERE mq.animal_id = a.id AND mq.date >= {_REF_DATE} - INTERVAL '14 days' AND mq.date < {_REF_DATE} - INTERVAL '7 days') baseline_l
        ) lac_trend ON true
        LEFT JOIN LATERAL (SELECT temp_c::float8 as temp, humidity::float8 as humidity, thi::float8 as thi FROM weather_cache wc ORDER BY wc.date DESC LIMIT 1) wc ON true
        WHERE a.active = true AND a.gender = 'female' AND m7.milk IS NOT NULL
        {(" AND a.id = :aid" if animal_id is not None else "")}
        ORDER BY a.name
    """)
    result = await session.execute(query, {"aid": animal_id} if animal_id is not None else {})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)
