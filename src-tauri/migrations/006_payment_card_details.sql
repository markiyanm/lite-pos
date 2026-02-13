-- Add card transaction metadata to payments table
ALTER TABLE payments ADD COLUMN card_auth_code TEXT;
ALTER TABLE payments ADD COLUMN card_last_four TEXT;
ALTER TABLE payments ADD COLUMN card_type TEXT;
ALTER TABLE payments ADD COLUMN card_entry_mode TEXT;
ALTER TABLE payments ADD COLUMN gateway_ref_num TEXT;
ALTER TABLE payments ADD COLUMN gateway_response TEXT;
