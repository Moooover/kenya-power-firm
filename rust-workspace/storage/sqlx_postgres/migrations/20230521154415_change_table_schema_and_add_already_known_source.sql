-- Add migration script here

ALTER TABLE location.manually_added_sources
    SET SCHEMA public;


-- INSERT THE ALREADY PROBLEMATIC SOURCE;
INSERT INTO manually_added_sources
    (source_url)
VALUES
    ('https://www.kplc.co.ke/img/full/Interruption%20Notices%20-%2018.05.2022.pdf') ON CONFLICT DO NOTHING;