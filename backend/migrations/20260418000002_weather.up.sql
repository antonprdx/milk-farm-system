CREATE TABLE IF NOT EXISTS weather_cache (
    date DATE PRIMARY KEY,
    temp_c DOUBLE PRECISION,
    humidity DOUBLE PRECISION,
    precipitation_mm DOUBLE PRECISION,
    wind_speed DOUBLE PRECISION,
    weather_main TEXT,
    weather_icon TEXT,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO system_settings (key, value, updated_at) VALUES
    ('weather_lat', '55.75', NOW()),
    ('weather_lon', '37.62', NOW()),
    ('weather_api_key', '', NOW()),
    ('milk_price_per_liter', '30.0', NOW()),
    ('feed_cost_per_kg', '15.0', NOW())
ON CONFLICT (key) DO NOTHING;
