-- Add migration script here

ALTER TABLE public.subscriber RENAME COLUMN updated_at TO last_login;