-- Rollback for 0006_typed_membership_assignment.
-- Restores the 0001/0002 membership stub so later foundation rollback stays valid.

DO $tepp$
DECLARE
    table_name text;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'membership_assignment',
        'text_segment',
        'project_record',
        'entity_record'
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

DO $tepp$
BEGIN
    IF to_regclass('public.tenant_record') IS NULL THEN
        RETURN;
    END IF;
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
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE membership_assignment TO tepp_app_runtime;
    ALTER TABLE membership_assignment ENABLE ROW LEVEL SECURITY;
    ALTER TABLE membership_assignment FORCE ROW LEVEL SECURITY;
    CREATE POLICY membership_assignment_tenant_isolation ON membership_assignment
        FOR ALL
        USING (
            tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
        )
        WITH CHECK (
            tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
        );
END
$tepp$;
