-- Add migration script here
CREATE INDEX IF NOT EXISTS nearby_locations_location_idx ON location.nearby_locations(location_id);