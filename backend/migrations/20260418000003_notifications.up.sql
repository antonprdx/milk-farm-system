CREATE TABLE notification_channels (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_type VARCHAR(20) NOT NULL CHECK (channel_type IN ('browser', 'telegram')),
    channel_token TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE notification_rules (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    channel_id INTEGER REFERENCES notification_channels(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_notification_channels_user_type ON notification_channels (user_id, channel_type);
CREATE INDEX idx_notification_rules_user ON notification_rules (user_id);

INSERT INTO system_settings (key, value, updated_at) VALUES
    ('telegram_bot_token', '', NOW()),
    ('vapid_public_key', '', NOW()),
    ('vapid_private_key', '', NOW())
ON CONFLICT (key) DO NOTHING;
