"""
Import FelBenitez dairy-cow-milk-yield-prediction dataset into milk-farm DB.

Usage:
    DATABASE_URL=postgres://milkfarm:milkfarm@localhost:5432/milkfarm python import_public_data.py

Dataset: https://github.com/FelBenitez/dairy-cow-milk-yield-prediction
"""
from __future__ import annotations

import asyncio
import datetime
import io
import os
import random
import urllib.request

import asyncpg
import numpy as np
import pandas as pd

DATA_URL = (
    "https://raw.githubusercontent.com/FelBenitez/"
    "dairy-cow-milk-yield-prediction/main/data/cattle_data_train.csv"
)

BREED_MAP = {
    "Holstein": "HOL",
    "Jersey": "JER",
    "Ayrshire": "AYR",
    "Guernsey": "GUE",
    "Brown Swiss": "BSW",
    "Simmental": "SIM",
}

BATCH = 10000


async def run(db_url: str) -> None:
    pool = await asyncpg.create_pool(db_url, min_size=4, max_size=8)

    print("Downloading dataset...")
    resp = urllib.request.urlopen(DATA_URL)
    raw = resp.read()
    df = pd.read_csv(io.BytesIO(raw))
    print(f"Loaded {len(df)} rows, {df['Cattle_ID'].nunique()} unique cattle")

    df["Date"] = pd.to_datetime(df["Date"])
    df["date_val"] = df["Date"].dt.date

    # --- Animal mapping ---
    unique_cattle = df.drop_duplicates(subset=["Cattle_ID"])[["Cattle_ID", "Breed", "Age_Months", "Weight_kg", "Parity", "Body_Condition_Score", "Farm_ID"]].copy()
    unique_cattle = unique_cattle.sort_values("Cattle_ID").reset_index(drop=True)

    async with pool.acquire() as conn:
        existing = await conn.fetch("SELECT id FROM animals")
        start_id = max((r["id"] for r in existing), default=0) + 1

    animal_id_map = dict(zip(unique_cattle["Cattle_ID"], range(start_id, start_id + len(unique_cattle))))
    df["aid"] = df["Cattle_ID"].map(animal_id_map)

    # --- Locations ---
    print("Inserting locations...")
    farms = df["Farm_ID"].unique()
    async with pool.acquire() as conn:
        for farm in farms:
            await conn.execute(
                "INSERT INTO locations (name, location_type) SELECT $1, 'farm' WHERE NOT EXISTS (SELECT 1 FROM locations WHERE name = $1)",
                f"Ферма {farm}",
            )
    print(f"  {len(farms)} farms")

    # --- Animals, bloodlines, calvings ---
    print("Inserting animals, bloodlines, calvings...")
    base_date = pd.Timestamp("2024-06-01")
    unique_cattle["aid"] = unique_cattle["Cattle_ID"].map(animal_id_map)
    unique_cattle["birth_date"] = unique_cattle["Age_Months"].apply(lambda m: (base_date - pd.DateOffset(months=int(m))).date())
    unique_cattle["breed_code"] = unique_cattle["Breed"].map(BREED_MAP).fillna("HOL")

    batch_animals = [
        (row["aid"], f"Корова-{row['aid']}", row["birth_date"], f"Ферма {row['Farm_ID']}")
        for _, row in unique_cattle.iterrows()
    ]
    batch_blood = [
        (row["aid"], row["breed_code"], 100)
        for _, row in unique_cattle.iterrows()
    ]
    batch_calvings = []
    for _, row in unique_cattle.iterrows():
        for lac in range(1, int(row["Parity"]) + 1):
            offset = (int(row["Parity"]) - lac) * 14
            cd = (base_date - pd.DateOffset(months=offset)).date()
            batch_calvings.append((row["aid"], cd, lac))

    async with pool.acquire() as conn:
        for i in range(0, len(batch_animals), BATCH):
            await conn.executemany(
                "INSERT INTO animals (id, name, gender, birth_date, active, location) VALUES ($1,$2,'female',$3,true,$4) ON CONFLICT (id) DO NOTHING",
                batch_animals[i:i+BATCH],
            )
        for i in range(0, len(batch_blood), BATCH):
            await conn.executemany(
                "INSERT INTO bloodlines (animal_id, blood_type_code, percentage) SELECT $1,$2,$3 WHERE NOT EXISTS (SELECT 1 FROM bloodlines WHERE animal_id=$1 AND blood_type_code=$2)",
                batch_blood[i:i+BATCH],
            )
        for i in range(0, len(batch_calvings), BATCH):
            await conn.executemany(
                "INSERT INTO calvings (animal_id, calving_date, lac_number) SELECT $1,$2,$3 WHERE NOT EXISTS (SELECT 1 FROM calvings WHERE animal_id=$1 AND calving_date=$2 AND lac_number=$3)",
                batch_calvings[i:i+BATCH],
            )
    print(f"  {len(batch_animals)} animals, {len(batch_blood)} bloodlines, {len(batch_calvings)} calvings")

    # --- Daily data (vectorized) ---
    print("Building daily data batches...")
    df_valid = df[df["aid"].notna()].copy()
    df_valid["aid"] = df_valid["aid"].astype(int)

    # Milk
    batch_milk = list(zip(
        df_valid["aid"].values,
        df_valid["date_val"].values,
        df_valid["Milk_Yield_L"].round(2).values,
    ))

    # Feed
    feed_df = df_valid[df_valid["Feed_Quantity_kg"].notna()].copy()
    batch_feed = list(zip(
        feed_df["aid"].values,
        feed_df["date_val"].values,
        feed_df["Feed_Quantity_kg"].round(2).values,
    ))

    # Rumination
    rum_df = df_valid[df_valid["Rumination_Time_hrs"].notna()].copy()
    rum_hrs = rum_df["Rumination_Time_hrs"].values
    rum_min = np.maximum(0, (rum_hrs * 60).astype(int))
    eating_min = (rum_hrs * 0.4 * 60).astype(int)
    batch_rum = list(zip(
        rum_df["aid"].values,
        rum_df["date_val"].values,
        eating_min,
        rum_min,
    ))

    # Weather (unique dates)
    weather_df = df_valid[df_valid["Ambient_Temperature_C"].notna() & df_valid["Humidity_percent"].notna()].copy()
    weather_unique = weather_df.groupby("date_val").agg({
        "Ambient_Temperature_C": "first",
        "Humidity_percent": "first",
    }).reset_index()
    batch_weather = list(zip(
        weather_unique["date_val"].values,
        weather_unique["Ambient_Temperature_C"].round(1).values,
        weather_unique["Humidity_percent"].round(1).values,
        ["Clear"] * len(weather_unique),
    ))

    # Vet (mastitis)
    mastitis_df = df_valid[df_valid["Mastitis"] == 1].copy()
    batch_vet = list(zip(
        mastitis_df["aid"].values,
        mastitis_df["date_val"].values,
    ))

    # Weight
    batch_weight = list(zip(
        unique_cattle["aid"].values,
        unique_cattle["Weight_kg"].round(1).values,
        unique_cattle["birth_date"].values,
    ))

    print(f"  milk:{len(batch_milk)} feed:{len(batch_feed)} rum:{len(batch_rum)} weather:{len(batch_weather)} vet:{len(batch_vet)} weight:{len(batch_weight)}")

    # --- Insert daily data ---
    async with pool.acquire() as conn:
        print("  milk_day_productions...")
        for i in range(0, len(batch_milk), BATCH):
            await conn.executemany(
                "INSERT INTO milk_day_productions (animal_id, date, milk_amount) VALUES ($1,$2,$3) ON CONFLICT (animal_id, date) DO NOTHING",
                batch_milk[i:i+BATCH],
            )
        print(f"    {len(batch_milk)} rows")

        print("  feed_day_amounts...")
        for i in range(0, len(batch_feed), BATCH):
            await conn.executemany(
                "INSERT INTO feed_day_amounts (animal_id, feed_date, feed_number, total) VALUES ($1,$2,1,$3) ON CONFLICT (animal_id, feed_date, feed_number) DO NOTHING",
                batch_feed[i:i+BATCH],
            )
        print(f"    {len(batch_feed)} rows")

        print("  ruminations...")
        for i in range(0, len(batch_rum), BATCH):
            await conn.executemany(
                "INSERT INTO ruminations (animal_id, date, eating_seconds, rumination_minutes) VALUES ($1,$2,$3,$4) ON CONFLICT (animal_id, date) DO NOTHING",
                batch_rum[i:i+BATCH],
            )
        print(f"    {len(batch_rum)} rows")

        print("  weather_cache...")
        for i in range(0, len(batch_weather), BATCH):
            await conn.executemany(
                "INSERT INTO weather_cache (date, temp_c, humidity, weather_main) VALUES ($1,$2,$3,$4) ON CONFLICT (date) DO NOTHING",
                batch_weather[i:i+BATCH],
            )
        print(f"    {len(batch_weather)} rows")

        print("  vet_records (mastitis)...")
        for i in range(0, len(batch_vet), BATCH):
            await conn.executemany(
                "INSERT INTO vet_records (animal_id, record_type, diagnosis_code, confirmed, event_date) SELECT $1,'disease','mastitis',true,$2 WHERE NOT EXISTS (SELECT 1 FROM vet_records WHERE animal_id=$1 AND diagnosis_code='mastitis' AND event_date=$2)",
                batch_vet[i:i+BATCH],
            )
        print(f"    {len(batch_vet)} rows")

        print("  weight_records...")
        for i in range(0, len(batch_weight), BATCH):
            await conn.executemany(
                "INSERT INTO weight_records (animal_id, weight_kg, measure_date) SELECT $1,$2,$3 WHERE NOT EXISTS (SELECT 1 FROM weight_records WHERE animal_id=$1 AND measure_date=$3)",
                batch_weight[i:i+BATCH],
            )
        print(f"    {len(batch_weight)} rows")

    # --- Activities (vectorized) ---
    print("Generating activities...")
    sorted_df = df_valid[["aid", "date_val"]].sort_values(["aid", "date_val"]).copy()
    sorted_df["counter"] = 300 + (sorted_df["aid"] % 100) + (sorted_df.groupby("aid").cumcount() * 3) + (sorted_df["aid"] % 7)
    act_dts = [datetime.datetime.combine(d, datetime.time(8, 0)) for d in sorted_df["date_val"].values]
    batch_act = list(zip(
        sorted_df["aid"].values,
        act_dts,
        sorted_df["counter"].values,
    ))

    async with pool.acquire() as conn:
        for i in range(0, len(batch_act), BATCH):
            await conn.executemany(
                "INSERT INTO activities (animal_id, activity_datetime, activity_counter, heat_attention) VALUES ($1,$2,$3,false) ON CONFLICT (animal_id, activity_datetime) DO NOTHING",
                batch_act[i:i+BATCH],
            )
    print(f"    {len(batch_act)} rows")

    # --- Milk quality (vectorized with numpy) ---
    print("Generating milk_quality...")
    rng = np.random.RandomState(42)
    milk_vals = df_valid["Milk_Yield_L"].values
    mastitis_flags = (df_valid["Mastitis"] == 1).values
    aids = df_valid["aid"].values
    dates = df_valid["date_val"].values

    fat = np.clip(3.8 - 0.005 * milk_vals + rng.randn(len(milk_vals)) * 0.3, 2.5, 6.0)
    protein = np.clip(3.2 - 0.003 * milk_vals + rng.randn(len(milk_vals)) * 0.2, 2.5, 4.5)
    lactose = np.clip(4.6 + rng.randn(len(milk_vals)) * 0.15, 3.5, 5.5)

    scc = np.where(mastitis_flags, rng.randint(400, 3000, len(milk_vals)), rng.randint(20, 250, len(milk_vals)))
    lactose = np.where(mastitis_flags, np.clip(lactose - 0.3, 3.0, 5.5), lactose)

    batch_mq = list(zip(
        aids,
        dates,
        fat.round(2),
        protein.round(2),
        lactose.round(2),
        (scc * 1000),
    ))

    async with pool.acquire() as conn:
        for i in range(0, len(batch_mq), BATCH):
            await conn.executemany(
                "INSERT INTO milk_quality (animal_id, date, fat_percentage, protein_percentage, lactose_percentage, scc, milkings, refusals) VALUES ($1,$2,$3,$4,$5,$6,2,0) ON CONFLICT (animal_id, date) DO NOTHING",
                batch_mq[i:i+BATCH],
            )
    print(f"    {len(batch_mq)} rows")

    await pool.close()
    print("Done! All data imported.")


if __name__ == "__main__":
    db_url = os.environ.get("DATABASE_URL", "postgres://milkfarm:milkfarm@localhost:5432/milkfarm")
    asyncio.run(run(db_url))
