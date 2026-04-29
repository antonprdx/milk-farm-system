DROP MATERIALIZED VIEW IF EXISTS rumination_daily_summary;
DROP MATERIALIZED VIEW IF EXISTS milk_quality_weekly;
DROP MATERIALIZED VIEW IF EXISTS milk_daily_summary;

SELECT remove_retention_policy('activities');
SELECT remove_retention_policy('robot_milk_data');
SELECT remove_retention_policy('milk_visit_quality');
SELECT remove_retention_policy('feed_visits');
SELECT remove_retention_policy('milk_visits');

DROP EXTENSION IF EXISTS timescaledb;
