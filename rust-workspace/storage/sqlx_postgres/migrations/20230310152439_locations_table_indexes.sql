-- Add migration script here

-- drop constraint on name
ALTER TABLE location.locations DROP CONSTRAINT locations_name_key;

-- add constraint on external_id
ALTER TABLE location.locations
ADD CONSTRAINT locations_external_id_index UNIQUE (external_id);
