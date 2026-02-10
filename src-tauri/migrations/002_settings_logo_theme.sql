-- Add store logo and theme settings
INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description)
VALUES
  ('store_logo', '', 'string', 'store', 'Store logo as base64 data URL'),
  ('theme', 'system', 'string', 'appearance', 'App theme: light, dark, or system');
