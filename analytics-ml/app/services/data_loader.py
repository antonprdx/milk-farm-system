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
            COALESCE(act_7d.act, 0) as avg_activity_7d,
            COALESCE(fpr.ratio, 0) as fat_protein_ratio,
            COALESCE(cond_asym.asym, 0) as cond_asymmetry
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
        LEFT JOIN LATERAL (
            SELECT (AVG(q.fat_percentage) / NULLIF(AVG(q.protein_percentage), 0))::float8 as ratio
            FROM milk_quality q
            WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
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
                WHERE v.animal_id = a.id AND v.visit_datetime >= CURRENT_DATE - INTERVAL '7 days'
            ) sub
        ) cond_asym ON true
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
            a.name as animal_name,
            m.date,
            m.milk_amount,
            COALESCE(f.total, 0) as feed_amount,
            COALESCE(r.rumination_minutes, 0) as rumination_minutes,
            COALESCE(act.activity_counter, 0) as activity_counter
        FROM milk_day_productions m
        JOIN animals a ON a.id = m.animal_id
        LEFT JOIN feed_day_amounts f ON f.animal_id = m.animal_id AND f.feed_date = m.date
        LEFT JOIN ruminations r ON r.animal_id = m.animal_id AND r.date = m.date
        LEFT JOIN LATERAL (
            SELECT AVG(activity_counter)::float8 as activity_counter
            FROM activities WHERE animal_id = m.animal_id AND activity_datetime::date = m.date
        ) act ON true
        WHERE m.animal_id = :animal_id AND m.date >= CURRENT_DATE - make_interval(days => :days)
        ORDER BY m.date
    """)
    result = await session.execute(query, {"animal_id": animal_id, "days": days})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_clustering_features(session: AsyncSession, days: int = 90) -> pd.DataFrame:
    query = text("""
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
            WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - make_interval(days => :days)
        ) m ON true
        LEFT JOIN LATERAL (
            SELECT AVG(r.rumination_minutes)::float8 as rum
            FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - make_interval(days => :days)
        ) r ON true
        LEFT JOIN LATERAL (
            SELECT AVG(act.activity_counter)::float8 as act
            FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - make_interval(days => :days)
        ) act ON true
        LEFT JOIN LATERAL (
            SELECT SUM(f.total)::float8 / NULLIF(COUNT(DISTINCT f.feed_date)::float8, 0) as feed
            FROM feed_day_amounts f WHERE f.animal_id = a.id AND f.feed_date >= CURRENT_DATE - make_interval(days => :days)
        ) f ON true
        LEFT JOIN LATERAL (
            SELECT (CURRENT_DATE - c.calving_date)::int8 as days
            FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
        ) dim ON true
        LEFT JOIN LATERAL (
            SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id
        ) lac ON true
        WHERE a.active = true AND a.gender = 'female' AND m.avg_milk IS NOT NULL
        ORDER BY a.name
    """)
    result = await session.execute(query, {"days": days})
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    df = pd.DataFrame(rows)
    df["animal_name"] = df["animal_name"].fillna("")
    return df


async def load_estrus_features(session: AsyncSession) -> pd.DataFrame:
    query = text("""
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
            COALESCE(rum_14.avg_rum, 0) as avg_rumination_14d
        FROM animals a
        LEFT JOIN LATERAL (
            SELECT (recent.act / NULLIF(baseline.act, 0))::float8 as ratio FROM (
                SELECT AVG(activity_counter)::float8 as act FROM activities
                WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '7 days'
            ) recent, (
                SELECT AVG(activity_counter)::float8 as act FROM activities
                WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '30 days'
                AND activity_datetime < CURRENT_DATE - INTERVAL '7 days'
            ) baseline
        ) act_r ON true
        LEFT JOIN LATERAL (
            SELECT (recent.rum / NULLIF(baseline.rum, 0))::float8 as ratio FROM (
                SELECT AVG(rumination_minutes)::float8 as rum FROM ruminations
                WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '7 days'
            ) recent, (
                SELECT AVG(rumination_minutes)::float8 as rum FROM ruminations
                WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '30 days'
                AND date < CURRENT_DATE - INTERVAL '7 days'
            ) baseline
        ) rum_r ON true
        LEFT JOIN LATERAL (
            SELECT (recent.milk / NULLIF(baseline.milk, 0))::float8 as ratio FROM (
                SELECT AVG(milk_amount)::float8 as milk FROM milk_day_productions
                WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '7 days'
            ) recent, (
                SELECT AVG(milk_amount)::float8 as milk FROM milk_day_productions
                WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '30 days'
                AND date < CURRENT_DATE - INTERVAL '7 days'
            ) baseline
        ) milk_r ON true
        LEFT JOIN LATERAL (
            SELECT (CURRENT_DATE - c.calving_date)::int8 as days
            FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
        ) dim ON true
        LEFT JOIN LATERAL (
            SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id
        ) lac ON true
        LEFT JOIN LATERAL (
            SELECT (CURRENT_DATE - h.heat_date)::int8 as days
            FROM heats h WHERE h.animal_id = a.id ORDER BY h.heat_date DESC LIMIT 1
        ) dsh ON true
        LEFT JOIN LATERAL (
            SELECT AVG(act.activity_counter)::float8 as avg_act
            FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - INTERVAL '14 days'
        ) act_14 ON true
        LEFT JOIN LATERAL (
            SELECT AVG(r.rumination_minutes)::float8 as avg_rum
            FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '14 days'
        ) rum_14 ON true
        WHERE a.active = true AND a.gender = 'female'
        ORDER BY a.name
    """)
    result = await session.execute(query)
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_equipment_anomaly_features(session: AsyncSession) -> pd.DataFrame:
    query = text("""
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
            WHERE v.visit_datetime >= CURRENT_DATE - INTERVAL '7 days'
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
            FROM milk_visit_quality v3 WHERE v3.animal_id = agg.animal_id AND v3.visit_datetime >= CURRENT_DATE - INTERVAL '7 days'
        ) anom ON true
        ORDER BY agg.animal_name
    """)
    result = await session.execute(query)
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_feed_recommendation_features(session: AsyncSession) -> pd.DataFrame:
    query = text("""
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
        LEFT JOIN LATERAL (SELECT (CURRENT_DATE - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
        LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac ON true
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days') m7 ON true
        LEFT JOIN LATERAL (SELECT AVG(f.total)::float8 as feed FROM feed_day_amounts f WHERE f.animal_id = a.id AND f.feed_date >= CURRENT_DATE - INTERVAL '7 days') f7 ON true
        LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '7 days') r7 ON true
        LEFT JOIN LATERAL (SELECT AVG(act.activity_counter)::float8 as act FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - INTERVAL '7 days') act7 ON true
        LEFT JOIN LATERAL (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '30 days') scc30 ON true
        LEFT JOIN LATERAL (
            SELECT (recent.milk / NULLIF(baseline.milk, 0) - 1)::float8 as trend
            FROM (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days') recent,
                 (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '14 days' AND m.date < CURRENT_DATE - INTERVAL '7 days') baseline
        ) m_trend ON true
        WHERE a.active = true AND a.gender = 'female' AND m7.milk IS NOT NULL
        ORDER BY a.name
    """)
    result = await session.execute(query)
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)


async def load_ketosis_features(session: AsyncSession) -> pd.DataFrame:
    query = text("""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            COALESCE(fpr7.ratio, 0) as fpr_7d,
            COALESCE(fpr14.ratio, 0) as fpr_14d,
            COALESCE(fpr_trend.trend, 0) as fpr_trend,
            COALESCE(rum7.rum, 0) as avg_rumination_7d,
            COALESCE(rum14.rum, 0) as avg_rumination_14d,
            COALESCE(rum_trend.trend, 0) as rumination_trend,
            COALESCE(milk7.milk, 0) as avg_milk_7d,
            COALESCE(milk_trend.trend, 0) as milk_trend,
            COALESCE(dim.days, 0) as dim_days,
            COALESCE(lac.n, 0) as lactation_number
        FROM animals a
        LEFT JOIN LATERAL (
            SELECT (AVG(q.fat_percentage) / NULLIF(AVG(q.protein_percentage), 0))::float8 as ratio
            FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
        ) fpr7 ON true
        LEFT JOIN LATERAL (
            SELECT (AVG(q.fat_percentage) / NULLIF(AVG(q.protein_percentage), 0))::float8 as ratio
            FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '14 days'
        ) fpr14 ON true
        LEFT JOIN LATERAL (
            SELECT (fpr7_inner.ratio / NULLIF(fpr14_inner.ratio, 0) - 1)::float8 as trend
            FROM (SELECT AVG(q.fat_percentage)::float8 / NULLIF(AVG(q.protein_percentage)::float8, 0) as ratio FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days') fpr7_inner,
                 (SELECT AVG(q.fat_percentage)::float8 / NULLIF(AVG(q.protein_percentage)::float8, 0) as ratio FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '14 days' AND q.date < CURRENT_DATE - INTERVAL '7 days') fpr14_inner
        ) fpr_trend ON true
        LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '7 days') rum7 ON true
        LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '14 days') rum14 ON true
        LEFT JOIN LATERAL (
            SELECT (rum7_inner.rum / NULLIF(rum14_inner.rum, 0) - 1)::float8 as trend
            FROM (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '7 days') rum7_inner,
                 (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '14 days' AND r.date < CURRENT_DATE - INTERVAL '7 days') rum14_inner
        ) rum_trend ON true
        LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days') milk7 ON true
        LEFT JOIN LATERAL (
            SELECT (recent.milk / NULLIF(baseline.milk, 0) - 1)::float8 as trend
            FROM (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days') recent,
                 (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '14 days' AND m.date < CURRENT_DATE - INTERVAL '7 days') baseline
        ) milk_trend ON true
        LEFT JOIN LATERAL (SELECT (CURRENT_DATE - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
        LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac ON true
        WHERE a.active = true AND a.gender = 'female' AND fpr7.ratio IS NOT NULL
        ORDER BY a.name
    """)
    result = await session.execute(query)
    rows = result.mappings().all()
    if not rows:
        return pd.DataFrame()
    return pd.DataFrame(rows)
