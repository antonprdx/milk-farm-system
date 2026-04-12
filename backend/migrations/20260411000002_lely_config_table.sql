-- Lely config stored in DB (UI-configurable)

CREATE TABLE lely_config (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    enabled BOOLEAN NOT NULL DEFAULT false,
    base_url TEXT NOT NULL DEFAULT '',
    username TEXT NOT NULL DEFAULT '',
    password_encrypted TEXT NOT NULL DEFAULT '',
    farm_key_encrypted TEXT NOT NULL DEFAULT '',
    sync_interval_secs BIGINT NOT NULL DEFAULT 300,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO lely_config (enabled, base_url, username, password_encrypted, farm_key_encrypted, sync_interval_secs)
VALUES (false, '', '', '', '', 300);
