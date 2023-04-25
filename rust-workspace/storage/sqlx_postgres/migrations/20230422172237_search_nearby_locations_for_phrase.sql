-- Add migration script here

CREATE OR REPLACE FUNCTION location.search_nearby_locations_with_nearby_location_id(candidates text[], nearby_location uuid)     RETURNS TABLE
            (
                candidate text,
                location_id uuid
            )
    LANGUAGE plpgsql
    AS $$

    DECLARE
        candidate                     TEXT;
        error_msg                     TEXT;

    BEGIN

        CREATE TEMP TABLE IF NOT EXISTS temp_table
        (
                candidate text,
                location_id uuid
        );

        FOREACH candidate IN ARRAY candidates
            LOOP
                BEGIN
                    INSERT INTO temp_table
                    SELECT candidate, location_id
                    FROM location.nearby_locations
					WHERE searcheable_response @@ to_tsquery(candidate) AND id = nearby_location;
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



CREATE OR REPLACE FUNCTION location.search_nearby_locations(candidates text[])     RETURNS TABLE
            (
                candidate text,
                location_id uuid
            )
    LANGUAGE plpgsql
    AS $$

    DECLARE
        candidate                     TEXT;
        error_msg                     TEXT;

    BEGIN

        CREATE TEMP TABLE IF NOT EXISTS temp_table
        (
                candidate text,
                location_id uuid
        );

        FOREACH candidate IN ARRAY candidates
            LOOP
                BEGIN
                    INSERT INTO temp_table
                    SELECT candidate, location_id
                    FROM location.nearby_locations
					WHERE searcheable_response @@ to_tsquery(candidate);
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