"""
Augment imported public data with realistic synthetic activity, rumination,
weather, health events, and estrus patterns based on real milk yields.

Usage:
    DATABASE_URL=postgres://milkfarm:milkfarm@localhost:5432/milkfarm python augment_real_data.py

Reads milk_day_productions from DB, generates correlated data for missing tables.
"""
from __future__ import annotations

import asyncio
import datetime as dt
import math
import os
import random
from collections import defaultdict

import asyncpg

ACTIVITIES_SQL = """
INSERT INTO activities (animal_id, activity_datetime, activity_counter, heat_attention)
VALUES ($1, $2, $3, $4)
ON CONFLICT (animal_id, activity_datetime) DO NOTHING
"""

HEATS_SQL = """
INSERT INTO heats (animal_id, heat_date, confirmed, confirmation_method)
VALUES ($1, $2, $3, $4)
"""

VET_MASTITIS_SQL = """
INSERT INTO vet_records (animal_id, record_type, diagnosis_code, confirmed, event_date, diagnosis)
VALUES ($1, 'disease', 'mastitis', true, $2, 'Клинический мастит')
"""

VET_KETOSIS_SQL = """
INSERT INTO vet_records (animal_id, record_type, diagnosis_code, confirmed, event_date, diagnosis)
VALUES ($1, 'disease', 'ketosis', true, $2, 'Субклинический кетоз')
"""

VET_LAMENESS_SQL = """
INSERT INTO vet_records (animal_id, record_type, diagnosis_code, confirmed, event_date, diagnosis)
VALUES ($1, 'disease', 'lameness', true, $2, 'Хромота')
"""

MILK_QUALITY_SQL = """
INSERT INTO milk_quality (animal_id, date, fat_percentage, protein_percentage,
                          lactose_percentage, scc, milkings, refusals)
VALUES ($1, $2, $3, $4, $5, $6, 2, 0)
ON CONFLICT (animal_id, date) DO NOTHING
"""

WEATHER_SQL = """
INSERT INTO weather_cache (date, temp_c, humidity, precipitation_mm, wind_speed, weather_main, thi)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (date) DO UPDATE SET
    temp_c = EXCLUDED.temp_c,
    humidity = EXCLUDED.humidity,
    precipitation_mm = EXCLUDED.precipitation_mm,
    wind_speed = EXCLUDED.wind_speed,
    weather_main = EXCLUDED.weather_main,
    thi = EXCLUDED.thi
"""


def calc_thi(temp_c: float, humidity: float) -> float:
    rh = max(0, min(100, humidity))
    thi = (1.8 * temp_c + 32) - (0.55 - 0.0055 * rh) * (1.8 * temp_c + 26)
    return round(thi, 1)


def seasonal_temp(day_of_year: int) -> float:
    base = 10.0
    amplitude = 15.0
    return base + amplitude * math.cos(2 * math.pi * (day_of_year - 200) / 365)


def seasonal_humidity(day_of_year: int) -> float:
    base = 65.0
    amplitude = 15.0
    return base + amplitude * math.sin(2 * math.pi * (day_of_year - 80) / 365)


async def run(db_url: str) -> None:
    pool = await asyncpg.create_pool(db_url, min_size=2, max_size=8)

    async with pool.acquire() as conn:
        milk_rows = await conn.fetch(
            "SELECT animal_id, date, milk_amount FROM milk_day_productions ORDER BY animal_id, date"
        )
        calvings = await conn.fetch(
            "SELECT animal_id, calving_date, lac_number FROM calvings ORDER BY animal_id, calving_date"
        )
        existing_mq_dates = await conn.fetch("SELECT DISTINCT date FROM milk_quality")
        existing_weather = await conn.fetch("SELECT date FROM weather_cache")
        existing_activities = await conn.fetch("SELECT DISTINCT animal_id FROM activities")

    if not milk_rows:
        print("No milk data found. Run import_public_data.py first.")
        return

    print(f"Found {len(milk_rows)} milk records")

    animal_milk: dict[int, list[tuple[dt.date, float]]] = defaultdict(list)
    for r in milk_rows:
        animal_milk[r["animal_id"]].append((r["date"], float(r["milk_amount"])))

    calving_map: dict[int, list[tuple[dt.date, int]]] = defaultdict(list)
    for r in calvings:
        calving_map[r["animal_id"]].append((r["calving_date"], r["lac_number"]))

    existing_mq_set = {r["date"] for r in existing_mq_dates}
    existing_w_set = {r["date"] for r in existing_weather}
    existing_act_set = {r["animal_id"] for r in existing_activities}

    rng = random.Random(42)

    # --- Weather ---
    print("Generating weather...")
    all_dates = sorted({d for _, entries in animal_milk.items() for d, _ in entries})
    weather_batch = []
    for d in all_dates:
        if d in existing_w_set:
            continue
        doy = d.timetuple().tm_yday
        temp = seasonal_temp(doy) + rng.gauss(0, 3)
        hum = seasonal_humidity(doy) + rng.gauss(0, 5)
        hum = max(20, min(100, hum))
        precip = max(0, rng.gauss(2, 4))
        wind = max(0, rng.gauss(4, 2))
        main = "Clear" if precip < 1 else ("Rain" if precip < 5 else "Storm")
        thi = calc_thi(temp, hum)
        weather_batch.append((d, round(temp, 1), round(hum, 1), round(precip, 1), round(wind, 1), main, thi))

    async with pool.acquire() as conn:
        for i in range(0, len(weather_batch), 1000):
            await conn.executemany(WEATHER_SQL, weather_batch[i : i + 1000])
    print(f"  {len(weather_batch)} weather rows")

    # --- Estrus heats ---
    print("Generating estrus heats...")
    heats_batch = []
    heat_date_set: dict[int, set[dt.date]] = defaultdict(set)
    for aid, calv_list in calving_map.items():
        for calving_date, _lac in calv_list:
            current = calving_date + dt.timedelta(days=35)
            end_date = calving_date + dt.timedelta(days=400)
            while current <= end_date:
                confirmed = rng.random() < 0.7
                method = "visual" if rng.random() < 0.5 else "activity_monitor"
                heats_batch.append((aid, current, confirmed, method))
                heat_date_set[aid].add(current)
                cycle_len = max(15, int(rng.gauss(21, 3)))
                current += dt.timedelta(days=cycle_len)
    async with pool.acquire() as conn:
        for i in range(0, len(heats_batch), 5000):
            await conn.executemany(HEATS_SQL, heats_batch[i : i + 5000])
    print(f"  {len(heats_batch)} heats")

    # --- Activities with estrus spikes ---
    print("Generating activities...")
    act_batch = []
    for aid in animal_milk:
        if aid in existing_act_set:
            continue
        entries = animal_milk[aid]
        base_counter = 300 + (aid % 100)
        for date_val, milk in entries:
            is_heat = date_val in heat_date_set.get(aid, set())
            if is_heat:
                counter = int(base_counter * rng.uniform(2.0, 3.0))
                heat_att = True
            else:
                counter = base_counter + rng.randint(-30, 30)
                heat_att = False
            act_dt = dt.datetime.combine(date_val, dt.time(8, 0))
            act_batch.append((aid, act_dt, counter, heat_att))
            base_counter = counter

    async with pool.acquire() as conn:
        for i in range(0, len(act_batch), 5000):
            await conn.executemany(ACTIVITIES_SQL, act_batch[i : i + 5000])
    print(f"  {len(act_batch)} activity rows")

    # --- Milk quality with mastitis/ketosis signals ---
    print("Generating milk quality...")
    mq_batch = []
    vet_mastitis = []
    vet_ketosis = []
    vet_lameness = []

    for aid, entries in animal_milk.items():
        calvs = calving_map.get(aid, [])
        if not calvs:
            continue
        last_calving = max(c[0] for c in calvs)

        for i, (date_val, milk) in enumerate(entries):
            if date_val in existing_mq_set:
                continue

            dim = (date_val - last_calving).days

            is_mastitis = rng.random() < 0.03
            is_ketosis = dim > 0 and dim < 60 and rng.random() < 0.08

            if is_mastitis:
                scc = rng.randint(400, 3000) * 1000
                fat = max(2.5, 3.8 - 0.005 * milk + rng.gauss(0, 0.5))
                protein = max(2.5, 3.2 - 0.003 * milk + rng.gauss(0, 0.3))
                lactose = max(3.0, 4.3 + rng.gauss(0, 0.2))
                vet_mastitis.append((aid, date_val))
            elif is_ketosis:
                scc = rng.randint(20, 150) * 1000
                fat = min(6.0, 4.5 + rng.gauss(0, 0.3))
                protein = max(2.5, 2.8 + rng.gauss(0, 0.2))
                lactose = max(3.5, 4.6 + rng.gauss(0, 0.15))
                vet_ketosis.append((aid, date_val))
            else:
                scc = rng.randint(20, 250) * 1000
                fat = max(2.5, min(6.0, 3.8 - 0.005 * milk + rng.gauss(0, 0.3)))
                protein = max(2.5, min(4.5, 3.2 - 0.003 * milk + rng.gauss(0, 0.2)))
                lactose = max(3.5, min(5.5, 4.6 + rng.gauss(0, 0.15)))

            fpr = fat / max(protein, 0.1)
            if fpr > 1.5 and dim < 60:
                vet_ketosis.append((aid, date_val))

            mq_batch.append((aid, date_val, round(fat, 2), round(protein, 2), round(lactose, 2), scc))

        if rng.random() < 0.15:
            lameness_idx = rng.randint(0, max(0, len(entries) - 1))
            vet_lameness.append((aid, entries[lameness_idx][0]))

    async with pool.acquire() as conn:
        for i in range(0, len(mq_batch), 5000):
            await conn.executemany(MILK_QUALITY_SQL, mq_batch[i : i + 5000])
        for i in range(0, len(vet_mastitis), 1000):
            await conn.executemany(VET_MASTITIS_SQL, vet_mastitis[i : i + 1000])
        for i in range(0, len(vet_ketosis), 1000):
            await conn.executemany(VET_KETOSIS_SQL, vet_ketosis[i : i + 1000])
        for i in range(0, len(vet_lameness), 1000):
            await conn.executemany(VET_LAMENESS_SQL, vet_lameness[i : i + 1000])
    print(f"  {len(mq_batch)} milk quality rows")
    print(f"  {len(vet_mastitis)} mastitis events")
    print(f"  {len(vet_ketosis)} ketosis events")
    print(f"  {len(vet_lameness)} lameness events")

    await pool.close()
    print("Augmentation complete!")


async def generate_milk_timeseries(db_url: str, n_cows: int = 500, lactation_days: int = 305) -> None:
    pool = await asyncpg.create_pool(db_url, min_size=2, max_size=8)

    async with pool.acquire() as conn:
        cows = await conn.fetch(
            "SELECT a.id, a.birth_date, c.calving_date, c.lac_number "
            "FROM animals a JOIN calvings c ON c.animal_id = a.id "
            "WHERE a.active = true AND a.gender = 'female' "
            "ORDER BY a.id LIMIT $1", n_cows * 3,
        )

    if not cows:
        print("No cows with calvings found")
        await pool.close()
        return

    rng = random.Random(123)
    batch = []
    used = set()
    for row in cows:
        aid = row["id"]
        if aid in used:
            continue
        used.add(aid)
        if len(used) > n_cows:
            break

        calving_date = row["calving_date"]
        peak_yield = rng.uniform(20, 45)
        a_param = 0.15
        b_param = -0.002
        c_param = 0.003

        for dim in range(1, lactation_days + 1):
            d = calving_date + dt.timedelta(days=dim)
            wood = a_param * (dim ** b_param) * math.exp(-c_param * dim)
            milk = peak_yield * wood * (1 + rng.gauss(0, 0.08))
            milk = max(0.5, milk)
            batch.append((aid, d, round(milk, 2)))

    print(f"Inserting {len(batch)} milk timeseries rows for {len(used)} cows...")
    async with pool.acquire() as conn:
        for i in range(0, len(batch), 5000):
            await conn.executemany(
                "INSERT INTO milk_day_productions (animal_id, date, milk_amount) "
                "VALUES ($1, $2, $3) ON CONFLICT (animal_id, date) DO UPDATE SET milk_amount = EXCLUDED.milk_amount",
                batch[i:i + 5000],
            )
    await pool.close()
    print(f"  Done: {len(batch)} rows")


if __name__ == "__main__":
    db_url = os.environ.get("DATABASE_URL", "postgres://milkfarm:milkfarm@localhost:5432/milkfarm")
    asyncio.run(run(db_url))
    asyncio.run(generate_milk_timeseries(db_url))
