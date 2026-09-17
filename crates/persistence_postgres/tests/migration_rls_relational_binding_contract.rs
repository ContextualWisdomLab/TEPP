use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn tenant_identifier_and_session_guc_must_form_the_same_equality_binding() {
    let catalog = MigrationCatalog::from_sql(
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
        CREATE POLICY document_record_tenant_isolation ON document_record
            FOR ALL
            USING (
                tenant_record_id IS NOT NULL
                AND current_setting('tepp.current_tenant_record_id', true) IS NOT NULL
            )
            WITH CHECK (
                tenant_record_id IS NOT NULL
                AND current_setting('tepp.current_tenant_record_id', true) IS NOT NULL
            );
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}

#[test]
fn explicit_with_check_cannot_weaken_a_tenant_bound_using_clause() {
    let catalog = MigrationCatalog::from_sql(
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
        CREATE POLICY document_record_tenant_isolation ON document_record
            FOR ALL
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            )
            WITH CHECK (
                tenant_record_id IS NOT NULL
            );
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}
