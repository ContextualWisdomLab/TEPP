-- Rollback for 0004_append_only_immutability_triggers.
-- Safe on empty databases via existence guards.

DO $tepp$
DECLARE
    trigger_table text;
BEGIN
    FOREACH trigger_table IN ARRAY ARRAY[
        'source_artifact',
        'audit_event',
        'reproducibility_manifest',
        'corpus_split_manifest',
        'model_run',
        'model_artifact'
    ]
    LOOP
        IF to_regclass(format('public.%I', trigger_table)) IS NOT NULL THEN
            EXECUTE format(
                'DROP TRIGGER IF EXISTS %I ON %I',
                trigger_table || '_reject_mutation',
                trigger_table
            );
        END IF;
    END LOOP;

    DROP FUNCTION IF EXISTS reject_append_only_mutation();

    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'tepp_app_runtime') THEN
        IF to_regclass('public.source_artifact') IS NOT NULL THEN
            EXECUTE 'GRANT UPDATE, DELETE ON TABLE source_artifact TO tepp_app_runtime';
        END IF;
        IF to_regclass('public.audit_event') IS NOT NULL THEN
            EXECUTE 'GRANT UPDATE, DELETE ON TABLE audit_event TO tepp_app_runtime';
        END IF;
        IF to_regclass('public.reproducibility_manifest') IS NOT NULL THEN
            EXECUTE 'GRANT UPDATE, DELETE ON TABLE reproducibility_manifest TO tepp_app_runtime';
        END IF;
        IF to_regclass('public.corpus_split_manifest') IS NOT NULL THEN
            EXECUTE 'GRANT UPDATE, DELETE ON TABLE corpus_split_manifest TO tepp_app_runtime';
        END IF;
        IF to_regclass('public.model_run') IS NOT NULL THEN
            EXECUTE 'GRANT UPDATE, DELETE ON TABLE model_run TO tepp_app_runtime';
        END IF;
        IF to_regclass('public.model_artifact') IS NOT NULL THEN
            EXECUTE 'GRANT UPDATE, DELETE ON TABLE model_artifact TO tepp_app_runtime';
        END IF;
    END IF;
END
$tepp$;
