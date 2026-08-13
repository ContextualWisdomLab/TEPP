-- Append-only immutability defense-in-depth (ADR 0013 / ERD).
-- Database roles lose UPDATE/DELETE/TRUNCATE on identity/manifest tables;
-- statement-level triggers reject all destructive operations, including
-- zero-row UPDATE/DELETE statements and TRUNCATE by table owners.

CREATE OR REPLACE FUNCTION reject_append_only_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $tepp$
BEGIN
    RAISE EXCEPTION 'append-only table % rejects %', TG_TABLE_NAME, TG_OP
        USING ERRCODE = 'integrity_constraint_violation';
END
$tepp$;

-- Least-privilege application role: insert/select only on identity tables.
REVOKE UPDATE, DELETE ON TABLE source_artifact FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE source_artifact FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE audit_event FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE audit_event FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE reproducibility_manifest FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE reproducibility_manifest FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE corpus_split_manifest FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE corpus_split_manifest FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE model_run FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE model_run FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE model_artifact FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE model_artifact FROM tepp_app_runtime;

CREATE TRIGGER source_artifact_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON source_artifact
    FOR EACH STATEMENT EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER audit_event_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON audit_event
    FOR EACH STATEMENT EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER reproducibility_manifest_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON reproducibility_manifest
    FOR EACH STATEMENT EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER corpus_split_manifest_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON corpus_split_manifest
    FOR EACH STATEMENT EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER model_run_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON model_run
    FOR EACH STATEMENT EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER model_artifact_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON model_artifact
    FOR EACH STATEMENT EXECUTE FUNCTION reject_append_only_mutation();
