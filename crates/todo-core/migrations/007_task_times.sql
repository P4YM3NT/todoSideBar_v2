ALTER TABLE tasks ADD COLUMN starts_at TEXT;
ALTER TABLE tasks ADD COLUMN ends_at TEXT;
-- Inclusive last day for all-day events; end day for timed events.
ALTER TABLE tasks ADD COLUMN ends_on TEXT;
