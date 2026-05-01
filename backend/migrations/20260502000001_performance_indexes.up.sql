DO $idempotent$
BEGIN
    -- Composite indexes for common LATERAL JOIN patterns
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_calvings_animal_date_desc') THEN
        CREATE INDEX idx_calvings_animal_date_desc ON calvings (animal_id, calving_date DESC);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_inseminations_animal_date_desc') THEN
        CREATE INDEX idx_inseminations_animal_date_desc ON inseminations (animal_id, insemination_date DESC);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_vet_records_animal_type_confirmed_date') THEN
        CREATE INDEX idx_vet_records_animal_type_confirmed_date ON vet_records (animal_id, record_type, confirmed, event_date);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_weight_records_animal_date_desc') THEN
        CREATE INDEX idx_weight_records_animal_date_desc ON weight_records (animal_id, measure_date DESC);
    END IF;
END;
$idempotent$;

-- TimescaleDB compression settings (idempotent)
ALTER TABLE feed_day_amounts SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'animal_id',
    timescaledb.compress_orderby = 'feed_date DESC'
);

ALTER TABLE activities SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'animal_id',
    timescaledb.compress_orderby = 'activity_datetime DESC'
);

ALTER TABLE heats SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'animal_id',
    timescaledb.compress_orderby = 'heat_date DESC'
);

-- Compression policies (skip if exists)
DO $idempotent$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'milk_day_productions' AND proc_name = 'policy_compression') THEN
        PERFORM add_compression_policy('milk_day_productions', INTERVAL '6 months');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'milk_quality' AND proc_name = 'policy_compression') THEN
        PERFORM add_compression_policy('milk_quality', INTERVAL '6 months');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'ruminations' AND proc_name = 'policy_compression') THEN
        PERFORM add_compression_policy('ruminations', INTERVAL '6 months');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'feed_day_amounts' AND proc_name = 'policy_compression') THEN
        PERFORM add_compression_policy('feed_day_amounts', INTERVAL '6 months');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'activities' AND proc_name = 'policy_compression') THEN
        PERFORM add_compression_policy('activities', INTERVAL '6 months');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'heats' AND proc_name = 'policy_compression') THEN
        PERFORM add_compression_policy('heats', INTERVAL '6 months');
    END IF;
END;
$idempotent$;

-- Continuous aggregates (skip if exist)
DO $idempotent$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.continuous_aggregates WHERE view_name = 'milk_daily_summary') THEN
        CREATE MATERIALIZED VIEW milk_daily_summary
        WITH (timescaledb.continuous) AS
        SELECT
            time_bucket('1 day', date) AS bucket,
            animal_id,
            AVG(milk_amount)::float8 AS avg_milk,
            SUM(milk_amount)::float8 AS total_milk,
            STDDEV(milk_amount)::float8 AS stddev_milk,
            COUNT(*)::int8 AS milkings
        FROM milk_day_productions
        GROUP BY bucket, animal_id
        WITH NO DATA;

        PERFORM add_continuous_aggregate_policy('milk_daily_summary',
            start_offset => INTERVAL '3 days',
            end_offset => INTERVAL '1 hour',
            schedule_interval => INTERVAL '1 hour');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.continuous_aggregates WHERE view_name = 'milk_quality_weekly') THEN
        CREATE MATERIALIZED VIEW milk_quality_weekly
        WITH (timescaledb.continuous) AS
        SELECT
            time_bucket('7 days', date) AS bucket,
            animal_id,
            AVG(scc)::float8 AS avg_scc,
            AVG(fat_percentage)::float8 AS avg_fat,
            AVG(protein_percentage)::float8 AS avg_protein,
            AVG(lactose_percentage)::float8 AS avg_lactose,
            AVG(fat_percentage / NULLIF(protein_percentage, 0))::float8 AS avg_fpr,
            COUNT(*)::int8 AS samples
        FROM milk_quality
        GROUP BY bucket, animal_id
        WITH NO DATA;

        PERFORM add_continuous_aggregate_policy('milk_quality_weekly',
            start_offset => INTERVAL '14 days',
            end_offset => INTERVAL '1 day',
            schedule_interval => INTERVAL '1 day');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.continuous_aggregates WHERE view_name = 'rumination_daily_summary') THEN
        CREATE MATERIALIZED VIEW rumination_daily_summary
        WITH (timescaledb.continuous) AS
        SELECT
            time_bucket('1 day', date) AS bucket,
            animal_id,
            AVG(rumination_minutes)::float8 AS avg_rumination,
            SUM(rumination_minutes)::float8 AS total_rumination,
            STDDEV(rumination_minutes)::float8 AS stddev_rumination,
            COUNT(*)::int8 AS records
        FROM ruminations
        GROUP BY bucket, animal_id
        WITH NO DATA;

        PERFORM add_continuous_aggregate_policy('rumination_daily_summary',
            start_offset => INTERVAL '3 days',
            end_offset => INTERVAL '1 hour',
            schedule_interval => INTERVAL '1 hour');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.continuous_aggregates WHERE view_name = 'feed_daily_summary') THEN
        CREATE MATERIALIZED VIEW feed_daily_summary
        WITH (timescaledb.continuous) AS
        SELECT
            time_bucket('1 day', feed_date) AS bucket,
            animal_id,
            AVG(total)::float8 AS avg_feed,
            SUM(total)::float8 AS total_feed,
            COUNT(DISTINCT feed_number)::int8 AS feedings
        FROM feed_day_amounts
        GROUP BY bucket, animal_id
        WITH NO DATA;

        PERFORM add_continuous_aggregate_policy('feed_daily_summary',
            start_offset => INTERVAL '3 days',
            end_offset => INTERVAL '1 hour',
            schedule_interval => INTERVAL '1 hour');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM timescaledb_information.continuous_aggregates WHERE view_name = 'activity_daily_summary') THEN
        CREATE MATERIALIZED VIEW activity_daily_summary
        WITH (timescaledb.continuous) AS
        SELECT
            time_bucket('1 day', activity_datetime) AS bucket,
            animal_id,
            AVG(activity_counter)::float8 AS avg_activity,
            STDDEV(activity_counter)::float8 AS stddev_activity,
            COUNT(*)::int8 AS records
        FROM activities
        GROUP BY bucket, animal_id
        WITH NO DATA;

        PERFORM add_continuous_aggregate_policy('activity_daily_summary',
            start_offset => INTERVAL '3 days',
            end_offset => INTERVAL '1 hour',
            schedule_interval => INTERVAL '1 hour');
    END IF;
END;
$idempotent$;

-- plan_cache_mode for sqlx prepared statements
ALTER DATABASE milkfarm SET plan_cache_mode = 'force_custom_plan';
