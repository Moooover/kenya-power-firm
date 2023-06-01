-- Add migration script here
CREATE INDEX nearby_locations_searcheable_idx ON location.nearby_locations USING GIN (searcheable_response);
