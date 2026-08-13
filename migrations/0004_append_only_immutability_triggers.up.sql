-- Append-only immutability defense-in-depth (ADR 0013 / ERD).
-- Database roles lose UPDATE/DELETE on identity/manifest tables; triggers reject
-- mutations even for table owners who still hold privileges.

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
REVOKE UPDATE, DELETE ON TABLE audit_event FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE reproducibility_manifest FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE corpus_split_manifest FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE model_run FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE model_artifact FROM tepp_app_runtime;

CREATE TRIGGER source_artifact_reject_mutation
    BEFORE UPDATE OR DELETE ON source_artifact
    FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER audit_event_reject_mutation
    BEFORE UPDATE OR DELETE ON audit_event
    FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER reproducibility_manifest_reject_mutation
    BEFORE UPDATE OR DELETE ON reproducibility_manifest
    FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER corpus_split_manifest_reject_mutation
    BEFORE UPDATE OR DELETE ON corpus_split_manifest
    FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER model_run_reject_mutation
    BEFORE UPDATE OR DELETE ON model_run
    FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();

CREATE TRIGGER model_artifact_reject_mutation
    BEFORE UPDATE OR DELETE ON model_artifact
    FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();
