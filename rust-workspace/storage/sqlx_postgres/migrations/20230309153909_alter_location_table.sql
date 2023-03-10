-- Add migration script here

ALTER TABLE location.locations DROP COLUMN textsearchable_index_col;


ALTER TABLE location.locations ADD COLUMN external_id TEXT NOT NULL;
ALTER TABLE location.locations ADD COLUMN address TEXT NOT NULL;
ALTER TABLE location.locations ADD COLUMN sanitized_address TEXT NOT NULL;


ALTER TABLE location.locations
    ADD COLUMN main_text_searcheable_index_col tsvector
               GENERATED ALWAYS AS (to_tsvector('english', name || ' ' || sanitized_address)) STORED;

CREATE INDEX main_text_searcheable_idx ON location.locations USING GIN (main_text_searcheable_index_col);


ALTER TABLE location.locations ADD COLUMN external_api_response JSONB NOT NULL;


DROP FUNCTION IF EXISTS location.search_locations;



ALTER TABLE location.locations
    ADD COLUMN secondary_text_searcheable_index_col tsvector
               GENERATED ALWAYS AS (to_tsvector('english', external_api_response)) STORED;

CREATE INDEX secondary_text_searcheable_idx ON location.locations USING GIN (secondary_text_searcheable_index_col);

CREATE OR REPLACE FUNCTION location.search_locations_primary_text(candidates text[]) RETURNS TABLE("like" types.location_name_and_search_query_with_id)
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
					WHERE main_text_searcheable_index_col @@ to_tsquery(candidate);
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


CREATE OR REPLACE FUNCTION location.search_locations_secondary_text(candidates text[]) RETURNS TABLE("like" types.location_name_and_search_query_with_id)
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
					WHERE secondary_text_searcheable_index_col @@ to_tsquery(candidate);
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
