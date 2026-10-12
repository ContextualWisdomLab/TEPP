use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn tenant_identifier_in_policy_name_cannot_substitute_for_predicate_binding() {
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
        CREATE POLICY tenant_record_id ON document_record
            FOR ALL
            USING (
                document_record_id::text =
                    current_setting('tepp.current_tenant_record_id', true)
            );
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}
