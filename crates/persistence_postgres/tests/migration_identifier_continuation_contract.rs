use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn postgresql_identifier_continuations_cannot_truncate_to_valid_table_prefixes() {
    for table_name in ["tenant_record$shadow", "tenant_record$1", "tenant_record측정"] {
        let up_sql = format!(
            "CREATE TABLE {table_name} (\n\
                 tenant_record_id uuid PRIMARY KEY,\n\
                 system_time timestamptz NOT NULL\n\
             );"
        );
        let down_sql = format!("DROP TABLE {table_name};");
        let catalog = MigrationCatalog::from_sql(&up_sql, &down_sql);

        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "PostgreSQL identifier continuation was truncated for {table_name}"
        );
    }
}

#[test]
fn postgresql_identifier_continuations_cannot_donate_rls_target_evidence() {
    for policy_target in ["document_record$shadow", "document_record측정"] {
        let up_sql = format!(
            r"
            CREATE TABLE document_record (
                document_record_id uuid PRIMARY KEY,
                tenant_record_id uuid NOT NULL,
                system_time timestamptz NOT NULL,
                available_time timestamptz NOT NULL
            );
            CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
            ALTER TABLE document_record ENABLE ROW LEVEL SECURITY;
            ALTER TABLE document_record FORCE ROW LEVEL SECURITY;
            CREATE POLICY document_record_tenant_isolation ON {policy_target}
                FOR ALL USING (
                    tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
                );
            "
        );
        let catalog = MigrationCatalog::from_sql(&up_sql, "DROP TABLE document_record;");

        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::MissingRlsPolicy),
            "RLS policy target prefix was accepted for {policy_target}"
        );
    }
}
