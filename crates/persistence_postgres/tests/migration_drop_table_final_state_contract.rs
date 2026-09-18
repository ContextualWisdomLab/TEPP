use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn table_catalog(final_mutation: &str) -> MigrationCatalog {
    let up_sql = format!(
        r#"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY tenant_record_tenant_isolation ON tenant_record
            FOR ALL
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            );
        {final_mutation}
        "#
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn committed_drop_table_cannot_reuse_historical_create_and_rls_evidence() {
    let catalog = table_catalog("DROP TABLE tenant_record;");
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::EmptyMigrationSql),
        "a committed table removal must invalidate historical CREATE TABLE and dependent RLS evidence"
    );
}

#[test]
fn rolled_back_drop_table_does_not_remove_the_durable_table() {
    let catalog = table_catalog("BEGIN; DROP TABLE tenant_record; ROLLBACK;");
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn create_table_only_catalog_remains_supported() {
    let catalog = table_catalog("");
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
