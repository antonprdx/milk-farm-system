-- Rollback: 20260410000005_add_pg_trgm
DROP INDEX IF EXISTS idx_animals_name_trgm;
DROP EXTENSION IF EXISTS pg_trgm;
