-- Add migration script here

CREATE OR REPLACE FUNCTION location.search_nearby_locations_with_nearby_location_id_and_area_name(candidates text[], nearby_location uuid, area_name text)
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
					WHERE searcheable_response @@ to_tsquery(candidate) AND id = nearby_location AND searcheable_response @@ to_tsquery(area_name);
                EXCEPTION
                    WHEN OTHERS THEN
                        GET STACKED DIAGNOSTICS error_msg = MESSAGE_TEXT;
                        RAISE WARNING 'Something went wrong with search_nearby_locations_with_nearby_location_id_and_area_name candidate = %: error_msg = %; area_name = % ', candidate, error_msg, area_name;
                END;
            END LOOP;

        RETURN QUERY SELECT * FROM temp_table;
        DROP TABLE temp_table;
    END;
$function$;


CREATE OR REPLACE FUNCTION location.search_nearby_locations_with_area_name(candidates text[], area_name text) RETURNS TABLE("like" types.nearby_location_type)
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
					WHERE searcheable_response @@ to_tsquery(candidate) AND searcheable_response @@ to_tsquery(area_name);
                EXCEPTION
                    WHEN OTHERS THEN
                        GET STACKED DIAGNOSTICS error_msg = MESSAGE_TEXT;
                        RAISE WARNING 'Something went wrong with search_nearby_locations_with_area_name candidate= %: error_msg = %; area_name = % ', candidate, error_msg, area_name;
                END;
            END LOOP;

        RETURN QUERY SELECT * FROM temp_table;
        DROP TABLE temp_table;
    END;
$$;


DROP FUNCTION IF EXISTS location.search_nearby_locations_with_nearby_location_id;

DROP FUNCTION IF EXISTS location.search_nearby_locations;
