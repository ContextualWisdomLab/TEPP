-- TEPP typed membership assignment (ADR 0013 / ERD MEMBERSHIP_ASSIGNMENT).
-- Replaces the 0001 polymorphic membership_target_id stub with exactly-one
-- typed foreign keys for the observed unit and the membership target.

CREATE TABLE entity_record (
    entity_record_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    entity_type_code text NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL
);

CREATE TABLE project_record (
    project_record_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    project_status_code text NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL
);

CREATE TABLE text_segment (
    text_segment_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    document_record_id uuid NOT NULL,
    start_byte bigint NOT NULL,
    end_byte bigint NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT text_segment_byte_span CHECK (start_byte >= 0 AND end_byte > start_byte)
);

DROP POLICY IF EXISTS membership_assignment_tenant_isolation ON membership_assignment;
DROP TABLE membership_assignment;

CREATE TABLE membership_assignment (
    membership_assignment_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    document_record_id uuid,
    text_segment_id uuid REFERENCES text_segment (text_segment_id),
    target_entity_id uuid REFERENCES entity_record (entity_record_id),
    target_project_id uuid REFERENCES project_record (project_record_id),
    membership_type_code text NOT NULL,
    membership_weight numeric NOT NULL,
    valid_from_window tstzrange NOT NULL,
    valid_to_window tstzrange,
    valid_time_precision_code text NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT membership_assignment_observed_unit_exactly_one CHECK (
        (document_record_id IS NOT NULL)::integer
        + (text_segment_id IS NOT NULL)::integer
        = 1
    ),
    CONSTRAINT membership_assignment_target_exactly_one CHECK (
        (target_entity_id IS NOT NULL)::integer
        + (target_project_id IS NOT NULL)::integer
        = 1
    ),
    CONSTRAINT membership_assignment_weight_positive CHECK (membership_weight > 0),
    CONSTRAINT membership_assignment_from_window_nonempty CHECK (
        NOT isempty(valid_from_window)
    ),
    CONSTRAINT membership_assignment_to_window_open_or_nonempty CHECK (
        valid_to_window IS NULL OR NOT isempty(valid_to_window)
    ),
    CONSTRAINT membership_assignment_windows_ordered CHECK (
        valid_to_window IS NULL OR NOT (valid_from_window >> valid_to_window)
    )
);

GRANT SELECT, INSERT ON TABLE entity_record TO tepp_app_runtime;
GRANT SELECT, INSERT ON TABLE project_record TO tepp_app_runtime;
GRANT SELECT, INSERT ON TABLE text_segment TO tepp_app_runtime;
GRANT SELECT, INSERT ON TABLE membership_assignment TO tepp_app_runtime;

ALTER TABLE entity_record ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_record FORCE ROW LEVEL SECURITY;
CREATE POLICY entity_record_tenant_isolation ON entity_record
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE project_record ENABLE ROW LEVEL SECURITY;
ALTER TABLE project_record FORCE ROW LEVEL SECURITY;
CREATE POLICY project_record_tenant_isolation ON project_record
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE text_segment ENABLE ROW LEVEL SECURITY;
ALTER TABLE text_segment FORCE ROW LEVEL SECURITY;
CREATE POLICY text_segment_tenant_isolation ON text_segment
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE membership_assignment ENABLE ROW LEVEL SECURITY;
ALTER TABLE membership_assignment FORCE ROW LEVEL SECURITY;
CREATE POLICY membership_assignment_tenant_isolation ON membership_assignment
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );
