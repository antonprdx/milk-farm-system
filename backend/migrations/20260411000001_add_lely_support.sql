-- Lely Integration: new tables, dedup indexes, expanded grazing_data

-- ============================================================
-- New tables
-- ============================================================

CREATE TABLE milk_visit_quality (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    visit_datetime TIMESTAMPTZ NOT NULL,
    milking_start_date TIMESTAMPTZ,
    device_address INTEGER,
    success_milking BOOLEAN,
    milk_yield DOUBLE PRECISION,
    bottle_number INTEGER,
    milk_temperature DOUBLE PRECISION,
    weight INTEGER,
    milk_destination INTEGER,
    lf_colour_code TEXT,
    lr_colour_code TEXT,
    rf_colour_code TEXT,
    rr_colour_code TEXT,
    lf_conductivity INTEGER,
    lr_conductivity INTEGER,
    rf_conductivity INTEGER,
    rr_conductivity INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_mvq_animal_visit ON milk_visit_quality (animal_id, visit_datetime);
CREATE INDEX idx_mvq_datetime ON milk_visit_quality (visit_datetime);

CREATE TABLE robot_milk_data (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    milking_date TIMESTAMPTZ NOT NULL,
    device_address INTEGER,
    milk_speed DOUBLE PRECISION,
    milk_speed_max DOUBLE PRECISION,
    lf_milk_time INTEGER,
    lr_milk_time INTEGER,
    rf_milk_time INTEGER,
    rr_milk_time INTEGER,
    lf_dead_milk_time INTEGER,
    lr_dead_milk_time INTEGER,
    rf_dead_milk_time INTEGER,
    rr_dead_milk_time INTEGER,
    lf_x_position INTEGER,
    lf_y_position INTEGER,
    lf_z_position INTEGER,
    lr_x_position INTEGER,
    lr_y_position INTEGER,
    lr_z_position INTEGER,
    rf_x_position INTEGER,
    rf_y_position INTEGER,
    rf_z_position INTEGER,
    rr_x_position INTEGER,
    rr_y_position INTEGER,
    rr_z_position INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_rmd_animal_milking ON robot_milk_data (animal_id, milking_date);
CREATE INDEX idx_rmd_datetime ON robot_milk_data (milking_date);

CREATE TABLE lely_sync_state (
    id SERIAL PRIMARY KEY,
    entity_type TEXT NOT NULL UNIQUE,
    last_synced_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'pending',
    records_synced BIGINT NOT NULL DEFAULT 0,
    error_message TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- Deduplicate existing data before creating unique indexes
-- ============================================================

DELETE FROM milk_visits a USING milk_visits b
WHERE a.id < b.id AND a.animal_id = b.animal_id AND a.visit_datetime = b.visit_datetime;

DELETE FROM activities a USING activities b
WHERE a.id < b.id AND a.animal_id = b.animal_id AND a.activity_datetime = b.activity_datetime;

DELETE FROM ruminations a USING ruminations b
WHERE a.id < b.id AND a.animal_id = b.animal_id AND a.date = b.date;

DELETE FROM feed_day_amounts a USING feed_day_amounts b
WHERE a.id < b.id AND a.animal_id = b.animal_id AND a.feed_date = b.feed_date AND a.feed_number = b.feed_number;

DELETE FROM feed_visits a USING feed_visits b
WHERE a.id < b.id AND a.animal_id = b.animal_id AND a.visit_datetime = b.visit_datetime;

-- ============================================================
-- Dedup / upsert indexes on existing tables
-- ============================================================

CREATE UNIQUE INDEX idx_milk_visits_animal_dt ON milk_visits (animal_id, visit_datetime);
CREATE UNIQUE INDEX idx_activities_animal_dt ON activities (animal_id, activity_datetime);
CREATE UNIQUE INDEX idx_ruminations_animal_date ON ruminations (animal_id, date);
CREATE UNIQUE INDEX idx_feed_da_animal_date_feed ON feed_day_amounts (animal_id, feed_date, feed_number);
CREATE UNIQUE INDEX idx_feed_visits_animal_dt ON feed_visits (animal_id, visit_datetime);

CREATE INDEX idx_animals_responder_number ON animals (responder_number) WHERE responder_number IS NOT NULL;

-- ============================================================
-- Expand grazing_data with Lely fields
-- ============================================================

ALTER TABLE grazing_data
    ADD COLUMN IF NOT EXISTS total_milking_cows INTEGER,
    ADD COLUMN IF NOT EXISTS cows_14dil INTEGER,
    ADD COLUMN IF NOT EXISTS percentage_in_pasture DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS grazing_day_yes_no BOOLEAN,
    ADD COLUMN IF NOT EXISTS sd_time_pasture INTEGER,
    ADD COLUMN IF NOT EXISTS cum_pasture_days INTEGER,
    ADD COLUMN IF NOT EXISTS cum_total_pasturetime INTEGER,
    ADD COLUMN IF NOT EXISTS tank_number BIGINT;

CREATE UNIQUE INDEX idx_grazing_data_date ON grazing_data (date);

-- ============================================================
-- Seed lely_sync_state with all entity types
-- ============================================================

INSERT INTO lely_sync_state (entity_type, status) VALUES
    ('animals', 'pending'),
    ('milk_day_productions', 'pending'),
    ('milk_visits', 'pending'),
    ('milk_visit_quality', 'pending'),
    ('milk_day_productions_quality', 'pending'),
    ('robot_milk_data', 'pending'),
    ('feed_day_amounts', 'pending'),
    ('feed_visits', 'pending'),
    ('activities', 'pending'),
    ('ruminations', 'pending'),
    ('grazing_data', 'pending'),
    ('calvings', 'pending'),
    ('inseminations', 'pending'),
    ('pregnancies', 'pending'),
    ('heats', 'pending'),
    ('dry_offs', 'pending'),
    ('sires', 'pending'),
    ('transfers', 'pending'),
    ('bloodlines', 'pending'),
    ('feed_types', 'pending'),
    ('feed_groups', 'pending'),
    ('contacts', 'pending'),
    ('locations', 'pending')
ON CONFLICT (entity_type) DO NOTHING;
