-- Add migration script here

CREATE INDEX start_time_index ON location.blackout_schedule (start_time);

CREATE INDEX line_schedule_schedule_id ON location.line_schedule(schedule_id);

CREATE INDEX line_schedule_line_id ON location.line_schedule(line_id);
