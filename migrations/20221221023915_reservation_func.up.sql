CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "btree_gist";

CREATE OR REPLACE FUNCTION rsvp.query(uid TEXT, rid TEXT, during TSTZRANGE) RETURNS TABLE (LIKE rsvp.reservations) AS $$
BEGIN
    -- if both are null, to find all reservations within during
    IF  uid IS NULL AND rid IS NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE timespan && during;
    -- if user_id is null, to find all reservations within during for the resource
    ELSIF uid IS NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND during @> timespan;
    -- if resource_id is null, to find all reservations within during for the user
    ELSIF rid is NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND during @> timespan;
    -- if both set, to find all reservations within during for the resource and user
    ELSE
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND user_id = uid AND during @> timespan;
    END IF;
END;
$$ LANGUAGE plpgsql;
