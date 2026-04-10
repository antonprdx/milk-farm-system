CREATE INDEX IF NOT EXISTS idx_pregnancies_date ON pregnancies (pregnancy_date);
CREATE INDEX IF NOT EXISTS idx_dry_offs_date ON dry_offs (dry_off_date);
CREATE INDEX IF NOT EXISTS idx_feed_da_animal_date ON feed_day_amounts (animal_id, feed_date);
CREATE INDEX IF NOT EXISTS idx_animals_active_gender ON animals (active, gender);
