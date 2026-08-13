-- TEPP model-run artifact chain (ADR 0013 / ERD MODEL_RUN + MODEL_ARTIFACT).
-- Append-only run and artifact identities bound to reproducibility and optional
-- corpus-split manifests. Tenant RLS policies are declared here for the new
-- tables so the embedded catalog remains FORCE-RLS complete.

CREATE TABLE corpus_split_manifest (
    corpus_split_manifest_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    split_manifest_digest text NOT NULL,
    knowledge_cutoff timestamptz NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT corpus_split_manifest_digest_unique UNIQUE (tenant_record_id, split_manifest_digest)
);

CREATE TABLE model_run (
    model_run_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    reproducibility_manifest_id uuid NOT NULL REFERENCES reproducibility_manifest (reproducibility_manifest_id),
    corpus_split_manifest_id uuid REFERENCES corpus_split_manifest (corpus_split_manifest_id),
    configuration_digest text NOT NULL,
    random_seed_manifest_digest text NOT NULL,
    engine_version_label text NOT NULL,
    compute_backend_code text NOT NULL,
    knowledge_cutoff timestamptz NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT model_run_identity_unique UNIQUE (
        tenant_record_id,
        reproducibility_manifest_id,
        configuration_digest,
        random_seed_manifest_digest
    )
);

CREATE TABLE model_artifact (
    model_artifact_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    model_run_id uuid NOT NULL REFERENCES model_run (model_run_id),
    artifact_type_code text NOT NULL,
    artifact_content_digest text NOT NULL,
    protected_object_ref text,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT model_artifact_content_unique UNIQUE (
        model_run_id,
        artifact_type_code,
        artifact_content_digest
    )
);

GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE corpus_split_manifest TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE model_run TO tepp_app_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE model_artifact TO tepp_app_runtime;

ALTER TABLE corpus_split_manifest ENABLE ROW LEVEL SECURITY;
ALTER TABLE corpus_split_manifest FORCE ROW LEVEL SECURITY;
CREATE POLICY corpus_split_manifest_tenant_isolation ON corpus_split_manifest
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE model_run ENABLE ROW LEVEL SECURITY;
ALTER TABLE model_run FORCE ROW LEVEL SECURITY;
CREATE POLICY model_run_tenant_isolation ON model_run
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE model_artifact ENABLE ROW LEVEL SECURITY;
ALTER TABLE model_artifact FORCE ROW LEVEL SECURITY;
CREATE POLICY model_artifact_tenant_isolation ON model_artifact
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );
