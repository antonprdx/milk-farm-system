from __future__ import annotations

from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession

from app.services.data_loader import async_session


async def compute_animal_features(
    session: AsyncSession,
    animal_id: int | None = None,
    days_back: int = 30,
) -> dict:
    conditions = ["a.active = true", "a.gender = 'female'"]
    params: dict = {"days": days_back}
    if animal_id is not None:
        conditions.append("a.id = :animal_id")
        params["animal_id"] = animal_id

    where = " AND ".join(conditions)

    query = text(f"""
        SELECT
            a.id as animal_id,
            a.name as animal_name,
            m.milk_7d,
            m.milk_30d,
            m.milk_trend,
            f.feed_7d,
            r.rum_7d,
            r.rum_30d,
            act.act_7d,
            scc.scc_30d,
            dim.days as dim_days,
            lac.n as lactation_number
        FROM animals a
        LEFT JOIN LATERAL (
            SELECT AVG(m.milk_amount)::float8 as milk_7d,
                   (SELECT AVG(m2.milk_amount)::float8 FROM milk_day_productions m2 WHERE m2.animal_id = a.id AND m2.date >= CURRENT_DATE - make_interval(days => :days)) as milk_30d,
                   (SELECT AVG(m3.milk_amount)::float8 FROM milk_day_productions m3 WHERE m3.animal_id = a.id AND m3.date >= CURRENT_DATE - INTERVAL '7 days')
                     / NULLIF((SELECT AVG(m4.milk_amount)::float8 FROM milk_day_productions m4 WHERE m4.animal_id = a.id AND m4.date >= CURRENT_DATE - INTERVAL '14 days' AND m4.date < CURRENT_DATE - INTERVAL '7 days'), 0) - 1 as milk_trend
            FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days'
        ) m ON true
        LEFT JOIN LATERAL (SELECT AVG(f.total)::float8 as feed_7d FROM feed_day_amounts f WHERE f.animal_id = a.id AND f.feed_date >= CURRENT_DATE - INTERVAL '7 days') f ON true
        LEFT JOIN LATERAL (
            SELECT AVG(r.rumination_minutes)::float8 as rum_7d,
                   (SELECT AVG(r2.rumination_minutes)::float8 FROM ruminations r2 WHERE r2.animal_id = a.id AND r2.date >= CURRENT_DATE - make_interval(days => :days)) as rum_30d
            FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '7 days'
        ) r ON true
        LEFT JOIN LATERAL (SELECT AVG(act.activity_counter)::float8 as act_7d FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - INTERVAL '7 days') act ON true
        LEFT JOIN LATERAL (SELECT AVG(q.scc)::float8 as scc_30d FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - make_interval(days => :days)) scc ON true
        LEFT JOIN LATERAL (SELECT (CURRENT_DATE - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
        LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac ON true
        WHERE {where}
    """)

    result = await session.execute(query, params)
    rows = result.mappings().all()
    return [dict(r) for r in rows] if rows else []
