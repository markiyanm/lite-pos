-- Replace printer_enabled + printer_prompt_before_print with single receipt_mode setting
-- Values: 'off' (no receipts), 'prompt' (show preview first), 'auto' (print automatically)
-- Migrate existing data: if printer_enabled=true and prompt=true → 'prompt',
--   if printer_enabled=true and prompt=false → 'auto', otherwise → 'off'
INSERT INTO settings (key, value, group_name)
  SELECT 'receipt_mode',
    CASE
      WHEN (SELECT value FROM settings WHERE key = 'printer_enabled') = 'true'
       AND (SELECT value FROM settings WHERE key = 'printer_prompt_before_print') = 'true'
        THEN 'prompt'
      WHEN (SELECT value FROM settings WHERE key = 'printer_enabled') = 'true'
        THEN 'auto'
      ELSE 'off'
    END,
    'printing'
  WHERE NOT EXISTS (SELECT 1 FROM settings WHERE key = 'receipt_mode');

-- Clean up old settings
DELETE FROM settings WHERE key IN ('printer_enabled', 'printer_prompt_before_print');
