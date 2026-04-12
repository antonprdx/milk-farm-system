DELETE FROM milk_quality a USING milk_quality b
WHERE a.id > b.id
  AND a.animal_id = b.animal_id
  AND a.date = b.date;

CREATE UNIQUE INDEX idx_milk_quality_animal_date ON milk_quality (animal_id, date);
