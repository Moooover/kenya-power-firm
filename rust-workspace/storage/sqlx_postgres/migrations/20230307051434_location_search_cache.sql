-- Add migration script here

CREATE TABLE IF NOT EXISTS location.location_search_cache (
  key TEXT PRIMARY KEY,
  value jsonb NOT NULL
);