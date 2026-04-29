CREATE TYPE alert_severity AS ENUM ('info', 'warning', 'critical');
CREATE TYPE alert_status AS ENUM ('active', 'acknowledged', 'resolved');
CREATE TYPE alert_category AS ENUM (
    'milk_drop',
    'high_scc',
    'activity_drop',
    'low_feed',
    'no_milking',
    'ketosis_risk',
    'mastitis_risk',
    'expected_calving',
    'equipment_anomaly',
    'other'
);

CREATE TABLE alerts (
    id SERIAL PRIMARY KEY,
    category alert_category NOT NULL,
    severity alert_severity NOT NULL DEFAULT 'warning',
    status alert_status NOT NULL DEFAULT 'active',
    animal_id INTEGER REFERENCES animals(id) ON DELETE SET NULL,
    message TEXT NOT NULL,
    details JSONB,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alerts_status ON alerts (status);
CREATE INDEX idx_alerts_animal ON alerts (animal_id);
CREATE INDEX idx_alerts_category ON alerts (category);
CREATE INDEX idx_alerts_detected_at ON alerts (detected_at DESC);
CREATE UNIQUE INDEX idx_alerts_active_unique ON alerts (category, animal_id) WHERE status = 'active';
