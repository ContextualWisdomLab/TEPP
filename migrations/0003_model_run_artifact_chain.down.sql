-- Rollback for 0003_model_run_artifact_chain.
-- Safe on empty databases: policy/table drops are existence-guarded.

DO $tepp$
DECLARE
    table_name text;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'model_artifact',
        'model_run',
        'corpus_split_manifest'
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
