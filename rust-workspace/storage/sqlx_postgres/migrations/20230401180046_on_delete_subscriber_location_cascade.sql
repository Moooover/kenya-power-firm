-- Add migration script here

ALTER TABLE location.adjuscent_locations
DROP CONSTRAINT "fk_initial_location_id",
ADD FOREIGN KEY (initial_location_id)
  REFERENCES location.subscriber_locations(id)
  ON DELETE CASCADE;
