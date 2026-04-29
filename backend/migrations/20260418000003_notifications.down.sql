DROP TABLE IF EXISTS notification_rules;
DROP TABLE IF EXISTS notification_channels;
DELETE FROM system_settings WHERE key IN ('telegram_bot_token', 'vapid_public_key', 'vapid_private_key');
