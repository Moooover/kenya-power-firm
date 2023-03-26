-- Add migration script here

CREATE SCHEMA IF NOT EXISTS importer;


CREATE TABLE IF NOT EXISTS importer.processed_files (
    id VARCHAR PRIMARY KEY,
    processed_at TIMESTAMPTZ NOT NULL
);
