-- Add iFields key setting for card-not-present payments
INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description) VALUES
    ('ifields_key', '', 'string', 'gateway', 'iFields public key for card-not-present payments (encrypted)');
