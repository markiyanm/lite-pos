-- Add default device ID setting for Sola Gateway
INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description) VALUES
    ('sola_gateway_default_device_id', '', 'string', 'gateway', 'Default Sola Gateway device ID for card-present transactions');
