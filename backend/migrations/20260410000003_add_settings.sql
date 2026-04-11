CREATE TABLE user_preferences (
    user_id INTEGER NOT NULL PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    theme TEXT NOT NULL DEFAULT 'light',
    page_size INTEGER NOT NULL DEFAULT 20,
    compact_view BOOLEAN NOT NULL DEFAULT false,
    language TEXT NOT NULL DEFAULT 'ru',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE system_settings (
    key TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO system_settings (key, value) VALUES
    ('jwt_access_ttl_secs', '900'),
    ('jwt_refresh_ttl_secs', '604800'),
    ('alert_min_milk', '5'),
    ('alert_max_scc', '400'),
    ('alert_days_before_calving', '14'),
    ('alert_activity_drop_pct', '30');
