DROP TRIGGER IF EXISTS analysis_run_state_event_reject_mutation ON analysis_run_state_event;
DROP TRIGGER IF EXISTS analysis_run_request_reject_mutation ON analysis_run_request;
DROP TRIGGER IF EXISTS analysis_run_state_event_validate_transition ON analysis_run_state_event;
DROP FUNCTION IF EXISTS validate_analysis_run_transition();
DROP TABLE IF EXISTS analysis_run_state_event;
DROP TABLE IF EXISTS analysis_run_request;
ALTER TABLE model_artifact
    DROP CONSTRAINT IF EXISTS model_artifact_tenant_identity_unique;
