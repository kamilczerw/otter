-- Add import provenance and free-form tags to transactions table
-- import_details stores a JSON-serialized ImportDetails struct, nullable
-- for transactions not created via an importer.
ALTER TABLE transactions ADD COLUMN import_details TEXT;

-- tags stores a JSON-serialized array of strings; defaults to an empty
-- array for existing rows and new transactions that don't specify tags.
ALTER TABLE transactions ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
