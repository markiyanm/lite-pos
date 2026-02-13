-- Add color theme setting (separate from light/dark mode)
INSERT OR IGNORE INTO settings (key, value, value_type, group_name, description)
VALUES
  ('color_theme', 'blue', 'string', 'appearance', 'App color scheme: blue, blueYellow, goldBrown, default');
