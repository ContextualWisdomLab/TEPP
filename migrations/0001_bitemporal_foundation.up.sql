-- TEPP bitemporal foundation (Task 8 / ADR 0013).
-- Object names are multi-word snake_case. Temporal, tenant, and audit columns
-- are explicit. Live PostgreSQL execution is accepted-target; SQL is the
-- contract source validated by `persistence_postgres` migration checks.

CREATE TABLE tenant_record (
    tenant_record_id uuid PRIMARY KEY,
    tenant_status_code text NOT NULL,
    system_time timestamptz NOT NULL
);

CREATE TABLE source_artifact (
    source_artifact_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    content_sha256 text NOT NULL,
    source_size_bytes bigint NOT NULL,
    media_type_code text NOT NULL,
    protected_object_ref text,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT source_artifact_digest_unique UNIQUE (tenant_record_id, content_sha256)
);

CREATE TABLE document_record (
    document_record_id uuid NOT NULL,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    source_artifact_id uuid NOT NULL REFERENCES source_artifact (source_artifact_id),
    content_sha256 text NOT NULL,
    language_profile_code text NOT NULL,
    assertion_time timestamptz,
    document_time timestamptz,
    valid_from timestamptz NOT NULL,
    valid_to timestamptz,
    system_from timestamptz NOT NULL,
    system_to timestamptz,
    available_time timestamptz NOT NULL,
    revision_number bigint NOT NULL,
    PRIMARY KEY (document_record_id, system_from),
    CONSTRAINT document_record_revision_unique UNIQUE (document_record_id, revision_number)
);

CREATE TABLE event_instance (
    event_instance_id uuid NOT NULL,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    event_type_code text NOT NULL,
    valid_from timestamptz NOT NULL,
    valid_to timestamptz,
    system_from timestamptz NOT NULL,
    system_to timestamptz,
    available_time timestamptz NOT NULL,
    lifecycle_status_code text NOT NULL,
    PRIMARY KEY (event_instance_id, system_from)
);

CREATE TABLE event_mention (
    event_mention_id uuid PRIMARY KEY,
    event_instance_id uuid NOT NULL,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    confidence_score numeric NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL
);

CREATE TABLE event_relation (
    event_relation_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    source_event_id uuid NOT NULL,
    target_event_id uuid NOT NULL,
    relation_type_code text NOT NULL,
    transition_edge boolean NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL
);

CREATE TABLE membership_assignment (
    membership_assignment_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    observation_document_id uuid NOT NULL,
    membership_target_id uuid NOT NULL,
    role_code text NOT NULL,
    membership_weight numeric NOT NULL,
    valid_from timestamptz NOT NULL,
    valid_to timestamptz,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL
);

CREATE TABLE audit_event (
    audit_event_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    action_code text NOT NULL,
    subject_record_id uuid NOT NULL,
    recorded_system_time timestamptz NOT NULL,
    CONSTRAINT audit_event_immutable CHECK (true)
);

CREATE TABLE reproducibility_manifest (
    reproducibility_manifest_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    knowledge_cutoff timestamptz NOT NULL,
    evidence_digest text NOT NULL,
    code_commit_sha text NOT NULL,
    dependency_lock_digest text NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT reproducibility_manifest_digest_unique UNIQUE (evidence_digest, code_commit_sha, dependency_lock_digest)
);
