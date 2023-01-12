CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "btree_gist";

CREATE OR REPLACE FUNCTION rsvp.query(
    uid TEXT,
    rid TEXT,
    during TSTZRANGE,
    status rsvp.reservation_status,
    page integer DEFAULT 1,
    is_desc bool DEFAULT FALSE,
    page_size integer DEFAULT 10
) RETURNS TABLE (LIKE rsvp.reservations) AS $$
DECLARE
    _sql text;
BEGIN
    -- if the `page_size` is not between 10 and 100,
    -- to set it to 10,
    IF page_size < 10 OR page_size > 100 THEN
        page_size := 10;
    END IF;

    -- if the `page` is fewer than 1,
    -- to set it to 1
    IF page < 1 THEN
        page := 1;
    END IF;

    -- to format the query based parameters
    _sql := format(
        'SELECT * FROM rsvp.reservations WHERE %L @> timespan AND status = %L AND %s ORDER BY lower(timespan) %s LIMIT %L::integer OFFSET %L::integer',
        during,
        status,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
            WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
            ELSE 'user_id = ' || quote_literal(uid) || ' AND resource_id = ' || quote_literal(rid)
        END,
        CASE
            WHEN is_desc THEN 'DESC'
            ELSE 'ASC'
        END,
        page_size,
        (page - 1) * page_size
    );

    -- to log the query
    RAISE NOTICE '%', _sql;

    -- to execute the query
    RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;
