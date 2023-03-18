-- Add migration script here
CREATE TABLE IF NOT EXISTS location.line_schedule (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  line_id uuid NOT NULL,
  schedule_id uuid,
  CONSTRAINT fk_line_id FOREIGN KEY (line_id) REFERENCES location.line(id),
  CONSTRAINT fk_schedule_id FOREIGN KEY (schedule_id) REFERENCES location.blackout_schedule(id)
);
