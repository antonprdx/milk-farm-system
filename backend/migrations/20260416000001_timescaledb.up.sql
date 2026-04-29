DO $$
BEGIN
    PERFORM 1 FROM pg_available_extensions WHERE name = 'timescaledb';
    IF FOUND THEN
        CREATE EXTENSION IF NOT EXISTS timescaledb;

        ALTER TABLE milk_day_productions DROP CONSTRAINT milk_day_productions_pkey;
        ALTER TABLE milk_day_productions ADD PRIMARY KEY (id, date);
        PERFORM create_hypertable('milk_day_productions', 'date', chunk_time_interval => INTERVAL '7 days', migrate_data => true);

        ALTER TABLE milk_quality DROP CONSTRAINT milk_quality_pkey;
        ALTER TABLE milk_quality ADD PRIMARY KEY (id, date);
        PERFORM create_hypertable('milk_quality', 'date', chunk_time_interval => INTERVAL '7 days', migrate_data => true);

        ALTER TABLE ruminations DROP CONSTRAINT ruminations_pkey;
        DROP INDEX IF EXISTS idx_ruminations_animal_date;
        ALTER TABLE ruminations ADD PRIMARY KEY (id, date);
        CREATE UNIQUE INDEX idx_ruminations_animal_date ON ruminations (animal_id, date);
        PERFORM create_hypertable('ruminations', 'date', chunk_time_interval => INTERVAL '7 days', migrate_data => true);

        ALTER TABLE feed_day_amounts DROP CONSTRAINT feed_day_amounts_pkey;
        DROP INDEX IF EXISTS idx_feed_da_animal_date_feed;
        ALTER TABLE feed_day_amounts ADD PRIMARY KEY (id, feed_date);
        CREATE UNIQUE INDEX idx_feed_da_animal_date_feed ON feed_day_amounts (animal_id, feed_date, feed_number);
        PERFORM create_hypertable('feed_day_amounts', 'feed_date', chunk_time_interval => INTERVAL '7 days', migrate_data => true);

        ALTER TABLE milk_visits DROP CONSTRAINT milk_visits_pkey;
        DROP INDEX IF EXISTS idx_milk_visits_animal_dt;
        ALTER TABLE milk_visits ADD PRIMARY KEY (id, visit_datetime);
        CREATE UNIQUE INDEX idx_milk_visits_animal_dt ON milk_visits (animal_id, visit_datetime);
        PERFORM create_hypertable('milk_visits', 'visit_datetime', chunk_time_interval => INTERVAL '7 days', migrate_data => true);

        ALTER TABLE feed_visits DROP CONSTRAINT feed_visits_pkey;
        DROP INDEX IF EXISTS idx_feed_visits_animal_dt;
        ALTER TABLE feed_visits ADD PRIMARY KEY (id, visit_datetime);
        CREATE UNIQUE INDEX idx_feed_visits_animal_dt ON feed_visits (animal_id, visit_datetime);
        PERFORM create_hypertable('feed_visits', 'visit_datetime', chunk_time_interval => INTERVAL '7 days', migrate_data => true);

        ALTER TABLE activities DROP CONSTRAINT activities_pkey;
        DROP INDEX IF EXISTS idx_activities_animal_dt;
        ALTER TABLE activities ADD PRIMARY KEY (id, activity_datetime);
        CREATE UNIQUE INDEX idx_activities_animal_dt ON activities (animal_id, activity_datetime);
        PERFORM create_hypertable('activities', 'activity_datetime', chunk_time_interval => INTERVAL '7 days', migrate_data => true);

        ALTER TABLE milk_visit_quality DROP CONSTRAINT milk_visit_quality_pkey;
        DROP INDEX IF EXISTS idx_mvq_animal_visit;
        ALTER TABLE milk_visit_quality ADD PRIMARY KEY (id, visit_datetime);
        CREATE UNIQUE INDEX idx_mvq_animal_visit ON milk_visit_quality (animal_id, visit_datetime);
        PERFORM create_hypertable('milk_visit_quality', 'visit_datetime', chunk_time_interval => INTERVAL '7 days', migrate_data => true);

        ALTER TABLE robot_milk_data DROP CONSTRAINT robot_milk_data_pkey;
        DROP INDEX IF EXISTS idx_rmd_animal_milking;
        ALTER TABLE robot_milk_data ADD PRIMARY KEY (id, milking_date);
        CREATE UNIQUE INDEX idx_rmd_animal_milking ON robot_milk_data (animal_id, milking_date);
        PERFORM create_hypertable('robot_milk_data', 'milking_date', chunk_time_interval => INTERVAL '7 days', migrate_data => true);

        RAISE NOTICE 'TimescaleDB hypertables created';
    ELSE
        RAISE NOTICE 'TimescaleDB not available, skipping hypertable creation';
    END IF;
END;
$$;
