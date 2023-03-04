-- Add migration script here
-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA "public";
CREATE EXTENSION IF NOT EXISTS "pg_trgm" WITH SCHEMA "public";


-- public schema

CREATE TABLE IF NOT EXISTS public.subscriber (
    id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    external_id VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL
);


CREATE TABLE IF NOT EXISTS public.source (
    id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
    url VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL
);


---- location schema


CREATE SCHEMA IF NOT EXISTS location;

CREATE TABLE IF NOT EXISTS location.county (
    id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
    name VARCHAR NOT NULL unique
);

CREATE TABLE IF NOT EXISTS location.tag (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  name VARCHAR NOT NULL,
  created_by uuid, -- optional, we can have public tags
  CONSTRAINT fk_created_by FOREIGN KEY(created_by) REFERENCES public.subscriber(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_name_created_by ON location.tag(name, created_by);


CREATE TABLE IF NOT EXISTS location.area (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  name TEXT NOT NULL,
  county_id uuid NOT NULL,
  CONSTRAINT fk_area_id FOREIGN KEY (county_id) REFERENCES location.county(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_name_country_id ON location.area(name, county_id);


CREATE TABLE IF NOT EXISTS location.line (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  name TEXT NOT NULL,
  area_id uuid,
  CONSTRAINT fk_area_id FOREIGN KEY (area_id) REFERENCES location.area(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_name_area_id ON location.line(name, area_id);


CREATE TABLE IF NOT EXISTS location.locations (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS location.subscriber_locations (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  subscriber_id uuid NOT NULL,
  location_id uuid NOT NULL,
  CONSTRAINT fk_subscriber_id FOREIGN KEY (subscriber_id) REFERENCES public.subscriber(id),
  CONSTRAINT fk_location_id FOREIGN KEY (location_id) REFERENCES location.locations(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_location_and_subscriber ON location.subscriber_locations(subscriber_id, location_id);


CREATE TABLE IF NOT EXISTS location.adjuscent_locations (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  initial_location_id uuid,
  adjuscent_location_id uuid,
  CONSTRAINT fk_initial_location_id FOREIGN KEY (initial_location_id) REFERENCES location.subscriber_locations(id),
  CONSTRAINT fk_adjuscent_location_id FOREIGN KEY (adjuscent_location_id) REFERENCES location.locations(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_initial_location_with_adjuscent_location ON location.adjuscent_locations(initial_location_id, adjuscent_location_id);


CREATE TABLE IF NOT EXISTS location.blackout_schedule (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  area_id uuid,
  start_time TIMESTAMPTZ NOT NULL,
  end_time TIMESTAMPTZ NOT NULL,
  source_id uuid,
  CONSTRAINT fk_area_id FOREIGN KEY (area_id) REFERENCES location.area(id),
  CONSTRAINT fk_source_id FOREIGN KEY (source_id) REFERENCES public.source(id)
);


---- communication schema


CREATE SCHEMA IF NOT EXISTS communication;


CREATE TABLE IF NOT EXISTS communication.strategies (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS communication.subscriber_strategies (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  subscriber_id uuid NOT NULL,
  strategy_id uuid NOT NULL,
  CONSTRAINT fk_subscriber_id FOREIGN KEY(subscriber_id) REFERENCES public.subscriber(id),
  CONSTRAINT fk_strategy_id FOREIGN KEY (strategy_id) REFERENCES communication.strategies(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_subscriber_strategy ON communication.subscriber_strategies(subscriber_id, strategy_id);



CREATE TABLE IF NOT EXISTS communication.reason_for_notification (
   id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
   name TEXT NOT NULL ,-- for now, either directly_affected | potentially_affected
   description TEXT NOT NULL
);


CREATE TABLE IF NOT EXISTS communication.subscriber_email (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  subscriber_id uuid NOT NULL,
  email TEXT NOT NULL, -- TODO: potentially more custom fields for email, etc..
  CONSTRAINT fk_subscriber_id FOREIGN KEY (subscriber_id) REFERENCES public.subscriber(id)
);


CREATE UNIQUE INDEX IF NOT EXISTS idx_subscriber_id_email ON communication.subscriber_email(subscriber_id, email);


CREATE TABLE IF NOT EXISTS communication.notifications (
  id uuid PRIMARY KEY DEFAULT public.uuid_generate_v4(),
  schedule_id uuid NOT NULL,
  subscriber_id uuid NOT NULL,
  target TEXT NOT NULL,
  -- we should not lose track of the notification just because a subscriber has removed the strategy, this is why
  -- target does not reference any table.
  strategy_id uuid NOT NULL,
  location_id uuid NOT NULL,
  reason_id uuid NOT NULL,
  sent_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  CONSTRAINT fk_subscriber_id FOREIGN KEY (subscriber_id) REFERENCES public.subscriber(id),
  CONSTRAINT fk_schedule_id FOREIGN KEY (schedule_id) REFERENCES location.blackout_schedule(id),
  CONSTRAINT fk_strategy_id FOREIGN KEY (strategy_id) REFERENCES communication.strategies(id),
  CONSTRAINT fk_location_id FOREIGN KEY (location_id) REFERENCES location.locations(id),
  CONSTRAINT fk_reason_id FOREIGN KEY (reason_id) REFERENCES communication.reason_for_notification(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_notification ON communication.notifications(schedule_id, subscriber_id, target, strategy_id, location_id, reason_id);