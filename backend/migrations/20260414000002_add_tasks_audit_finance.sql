CREATE TYPE task_status AS ENUM ('pending', 'in_progress', 'done', 'cancelled');
CREATE TYPE task_priority AS ENUM ('low', 'medium', 'high', 'urgent');
CREATE TYPE task_category AS ENUM (
    'health',
    'reproduction',
    'feeding',
    'maintenance',
    'administrative',
    'other'
);

CREATE TABLE tasks (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    category task_category NOT NULL DEFAULT 'other',
    priority task_priority NOT NULL DEFAULT 'medium',
    status task_status NOT NULL DEFAULT 'pending',
    animal_id INTEGER REFERENCES animals(id),
    due_date DATE,
    assigned_to TEXT,
    completed_at TIMESTAMPTZ,
    created_by INTEGER REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_status ON tasks (status);
CREATE INDEX idx_tasks_priority ON tasks (priority);
CREATE INDEX idx_tasks_category ON tasks (category);
CREATE INDEX idx_tasks_animal_id ON tasks (animal_id) WHERE animal_id IS NOT NULL;
CREATE INDEX idx_tasks_due_date ON tasks (due_date) WHERE due_date IS NOT NULL;
CREATE INDEX idx_tasks_created_by ON tasks (created_by);

CREATE TABLE audit_log (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_user ON audit_log (user_id);
CREATE INDEX idx_audit_log_entity ON audit_log (entity_type, entity_id);
CREATE INDEX idx_audit_log_created ON audit_log (created_at DESC);

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('income', 'expense')),
    category TEXT NOT NULL,
    amount NUMERIC(12,2) NOT NULL,
    description TEXT,
    transaction_date DATE NOT NULL,
    animal_id INTEGER REFERENCES animals(id),
    reference TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transactions_type ON transactions (transaction_type);
CREATE INDEX idx_transactions_category ON transactions (category);
CREATE INDEX idx_transactions_date ON transactions (transaction_date);
CREATE INDEX idx_transactions_animal_id ON transactions (animal_id) WHERE animal_id IS NOT NULL;
