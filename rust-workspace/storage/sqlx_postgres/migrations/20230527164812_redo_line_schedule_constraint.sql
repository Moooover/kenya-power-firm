-- Add migration script here


ALTER TABLE location.line_schedule
drop CONSTRAINT fk_line_id;

ALTER TABLE location.line_schedule
drop CONSTRAINT fk_schedule_id;


ALTER TABLE location.line_schedule
ADD CONSTRAINT fk_line_id FOREIGN KEY (line_id) REFERENCES location.line(id) ON DELETE CASCADE;


ALTER TABLE location.line_schedule
ADD CONSTRAINT fk_schedule_id FOREIGN KEY (schedule_id) REFERENCES location.blackout_schedule(id) ON DELETE CASCADE;
