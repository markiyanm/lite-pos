-- Add Sola Gateway credit card processor settings
INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description) VALUES
    ('sola_gateway_card_present', 'false', 'boolean', 'gateway', 'Accept card-present transactions via terminal'),
    ('sola_gateway_card_not_present', 'false', 'boolean', 'gateway', 'Accept card-not-present (keyed) transactions'),
    ('sola_gateway_api_key', '', 'string', 'gateway', 'Sola Gateway API key (encrypted)');
