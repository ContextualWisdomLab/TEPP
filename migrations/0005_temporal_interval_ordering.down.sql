-- Rollback temporal interval ordering checks.

ALTER TABLE membership_assignment
    DROP CONSTRAINT IF EXISTS membership_assignment_valid_order;

ALTER TABLE event_instance
    DROP CONSTRAINT IF EXISTS event_instance_system_order,
    DROP CONSTRAINT IF EXISTS event_instance_valid_order;

ALTER TABLE document_record
    DROP CONSTRAINT IF EXISTS document_record_revision_positive,
    DROP CONSTRAINT IF EXISTS document_record_system_order,
    DROP CONSTRAINT IF EXISTS document_record_valid_order;
