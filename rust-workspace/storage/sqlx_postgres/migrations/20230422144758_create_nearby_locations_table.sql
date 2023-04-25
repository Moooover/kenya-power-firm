-- Add migration script here

CREATE TABLE location.nearby_locations (
    id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
    source_url VARCHAR NOT NULL UNIQUE,
    location_id uuid NOT NULL,
    response jsonb NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    searcheable_response tsvector
               GENERATED ALWAYS AS (to_tsvector('english', response)) STORED,
    CONSTRAINT fk_location_id FOREIGN KEY (location_id) REFERENCES location.locations(id)
)