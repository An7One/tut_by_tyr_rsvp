DROP TRIGGER reservations_trigger ON rsvp.resservations;
DROP FUNCTION rsvp.reservations_trigger();
DROP TABLE rsvp.reservation_changes CASCADE;