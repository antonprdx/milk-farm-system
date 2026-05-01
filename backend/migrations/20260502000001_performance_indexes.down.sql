-- Drop continuous aggregate policies and views
DROP MATERIALIZED VIEW IF EXISTS activity_daily_summary;
DROP MATERIALIZED VIEW IF EXISTS feed_daily_summary;
DROP MATERIALIZED VIEW IF EXISTS rumination_daily_summary;
DROP MATERIALIZED VIEW IF EXISTS milk_quality_weekly;
DROP MATERIALIZED VIEW IF EXISTS milk_daily_summary;

-- Remove compression policies
SELECT remove_compression_policy('milk_day_productions') WHERE EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'milk_day_productions' AND proc_name = 'policy_compression');
SELECT remove_compression_policy('milk_quality') WHERE EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'milk_quality' AND proc_name = 'policy_compression');
SELECT remove_compression_policy('ruminations') WHERE EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'ruminations' AND proc_name = 'policy_compression');
SELECT remove_compression_policy('feed_day_amounts') WHERE EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'feed_day_amounts' AND proc_name = 'policy_compression');
SELECT remove_compression_policy('activities') WHERE EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'activities' AND proc_name = 'policy_compression');
SELECT remove_compression_policy('heats') WHERE EXISTS (SELECT 1 FROM timescaledb_information.jobs WHERE hypertable_name = 'heats' AND proc_name = 'policy_compression');

-- Disable compression on tables we enabled it for
ALTER TABLE feed_day_amounts SET (timescaledb.compress = false);
ALTER TABLE activities SET (timescaledb.compress = false);
ALTER TABLE heats SET (timescaledb.compress = false);

-- Drop indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_calvings_animal_date_desc;
DROP INDEX CONCURRENTLY IF EXISTS idx_inseminations_animal_date_desc;
DROP INDEX CONCURRENTLY IF EXISTS idx_vet_records_animal_type_confirmed_date;
DROP INDEX CONCURRENTLY IF EXISTS idx_weight_records_animal_date_desc;
