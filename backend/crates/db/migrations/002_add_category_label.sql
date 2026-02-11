-- Add optional label field to categories for human-friendly display names
ALTER TABLE categories ADD COLUMN label TEXT;
