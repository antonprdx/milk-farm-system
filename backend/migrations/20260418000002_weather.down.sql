DROP TABLE IF EXISTS weather_cache;
DELETE FROM system_settings WHERE key IN ('weather_lat', 'weather_lon', 'weather_api_key', 'milk_price_per_liter', 'feed_cost_per_kg');
