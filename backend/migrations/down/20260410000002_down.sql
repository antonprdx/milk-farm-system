-- Rollback: 20260410000002_add_performance_indexes
DROP INDEX IF EXISTS idx_animals_active_gender;
DROP INDEX IF EXISTS idx_feed_da_animal_date;
DROP INDEX IF EXISTS idx_dry_offs_date;
DROP INDEX IF EXISTS idx_pregnancies_date;
