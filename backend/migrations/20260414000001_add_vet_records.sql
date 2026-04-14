CREATE TYPE vet_record_type AS ENUM (
    'vaccination',
    'treatment',
    'disease',
    'surgery',
    'deworming',
    'hoof_care',
    'examination',
    'other'
);

CREATE TYPE vet_record_status AS ENUM (
    'planned',
    'in_progress',
    'completed',
    'cancelled'
);

CREATE TABLE vet_records (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    record_type vet_record_type NOT NULL,
    status vet_record_status NOT NULL DEFAULT 'completed',
    event_date DATE NOT NULL,
    diagnosis TEXT,
    treatment TEXT,
    medication TEXT,
    dosage TEXT,
    withdrawal_days INTEGER,
    withdrawal_end_date DATE,
    veterinarian TEXT,
    notes TEXT,
    follow_up_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vet_records_animal_id ON vet_records (animal_id);
CREATE INDEX idx_vet_records_type ON vet_records (record_type);
CREATE INDEX idx_vet_records_status ON vet_records (status);
CREATE INDEX idx_vet_records_event_date ON vet_records (event_date);
CREATE INDEX idx_vet_records_follow_up ON vet_records (follow_up_date) WHERE follow_up_date IS NOT NULL;
CREATE INDEX idx_vet_records_withdrawal_end ON vet_records (withdrawal_end_date) WHERE withdrawal_end_date IS NOT NULL AND status = 'completed';

CREATE TABLE weight_records (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    weight_kg NUMERIC(8,2) NOT NULL,
    bcs NUMERIC(3,1),
    measure_date DATE NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_weight_records_animal_id ON weight_records (animal_id);
CREATE INDEX idx_weight_records_measure_date ON weight_records (measure_date);
