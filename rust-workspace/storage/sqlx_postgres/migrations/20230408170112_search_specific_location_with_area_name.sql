-- Add migration script here
CREATE OR REPLACE FUNCTION location.search_specific_location_primary_text(candidates text[], location_id uuid, area text) RETURNS TABLE("like" types.location_name_and_search_query_with_id)
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
					WHERE main_text_searcheable_index_col @@ to_tsquery(candidate) AND id = location_id AND secondary_text_searcheable_index_col @@ to_tsquery(area);
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


CREATE OR REPLACE FUNCTION location.search_specific_location_secondary_text(candidates text[], location_id uuid, area text) RETURNS TABLE("like" types.location_name_and_search_query_with_id)
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
					WHERE secondary_text_searcheable_index_col @@ to_tsquery(candidate) AND id = location_id AND secondary_text_searcheable_index_col @@ to_tsquery(area);
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