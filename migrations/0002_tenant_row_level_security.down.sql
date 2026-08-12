-- Rollback for 0002_tenant_row_level_security.
-- Safe on empty databases: each ALTER/DROP POLICY runs only when the table exists.

DO $tepp$
DECLARE
    table_name text;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'reproducibility_manifest',
        'audit_event',
        'membership_assignment',
        'event_relation',
        'event_mention',
        'event_instance',
        'document_record',
        'source_artifact',
        'tenant_record'
    ]
    LOOP
        IF to_regclass(format('public.%I', table_name)) IS NOT NULL THEN
            EXECUTE format(
                'DROP POLICY IF EXISTS %I ON %I',
                table_name || '_tenant_isolation',
                table_name
            );
            EXECUTE format('ALTER TABLE %I NO FORCE ROW LEVEL SECURITY', table_name);
            EXECUTE format('ALTER TABLE %I DISABLE ROW LEVEL SECURITY', table_name);
            EXECUTE format('REVOKE ALL ON TABLE %I FROM tepp_app_runtime', table_name);
        END IF;
    END LOOP;

    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'tepp_app_runtime') THEN
        EXECUTE 'REVOKE USAGE ON SCHEMA public FROM tepp_app_runtime';
        EXECUTE 'DROP ROLE tepp_app_runtime';
    END IF;
END
$tepp$;
