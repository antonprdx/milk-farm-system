DROP TABLE IF EXISTS culling_events;
ALTER TABLE heats DROP COLUMN IF EXISTS confirmation_method;
ALTER TABLE heats DROP COLUMN IF EXISTS confirmed;
ALTER TABLE vet_records DROP COLUMN IF EXISTS confirmed;
ALTER TABLE vet_records DROP COLUMN IF EXISTS diagnosis_code;
