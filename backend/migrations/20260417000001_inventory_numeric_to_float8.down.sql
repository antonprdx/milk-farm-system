ALTER TABLE inventory_transactions ALTER COLUMN quantity TYPE NUMERIC(12,3) USING quantity::NUMERIC(12,3);
ALTER TABLE inventory_items ALTER COLUMN cost_per_unit TYPE NUMERIC(12,2) USING cost_per_unit::NUMERIC(12,2);
ALTER TABLE inventory_items ALTER COLUMN min_quantity TYPE NUMERIC(12,3) USING min_quantity::NUMERIC(12,3);
ALTER TABLE inventory_items ALTER COLUMN quantity TYPE NUMERIC(12,3) USING quantity::NUMERIC(12,3);
