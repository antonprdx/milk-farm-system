DO $$
BEGIN
    PERFORM 1 FROM pg_available_extensions WHERE name = 'timescaledb';
    IF NOT FOUND THEN
        RAISE NOTICE 'TimescaleDB not available, skipping continuous aggregates';
        RETURN;
    END IF;

    CREATE MATERIALIZED VIEW milk_daily_summary
    WITH (timescaledb.continuous) AS
    SELECT
        time_bucket('1 day', date) AS bucket,
        animal_id,
        AVG(milk_amount)::float8 AS avg_milk,
        SUM(milk_amount)::float8 AS total_milk
    FROM milk_day_productions
    GROUP BY bucket, animal_id
    WITH NO DATA;

    PERFORM add_continuous_aggregate_policy('milk_daily_summary',
        start_offset => INTERVAL '3 days',
        end_offset => INTERVAL '1 hour',
        schedule_interval => INTERVAL '1 hour');

    CREATE MATERIALIZED VIEW milk_quality_weekly
    WITH (timescaledb.continuous) AS
    SELECT
        time_bucket('7 days', date) AS bucket,
        animal_id,
        AVG(fat_percentage)::float8 AS avg_fat,
        AVG(protein_percentage)::float8 AS avg_protein,
        AVG(scc)::float8 AS avg_scc,
        AVG(milk_amount)::float8 AS avg_milk_amount
    FROM milk_quality
    GROUP BY bucket, animal_id
    WITH NO DATA;

    PERFORM add_continuous_aggregate_policy('milk_quality_weekly',
        start_offset => INTERVAL '21 days',
        end_offset => INTERVAL '1 day',
        schedule_interval => INTERVAL '1 day');

    CREATE MATERIALIZED VIEW rumination_daily_summary
    WITH (timescaledb.continuous) AS
    SELECT
        time_bucket('1 day', date) AS bucket,
        animal_id,
        AVG(rumination_minutes)::float8 AS avg_rumination,
        AVG(eating_seconds)::float8 AS avg_eating
    FROM ruminations
    GROUP BY bucket, animal_id
    WITH NO DATA;

    PERFORM add_continuous_aggregate_policy('rumination_daily_summary',
        start_offset => INTERVAL '3 days',
        end_offset => INTERVAL '1 hour',
        schedule_interval => INTERVAL '1 hour');

    PERFORM add_retention_policy('milk_visits', INTERVAL '2 years');
    PERFORM add_retention_policy('feed_visits', INTERVAL '2 years');
    PERFORM add_retention_policy('milk_visit_quality', INTERVAL '2 years');
    PERFORM add_retention_policy('robot_milk_data', INTERVAL '2 years');
    PERFORM add_retention_policy('activities', INTERVAL '2 years');

    RAISE NOTICE 'TimescaleDB continuous aggregates and retention policies created';
END;
$$;
