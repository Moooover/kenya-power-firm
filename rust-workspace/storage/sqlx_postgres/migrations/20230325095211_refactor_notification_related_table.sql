-- Add migration script here

DROP TABLE IF EXISTS communication.notifications, communication.reason_for_notification;

CREATE TABLE IF NOT EXISTS communication.notifications (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  source_id uuid NOT NULL,
  directly_affected bool NOT NULL,
  subscriber_id uuid NOT NULL,
  line TEXT NOT NULL,
  strategy_id uuid NOT NULL,
  location_id_matched uuid NOT NULL,
  external_id TEXT NOT NULL,
  sent_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  CONSTRAINT fk_subscriber_id FOREIGN KEY (subscriber_id) REFERENCES public.subscriber(id),
  CONSTRAINT fk_source_id FOREIGN KEY (source_id) REFERENCES public.source(id),
  CONSTRAINT fk_strategy_id FOREIGN KEY (strategy_id) REFERENCES communication.strategies(id),
  CONSTRAINT fk_location_id_matched FOREIGN KEY (location_id_matched) REFERENCES location.locations(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_notification ON communication.notifications(source_id, directly_affected, subscriber_id, line, strategy_id);


INSERT INTO communication.strategies
    (name)
VALUES
    ('EMAIL');
