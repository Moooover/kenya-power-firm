-- Add migration script here
ALTER TABLE importer.processed_files ALTER COLUMN processed_at SET DEFAULT now();
