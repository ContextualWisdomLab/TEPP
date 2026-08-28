-- Durable analysis-run receipts and append-only lifecycle events (issue 166).

ALTER TABLE model_artifact
    ADD CONSTRAINT model_artifact_tenant_identity_unique
    UNIQUE (tenant_record_id, model_artifact_id);

CREATE TABLE analysis_run_request (
    analysis_run_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    tenant_workspace_id text NOT NULL CHECK (tenant_workspace_id <> ''),
    idempotency_key text NOT NULL CHECK (idempotency_key <> ''),
    request_contract_version smallint NOT NULL CHECK (request_contract_version > 0),
    snapshot_id text NOT NULL CHECK (snapshot_id <> ''),
    knowledge_cutoff timestamptz NOT NULL,
    model_contract_version text NOT NULL CHECK (model_contract_version <> ''),
    output_profile text NOT NULL CHECK (output_profile <> ''),
    request_payload_sha256 text NOT NULL CHECK (request_payload_sha256 ~ '^[0-9a-f]{64}$'),
    request_payload text NOT NULL CHECK (request_payload <> ''),
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT analysis_run_request_idempotency_unique UNIQUE (tenant_record_id, idempotency_key),
    CONSTRAINT analysis_run_request_tenant_identity_unique UNIQUE (tenant_record_id, analysis_run_id)
);

CREATE TABLE analysis_run_state_event (
    analysis_run_state_event_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL,
    analysis_run_id uuid NOT NULL,
    state_sequence bigint NOT NULL CHECK (state_sequence > 0),
    run_state_code text NOT NULL CHECK (run_state_code IN ('accepted', 'running', 'succeeded', 'failed')),
    model_artifact_id uuid,
    result_sha256 text CHECK (result_sha256 IS NULL OR result_sha256 ~ '^[0-9a-f]{64}$'),
    result_schema_version text,
    failure_code text,
    terminal_payload text,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT analysis_run_state_event_run_fk FOREIGN KEY (tenant_record_id, analysis_run_id)
        REFERENCES analysis_run_request (tenant_record_id, analysis_run_id),
    CONSTRAINT analysis_run_state_event_artifact_fk FOREIGN KEY (tenant_record_id, model_artifact_id)
        REFERENCES model_artifact (tenant_record_id, model_artifact_id),
    CONSTRAINT analysis_run_state_event_sequence_unique UNIQUE (analysis_run_id, state_sequence),
    CONSTRAINT analysis_run_state_event_terminal_shape CHECK (
        (run_state_code IN ('accepted', 'running') AND terminal_payload IS NULL
            AND model_artifact_id IS NULL AND result_sha256 IS NULL
            AND result_schema_version IS NULL AND failure_code IS NULL)
        OR (run_state_code = 'succeeded' AND terminal_payload IS NOT NULL
            AND model_artifact_id IS NOT NULL AND result_sha256 IS NOT NULL
            AND result_schema_version IS NOT NULL AND failure_code IS NULL)
        OR (run_state_code = 'failed' AND terminal_payload IS NOT NULL
            AND model_artifact_id IS NULL AND result_sha256 IS NULL
            AND result_schema_version IS NULL AND failure_code IS NOT NULL)
    )
);

CREATE UNIQUE INDEX analysis_run_state_event_terminal_unique
    ON analysis_run_state_event (analysis_run_id)
    WHERE run_state_code IN ('succeeded', 'failed');

CREATE FUNCTION validate_analysis_run_transition()
RETURNS trigger
LANGUAGE plpgsql
AS $tepp$
DECLARE
    previous_state text;
    previous_sequence bigint;
    stored_artifact_digest text;
BEGIN
    -- Serialize all state appends for one run, including the first event.
    PERFORM 1 FROM analysis_run_request
        WHERE tenant_record_id = NEW.tenant_record_id
          AND analysis_run_id = NEW.analysis_run_id
        FOR UPDATE;
    SELECT run_state_code, state_sequence
      INTO previous_state, previous_sequence
      FROM analysis_run_state_event
     WHERE tenant_record_id = NEW.tenant_record_id
       AND analysis_run_id = NEW.analysis_run_id
     ORDER BY state_sequence DESC
     LIMIT 1;
    IF (previous_state IS NULL AND (NEW.run_state_code <> 'accepted' OR NEW.state_sequence <> 1))
       OR (previous_state = 'accepted' AND (NEW.run_state_code NOT IN ('running', 'succeeded', 'failed') OR NEW.state_sequence <> previous_sequence + 1))
       OR (previous_state = 'running' AND (NEW.run_state_code NOT IN ('succeeded', 'failed') OR NEW.state_sequence <> previous_sequence + 1))
       OR previous_state IN ('succeeded', 'failed') THEN
        RAISE EXCEPTION 'invalid analysis-run state transition'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF NEW.run_state_code = 'succeeded' THEN
        SELECT artifact_content_digest INTO stored_artifact_digest
          FROM model_artifact
         WHERE tenant_record_id = NEW.tenant_record_id
           AND model_artifact_id = NEW.model_artifact_id;
        IF stored_artifact_digest IS DISTINCT FROM NEW.result_sha256 THEN
            RAISE EXCEPTION 'analysis-run artifact digest mismatch'
                USING ERRCODE = 'integrity_constraint_violation';
        END IF;
    END IF;
    RETURN NEW;
END
$tepp$;

CREATE TRIGGER analysis_run_state_event_validate_transition
    BEFORE INSERT ON analysis_run_state_event
    FOR EACH ROW EXECUTE FUNCTION validate_analysis_run_transition();

GRANT SELECT, INSERT ON TABLE analysis_run_request TO tepp_app_runtime;
GRANT SELECT, INSERT ON TABLE analysis_run_state_event TO tepp_app_runtime;

ALTER TABLE analysis_run_request ENABLE ROW LEVEL SECURITY;
ALTER TABLE analysis_run_request FORCE ROW LEVEL SECURITY;
CREATE POLICY analysis_run_request_tenant_isolation ON analysis_run_request
    FOR ALL USING (tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), ''))
    WITH CHECK (tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), ''));

ALTER TABLE analysis_run_state_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE analysis_run_state_event FORCE ROW LEVEL SECURITY;
CREATE POLICY analysis_run_state_event_tenant_isolation ON analysis_run_state_event
    FOR ALL USING (tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), ''))
    WITH CHECK (tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), ''));

REVOKE UPDATE, DELETE, TRUNCATE ON TABLE analysis_run_request FROM tepp_app_runtime;
REVOKE UPDATE, DELETE, TRUNCATE ON TABLE analysis_run_state_event FROM tepp_app_runtime;

CREATE TRIGGER analysis_run_request_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON analysis_run_request
    FOR EACH STATEMENT EXECUTE FUNCTION reject_append_only_mutation();
CREATE TRIGGER analysis_run_state_event_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON analysis_run_state_event
    FOR EACH STATEMENT EXECUTE FUNCTION reject_append_only_mutation();
