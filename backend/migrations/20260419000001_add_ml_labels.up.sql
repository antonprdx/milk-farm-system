-- Add structured diagnosis code and confirmation flag for vet records (ML training labels)
ALTER TABLE vet_records
    ADD COLUMN IF NOT EXISTS diagnosis_code VARCHAR(50),
    ADD COLUMN IF NOT EXISTS confirmed BOOLEAN NOT NULL DEFAULT false;

-- Add confirmation fields for heats (ML estrus detection labels)
ALTER TABLE heats
    ADD COLUMN IF NOT EXISTS confirmed BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS confirmation_method VARCHAR(30);

-- Create culling events table (ML culling model labels)
CREATE TABLE IF NOT EXISTS culling_events (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    culling_date DATE NOT NULL,
    reason VARCHAR(50) NOT NULL,
    details JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_culling_events_animal_id ON culling_events(animal_id);
CREATE INDEX IF NOT EXISTS idx_culling_events_date ON culling_events(culling_date);
CREATE INDEX IF NOT EXISTS idx_vet_records_diagnosis_code ON vet_records(diagnosis_code) WHERE diagnosis_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_vet_records_confirmed ON vet_records(confirmed) WHERE confirmed = true;
CREATE INDEX IF NOT EXISTS idx_heats_confirmed ON heats(confirmed) WHERE confirmed = true;
