-- Add migration script here

DROP FUNCTION location.search_nearby_locations_with_nearby_location_id;

DO $$ BEGIN
    CREATE TYPE types.nearby_location_type AS (
            candidate text,
            location_id uuid
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE OR REPLACE FUNCTION location.search_nearby_locations_with_nearby_location_id(candidates text[], nearby_location uuid)
 RETURNS TABLE("like" types.nearby_location_type)
 LANGUAGE plpgsql
AS $function$

    DECLARE
        candidate                     TEXT;
        error_msg                     TEXT;

    BEGIN

        CREATE TEMP TABLE IF NOT EXISTS temp_table
        (
 			LIKE types.nearby_location_type
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
$function$;

DROP FUNCTION location.search_nearby_locations;

CREATE OR REPLACE FUNCTION location.search_nearby_locations(candidates text[]) RETURNS TABLE("like" types.nearby_location_type)
    LANGUAGE plpgsql
    AS $$

    DECLARE
        candidate                     TEXT;
        error_msg                     TEXT;

    BEGIN

        CREATE TEMP TABLE IF NOT EXISTS temp_table
        (
          LIKE types.nearby_location_type
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