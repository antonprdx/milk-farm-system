-- Rollback: 20260412000001_add_missing_indexes
DROP INDEX IF EXISTS idx_transfers_type;
DROP INDEX IF EXISTS idx_transfers_date;
DROP INDEX IF EXISTS idx_sires_life_number;
DROP INDEX IF EXISTS idx_sires_sire_code;
DROP INDEX IF EXISTS idx_contacts_contact_type_id;
DROP INDEX IF EXISTS idx_contacts_name;
DROP INDEX IF EXISTS idx_contacts_active;
