use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn tenant_session_operand_cannot_fallback_to_the_row_tenant_identifier() {
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
                tenant_record_id::text = coalesce(
                    current_setting('tepp.current_tenant_record_id', true),
                    tenant_record_id::text
                )
            )
            WITH CHECK (
                tenant_record_id::text = coalesce(
                    current_setting('tepp.current_tenant_record_id', true),
                    tenant_record_id::text
                )
            );
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}
