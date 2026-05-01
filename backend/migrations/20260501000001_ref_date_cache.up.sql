CREATE TABLE IF NOT EXISTS ref_date_cache (
    id int PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    ref_date date NOT NULL
);

INSERT INTO ref_date_cache (id, ref_date)
VALUES (1, COALESCE((SELECT MAX(date) FROM milk_day_productions), CURRENT_DATE))
ON CONFLICT (id) DO UPDATE SET ref_date = EXCLUDED.ref_date;

CREATE OR REPLACE FUNCTION fn_update_ref_date() RETURNS trigger AS $$
BEGIN
    UPDATE ref_date_cache SET ref_date = COALESCE((SELECT MAX(date) FROM milk_day_productions), CURRENT_DATE) WHERE id = 1;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_update_ref_date ON milk_day_productions;
CREATE TRIGGER trg_update_ref_date
    AFTER INSERT OR DELETE OR UPDATE ON milk_day_productions
    FOR EACH STATEMENT EXECUTE FUNCTION fn_update_ref_date();
