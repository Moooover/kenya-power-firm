-- Add migration script here

ALTER TABLE location.locations
    ADD COLUMN textsearchable_index_col tsvector
               GENERATED ALWAYS AS (to_tsvector('english', name)) STORED;


CREATE INDEX text_search_idx ON location.locations USING GIN (textsearchable_index_col);


CREATE SCHEMA IF NOT EXISTS types;

DO $$ BEGIN
    CREATE TYPE types.location_name_and_search_query_with_id AS (
            search_query text,
            location text,
            id uuid
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE OR REPLACE FUNCTION location.search_locations(candidates text[]) RETURNS TABLE("like" types.location_name_and_search_query_with_id)
    LANGUAGE plpgsql
    AS $$

    DECLARE
        candidate                     TEXT;
        error_msg                     TEXT;

    BEGIN

        CREATE TEMP TABLE IF NOT EXISTS temp_table
        (
            LIKE types.location_name_and_search_query_with_id
        );

        FOREACH candidate IN ARRAY candidates
            LOOP
                BEGIN
                    INSERT INTO temp_table
                    SELECT candidate, name, id
                    FROM location.locations
					WHERE textsearchable_index_col @@ to_tsquery(candidate);
                EXCEPTION
                    WHEN OTHERS THEN
                        GET STACKED DIAGNOSTICS error_msg = MESSAGE_TEXT;
                        RAISE WARNING 'Something went wrong with candidate %: %', candidate, error_msg;
                END;
            END LOOP;

        RETURN QUERY SELECT * FROM temp_table;
        DROP TABLE temp_table;
    END;
$$;
