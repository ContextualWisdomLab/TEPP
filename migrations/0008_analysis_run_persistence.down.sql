DROP TABLE IF EXISTS analysis_run_state_event;
DROP TABLE IF EXISTS analysis_run_request;
DROP FUNCTION IF EXISTS validate_analysis_run_transition();
ALTER TABLE model_artifact
    DROP CONSTRAINT IF EXISTS model_artifact_tenant_identity_unique;
DROP INDEX IF EXISTS model_artifact_tenant_identity_unique_index;
