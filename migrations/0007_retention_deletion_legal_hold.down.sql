-- Rollback for 0007_retention_deletion_legal_hold.

DROP FUNCTION IF EXISTS release_legal_hold(uuid, timestamptz);
DROP FUNCTION IF EXISTS supersede_retention_policy(uuid, uuid, integer, text, timestamptz, timestamptz);
DROP FUNCTION IF EXISTS enforce_retention_policy_succession();
DROP FUNCTION IF EXISTS enforce_legal_hold_release();
DROP FUNCTION IF EXISTS reject_tombstoned_evidence_restore();
DROP FUNCTION IF EXISTS reject_held_evidence_deletion();
DROP FUNCTION IF EXISTS guard_legal_hold_insert();

DO $tepp$
DECLARE
    table_name text;
BEGIN
    IF to_regclass('public.document_record') IS NOT NULL THEN
        DROP TRIGGER IF EXISTS document_record_reject_tombstone_restore ON public.document_record;
    END IF;
    IF to_regclass('public.retention_policy') IS NOT NULL THEN
        DROP TRIGGER IF EXISTS retention_policy_enforce_succession ON public.retention_policy;
        DROP TRIGGER IF EXISTS retention_policy_reject_truncate ON public.retention_policy;
    END IF;
    IF to_regclass('public.legal_hold') IS NOT NULL THEN
        DROP TRIGGER IF EXISTS legal_hold_enforce_release ON public.legal_hold;
        DROP TRIGGER IF EXISTS legal_hold_guard_insert ON public.legal_hold;
        DROP TRIGGER IF EXISTS legal_hold_reject_mutation ON public.legal_hold;
    END IF;
    IF to_regclass('public.deletion_request') IS NOT NULL THEN
        DROP TRIGGER IF EXISTS deletion_request_reject_held_deletion ON public.deletion_request;
        DROP TRIGGER IF EXISTS deletion_request_reject_mutation ON public.deletion_request;
    END IF;
    IF to_regclass('public.evidence_tombstone') IS NOT NULL THEN
        DROP TRIGGER IF EXISTS evidence_tombstone_reject_mutation ON public.evidence_tombstone;
    END IF;

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
