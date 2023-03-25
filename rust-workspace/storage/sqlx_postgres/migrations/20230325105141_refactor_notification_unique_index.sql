-- Add migration script here

DROP INDEX IF EXISTS idx_unique_notification;

CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_notification ON communication.notifications(source_id, subscriber_id, line, strategy_id);
