-- Add gateway sync columns to customers table
ALTER TABLE customers ADD COLUMN gateway_customer_id TEXT;
ALTER TABLE customers ADD COLUMN gateway_sync_status TEXT NOT NULL DEFAULT 'unsynced' CHECK (gateway_sync_status IN ('unsynced', 'synced', 'pending', 'error', 'archived', 'orphaned'));
ALTER TABLE customers ADD COLUMN gateway_synced_at TEXT;
ALTER TABLE customers ADD COLUMN gateway_revision INTEGER;

-- Sync log table for tracking sync operations
CREATE TABLE IF NOT EXISTS sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL DEFAULT 'customer',
    entity_id INTEGER NOT NULL,
    direction TEXT NOT NULL CHECK (direction IN ('push', 'pull')),
    action TEXT NOT NULL CHECK (action IN ('create', 'update', 'delete')),
    status TEXT NOT NULL CHECK (status IN ('success', 'error')),
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Seed gateway sync settings
INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description) VALUES
    ('gateway_customer_sync_enabled', 'true', 'boolean', 'payments', 'Enable automatic customer sync with Sola gateway'),
    ('gateway_sync_interval_minutes', '15', 'integer', 'payments', 'Interval in minutes between automatic syncs (5-1440)');
