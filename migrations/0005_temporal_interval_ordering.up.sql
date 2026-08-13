-- Physical temporal interval ordering (ADR 0013 / ERD).
-- Reject definitely-later starts than ends on valid/system windows while
-- preserving open-ended NULL upper bounds and equal (point) bounds.

ALTER TABLE document_record
    ADD CONSTRAINT document_record_valid_order
        CHECK (valid_to IS NULL OR valid_from <= valid_to),
    ADD CONSTRAINT document_record_system_order
        CHECK (system_to IS NULL OR system_from <= system_to),
    ADD CONSTRAINT document_record_revision_positive
        CHECK (revision_number > 0);

ALTER TABLE event_instance
    ADD CONSTRAINT event_instance_valid_order
        CHECK (valid_to IS NULL OR valid_from <= valid_to),
    ADD CONSTRAINT event_instance_system_order
        CHECK (system_to IS NULL OR system_from <= system_to);

ALTER TABLE membership_assignment
    ADD CONSTRAINT membership_assignment_valid_order
        CHECK (valid_to IS NULL OR valid_from <= valid_to);
