CREATE TABLE farm_zones (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    zone_type VARCHAR(30) NOT NULL CHECK (zone_type IN ('barn', 'pasture', 'milking_parlor', 'feed_area', 'hospital', 'other')),
    capacity INTEGER,
    position_data JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_farm_zones_type ON farm_zones (zone_type);
