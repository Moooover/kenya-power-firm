-- Add migration script here

CREATE INDEX IF NOT EXISTS start_time_index ON location.blackout_schedule (start_time);

CREATE INDEX IF NOT EXISTS line_schedule_schedule_id ON location.line_schedule(schedule_id);

CREATE INDEX IF NOT EXISTS line_schedule_line_id ON location.line_schedule(line_id);
