-- Customer payment tokens for card-on-file feature
CREATE TABLE IF NOT EXISTS customer_payment_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE DEFAULT (lower(hex(randomblob(16)))),
    customer_id INTEGER NOT NULL REFERENCES customers(id),
    gateway_payment_method_id TEXT,
    token TEXT NOT NULL,
    card_last_four TEXT NOT NULL,
    card_brand TEXT,
    exp_month INTEGER,
    exp_year INTEGER,
    is_default INTEGER NOT NULL DEFAULT 0,
    gateway_revision INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_cpt_customer_id ON customer_payment_tokens(customer_id);

CREATE TRIGGER IF NOT EXISTS trg_customer_payment_tokens_updated_at
    AFTER UPDATE ON customer_payment_tokens
    FOR EACH ROW
BEGIN
    UPDATE customer_payment_tokens SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Card on file setting (default: off)
INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description)
VALUES ('enable_card_on_file', 'false', 'boolean', 'payments', 'Enable saving customer card tokens for future transactions');
