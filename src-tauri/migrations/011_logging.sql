-- Logging / Diagnostics settings
INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description)
VALUES ('enable_logging', 'true', 'boolean', 'diagnostics', 'Enable application event logging');

INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description)
VALUES ('log_retention_days', '30', 'integer', 'diagnostics', 'Number of days to retain log files (7-90)');

INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description)
VALUES ('log_level', 'info', 'string', 'diagnostics', 'Minimum log level to record (error, warn, info, debug)');
