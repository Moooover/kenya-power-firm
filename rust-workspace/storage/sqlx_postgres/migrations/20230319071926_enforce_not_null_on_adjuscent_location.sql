-- Add migration script here

ALTER TABLE location.adjuscent_locations ALTER COLUMN adjuscent_location_id SET NOT NULL;
ALTER TABLE location.adjuscent_locations ALTER COLUMN initial_location_id SET NOT NULL;
