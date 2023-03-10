-- Add migration script here
ALTER TABLE location.location_search_cache
ADD created_at TIMESTAMPTZ DEFAULT now() NOT NULL;
