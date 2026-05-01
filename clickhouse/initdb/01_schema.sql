CREATE DATABASE IF NOT EXISTS milkfarm_analytics;

CREATE TABLE IF NOT EXISTS milkfarm_analytics.milk_day_productions
(
    date Date,
    animal_id Int32,
    milk_amount Float64,
    milking_count UInt32 DEFAULT 1
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (animal_id, date);

CREATE TABLE IF NOT EXISTS milkfarm_analytics.milk_quality
(
    date Date,
    animal_id Int32,
    scc Float64,
    fat_percentage Float64,
    protein_percentage Float64,
    lactose_percentage Float64
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (animal_id, date);

CREATE TABLE IF NOT EXISTS milkfarm_analytics.ruminations
(
    date Date,
    animal_id Int32,
    rumination_minutes Float64
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (animal_id, date);

CREATE TABLE IF NOT EXISTS milkfarm_analytics.feed_day_amounts
(
    date Date,
    animal_id Int32,
    total Float64,
    feed_number UInt32 DEFAULT 1
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (animal_id, date);

CREATE TABLE IF NOT EXISTS milkfarm_analytics.activities
(
    date Date,
    animal_id Int32,
    activity_counter Float64
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (animal_id, date);

CREATE TABLE IF NOT EXISTS milkfarm_analytics.heats
(
    date Date,
    animal_id Int32,
    confirmed UInt8 DEFAULT 0
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (animal_id, date);

CREATE MATERIALIZED VIEW IF NOT EXISTS milkfarm_analytics.mv_mastitis_features
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(bucket)
ORDER BY (animal_id, bucket)
AS
SELECT
    animal_id,
    date AS bucket,
    avgState(milk_amount) as avg_milk,
    countState() as milk_count,
    avgState(rumination_minutes) as avg_rumination,
    avgState(activity_counter) as avg_activity,
    avgState(scc) as avg_scc,
    avgState(fat_percentage) as avg_fat,
    avgState(protein_percentage) as avg_protein,
    avgState(lactose_percentage) as avg_lactose
FROM milkfarm_analytics.milk_day_productions m
LEFT JOIN milkfarm_analytics.ruminations r ON m.animal_id = r.animal_id AND m.date = r.date
LEFT JOIN milkfarm_analytics.activities a ON m.animal_id = a.animal_id AND m.date = a.date
LEFT JOIN milkfarm_analytics.milk_quality q ON m.animal_id = q.animal_id AND m.date = q.date
GROUP BY animal_id, date;
