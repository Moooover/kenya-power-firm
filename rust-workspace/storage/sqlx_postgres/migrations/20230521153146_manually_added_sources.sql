-- Add migration script here
CREATE TABLE location.manually_added_sources (
    id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
    source_url VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL
);