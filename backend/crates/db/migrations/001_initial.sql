-- Categories
CREATE TABLE categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TRIGGER trg_categories_updated_at
BEFORE UPDATE ON categories
BEGIN
    UPDATE categories SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = NEW.id;
END;

-- Months
CREATE TABLE months (
    id TEXT PRIMARY KEY,
    month TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TRIGGER trg_months_updated_at
BEFORE UPDATE ON months
BEGIN
    UPDATE months SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = NEW.id;
END;

-- Budget Entries
CREATE TABLE budget_entries (
    id TEXT PRIMARY KEY,
    month_id TEXT NOT NULL REFERENCES months(id) ON DELETE RESTRICT,
    category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    budgeted INTEGER NOT NULL,
    due_day INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(month_id, category_id)
);

CREATE INDEX idx_budget_entries_month_id ON budget_entries(month_id);
CREATE INDEX idx_budget_entries_category_id ON budget_entries(category_id);

CREATE TRIGGER trg_budget_entries_updated_at
BEFORE UPDATE ON budget_entries
BEGIN
    UPDATE budget_entries SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = NEW.id;
END;

-- Transactions
CREATE TABLE transactions (
    id TEXT PRIMARY KEY,
    entry_id TEXT NOT NULL REFERENCES budget_entries(id) ON DELETE RESTRICT,
    amount INTEGER NOT NULL,
    date TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_transactions_entry_id ON transactions(entry_id);

CREATE TRIGGER trg_transactions_updated_at
BEFORE UPDATE ON transactions
BEGIN
    UPDATE transactions SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = NEW.id;
END;
