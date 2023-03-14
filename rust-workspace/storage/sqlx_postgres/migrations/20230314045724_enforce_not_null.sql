-- Add migration script here
ALTER TABLE location.blackout_schedule ALTER COLUMN area_id SET NOT NULL;

ALTER TABLE location.blackout_schedule ALTER COLUMN source_id SET NOT NULL;
