CREATE INDEX IF NOT EXISTS idx_contacts_active ON contacts (active);
CREATE INDEX IF NOT EXISTS idx_contacts_name ON contacts (name);
CREATE INDEX IF NOT EXISTS idx_contacts_contact_type_id ON contacts (contact_type_id) WHERE contact_type_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_sires_sire_code ON sires (sire_code) WHERE sire_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_sires_life_number ON sires (life_number) WHERE life_number IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_transfers_date ON transfers (transfer_date);
CREATE INDEX IF NOT EXISTS idx_transfers_type ON transfers (transfer_type);
