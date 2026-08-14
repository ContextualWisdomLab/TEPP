-- Rollback for 0007_retention_deletion_legal_hold.

DROP TRIGGER IF EXISTS document_record_reject_tombstone_restore ON document_record;
DROP TRIGGER IF EXISTS deletion_request_reject_held_deletion ON deletion_request;
DROP TRIGGER IF EXISTS retention_policy_enforce_succession ON retention_policy;
DROP TRIGGER IF EXISTS retention_policy_reject_truncate ON retention_policy;
DROP TRIGGER IF EXISTS legal_hold_reject_mutation ON legal_hold;
DROP TRIGGER IF EXISTS deletion_request_reject_mutation ON deletion_request;
DROP TRIGGER IF EXISTS evidence_tombstone_reject_mutation ON evidence_tombstone;
DROP FUNCTION IF EXISTS supersede_retention_policy(uuid, uuid, integer, text, timestamptz, timestamptz);
DROP FUNCTION IF EXISTS enforce_retention_policy_succession();
DROP FUNCTION IF EXISTS reject_tombstoned_evidence_restore();
DROP FUNCTION IF EXISTS reject_held_evidence_deletion();

DO $tepp$
DECLARE
    table_name text;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'evidence_tombstone',
        'deletion_request',
        'legal_hold',
        'retention_policy'
    ]
    LOOP
        IF to_regclass(format('public.%I', table_name)) IS NOT NULL THEN
            EXECUTE format(
                'DROP POLICY IF EXISTS %I ON public.%I',
                table_name || '_tenant_isolation',
                table_name
            );
            EXECUTE format(
                'ALTER TABLE public.%I NO FORCE ROW LEVEL SECURITY',
                table_name
            );
            EXECUTE format('ALTER TABLE public.%I DISABLE ROW LEVEL SECURITY', table_name);
            EXECUTE format(
                'REVOKE ALL ON TABLE public.%I FROM tepp_app_runtime',
                table_name
            );
            EXECUTE format('DROP TABLE IF EXISTS public.%I', table_name);
        END IF;
    END LOOP;
END
$tepp$;
