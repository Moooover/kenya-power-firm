-- Add migration script here
CREATE INDEX idx_location_name ON location.area USING btree (name);
