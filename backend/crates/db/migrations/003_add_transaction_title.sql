-- Add nullable title column to transactions table
-- Max length: 50 characters (configurable via backend constant)
ALTER TABLE transactions ADD COLUMN title TEXT;

-- No index needed - title is not used for filtering/searching in Phase 1
