-- Add migration script here
CREATE INDEX idx_adjuscent_location_id
ON location.adjuscent_locations(adjuscent_location_id);

