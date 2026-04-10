CREATE TYPE gender_type AS ENUM ('male', 'female');

CREATE TYPE birth_remark_type AS ENUM (
    'normal',
    'abnormal_calf',
    'alive_premature_born',
    'abortion',
    'twin_calf_free_martin',
    'twin_calf_same_sex',
    'departed_auto_transfer_out'
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE locations (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    location_type TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE contacts (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    contact_type_id INTEGER,
    contact_type_name TEXT,
    farm_number TEXT,
    phone_cell TEXT,
    phone_home TEXT,
    phone_work TEXT,
    phone_fax TEXT,
    email TEXT,
    company_name TEXT,
    description TEXT,
    street_name TEXT,
    street_name_ext TEXT,
    postal_code TEXT,
    country_id INTEGER,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE animals (
    id SERIAL PRIMARY KEY,
    life_number TEXT,
    name TEXT,
    user_number BIGINT,
    gender gender_type NOT NULL,
    birth_date DATE NOT NULL,
    hair_color_code TEXT,
    father_life_number TEXT,
    mother_life_number TEXT,
    description TEXT,
    ucn_number TEXT,
    use_as_sire BOOLEAN,
    location TEXT,
    group_number INTEGER,
    keep BOOLEAN,
    gestation INTEGER,
    responder_number TEXT,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_animals_life_number ON animals (life_number);
CREATE INDEX idx_animals_ucn_number ON animals (ucn_number);
CREATE INDEX idx_animals_active ON animals (active);

CREATE TABLE sires (
    id SERIAL PRIMARY KEY,
    sire_code TEXT,
    life_number TEXT,
    name TEXT,
    active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE calvings (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    calving_date DATE NOT NULL,
    remarks TEXT,
    lac_number INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_calvings_animal_id ON calvings (animal_id);
CREATE INDEX idx_calvings_date ON calvings (calving_date);

CREATE TABLE calves (
    id SERIAL PRIMARY KEY,
    calving_id INTEGER NOT NULL REFERENCES calvings(id) ON DELETE CASCADE,
    life_number TEXT,
    gender gender_type NOT NULL,
    birth_remark birth_remark_type,
    keep BOOLEAN,
    weight DOUBLE PRECISION,
    born_dead BOOLEAN,
    animal_number BIGINT,
    calf_name TEXT,
    hair_color_code TEXT,
    born_dead_reason_id INTEGER
);

CREATE INDEX idx_calves_calving_id ON calves (calving_id);

CREATE TABLE inseminations (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    insemination_date DATE NOT NULL,
    sire_code TEXT,
    insemination_type TEXT,
    charge_number TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_inseminations_animal_id ON inseminations (animal_id);
CREATE INDEX idx_inseminations_date ON inseminations (insemination_date);

CREATE TABLE pregnancies (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    pregnancy_date DATE NOT NULL,
    pregnancy_type TEXT,
    insemination_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pregnancies_animal_id ON pregnancies (animal_id);

CREATE TABLE heats (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    heat_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_heats_animal_id ON heats (animal_id);
CREATE INDEX idx_heats_date ON heats (heat_date);

CREATE TABLE dry_offs (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    dry_off_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_dry_offs_animal_id ON dry_offs (animal_id);

CREATE TABLE milk_day_productions (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    date DATE NOT NULL,
    milk_amount DOUBLE PRECISION,
    avg_amount DOUBLE PRECISION,
    avg_weight DOUBLE PRECISION,
    isk DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_milk_dp_animal_id ON milk_day_productions (animal_id);
CREATE INDEX idx_milk_dp_date ON milk_day_productions (date);
CREATE UNIQUE INDEX idx_milk_dp_animal_date ON milk_day_productions (animal_id, date);

CREATE TABLE milk_visits (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    visit_datetime TIMESTAMPTZ NOT NULL,
    milk_amount DOUBLE PRECISION,
    duration_seconds INTEGER,
    milk_destination INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_milk_visits_animal_id ON milk_visits (animal_id);
CREATE INDEX idx_milk_visits_datetime ON milk_visits (visit_datetime);

CREATE TABLE milk_quality (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    date DATE NOT NULL,
    milk_amount DOUBLE PRECISION,
    avg_amount DOUBLE PRECISION,
    avg_weight DOUBLE PRECISION,
    isk DOUBLE PRECISION,
    fat_percentage DOUBLE PRECISION,
    protein_percentage DOUBLE PRECISION,
    lactose_percentage DOUBLE PRECISION,
    scc INTEGER,
    milkings INTEGER,
    refusals INTEGER,
    failures INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_milk_quality_animal_id ON milk_quality (animal_id);
CREATE INDEX idx_milk_quality_date ON milk_quality (date);

CREATE TABLE bulk_tank_tests (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL,
    fat DOUBLE PRECISION NOT NULL,
    protein DOUBLE PRECISION NOT NULL,
    lactose DOUBLE PRECISION,
    scc INTEGER,
    ffa DOUBLE PRECISION
);

CREATE INDEX idx_bulk_tank_date ON bulk_tank_tests (date);

CREATE TABLE feed_types (
    id SERIAL PRIMARY KEY,
    number_of_feed_type INTEGER NOT NULL,
    feed_type TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    dry_matter_percentage DOUBLE PRECISION NOT NULL,
    stock_attention_level INTEGER,
    price DOUBLE PRECISION NOT NULL
);

CREATE TABLE feed_groups (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    min_milk_yield DOUBLE PRECISION,
    max_milk_yield DOUBLE PRECISION,
    avg_milk_yield DOUBLE PRECISION,
    avg_milk_fat DOUBLE PRECISION,
    avg_milk_protein DOUBLE PRECISION,
    avg_weight DOUBLE PRECISION,
    max_robot_feed_types INTEGER,
    max_feed_intake_robot DOUBLE PRECISION,
    min_feed_intake_robot DOUBLE PRECISION,
    number_of_cows INTEGER
);

CREATE TABLE feed_day_amounts (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    feed_date DATE NOT NULL,
    feed_number INTEGER NOT NULL,
    total DOUBLE PRECISION NOT NULL,
    rest_feed INTEGER
);

CREATE INDEX idx_feed_da_animal_id ON feed_day_amounts (animal_id);
CREATE INDEX idx_feed_da_date ON feed_day_amounts (feed_date);

CREATE TABLE feed_visits (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    visit_datetime TIMESTAMPTZ NOT NULL,
    feed_number INTEGER,
    amount DOUBLE PRECISION,
    duration_seconds INTEGER
);

CREATE INDEX idx_feed_visits_animal_id ON feed_visits (animal_id);
CREATE INDEX idx_feed_visits_datetime ON feed_visits (visit_datetime);

CREATE TABLE activities (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    activity_datetime TIMESTAMPTZ NOT NULL,
    activity_counter INTEGER,
    heat_attention BOOLEAN
);

CREATE INDEX idx_activities_animal_id ON activities (animal_id);
CREATE INDEX idx_activities_datetime ON activities (activity_datetime);

CREATE TABLE ruminations (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    date DATE NOT NULL,
    eating_seconds INTEGER,
    rumination_minutes INTEGER
);

CREATE INDEX idx_ruminations_animal_id ON ruminations (animal_id);
CREATE INDEX idx_ruminations_date ON ruminations (date);

CREATE TABLE grazing_data (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL,
    animal_count INTEGER,
    pasture_time INTEGER,
    lactation_period TEXT
);

CREATE INDEX idx_grazing_date ON grazing_data (date);

CREATE TABLE transfers (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    transfer_date TIMESTAMPTZ NOT NULL,
    transfer_type TEXT NOT NULL,
    reason_id INTEGER,
    from_location TEXT,
    to_location TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transfers_animal_id ON transfers (animal_id);

CREATE TABLE bloodlines (
    id SERIAL PRIMARY KEY,
    animal_id INTEGER NOT NULL REFERENCES animals(id),
    blood_type_code TEXT NOT NULL,
    percentage DOUBLE PRECISION NOT NULL CHECK (percentage >= 0 AND percentage <= 100)
);

CREATE INDEX idx_bloodlines_animal_id ON bloodlines (animal_id);

CREATE TABLE sync_log (
    id SERIAL PRIMARY KEY,
    entity_type TEXT NOT NULL,
    last_synced_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'pending',
    records_synced BIGINT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
