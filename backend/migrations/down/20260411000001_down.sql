-- Rollback: 20260411000001_add_lely_support
DROP INDEX IF EXISTS idx_grazing_data_date;
ALTER TABLE grazing_data
    DROP COLUMN IF EXISTS tank_number,
    DROP COLUMN IF EXISTS cum_total_pasturetime,
    DROP COLUMN IF EXISTS cum_pasture_days,
    DROP COLUMN IF EXISTS sd_time_pasture,
    DROP COLUMN IF EXISTS grazing_day_yes_no,
    DROP COLUMN IF EXISTS percentage_in_pasture,
    DROP COLUMN IF EXISTS cows_14dil,
    DROP COLUMN IF EXISTS total_milking_cows;
DROP INDEX IF EXISTS idx_animals_responder_number;
DROP INDEX IF EXISTS idx_feed_visits_animal_dt;
DROP INDEX IF EXISTS idx_feed_da_animal_date_feed;
DROP INDEX IF EXISTS idx_ruminations_animal_date;
DROP INDEX IF EXISTS idx_activities_animal_dt;
DROP INDEX IF EXISTS idx_milk_visits_animal_dt;
DROP TABLE IF EXISTS lely_sync_state;
DROP TABLE IF EXISTS robot_milk_data;
DROP TABLE IF EXISTS milk_visit_quality;
