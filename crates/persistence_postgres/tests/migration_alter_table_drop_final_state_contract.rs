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
fn committed_drop_column_cannot_reuse_historical_tenant_boundary_evidence() {
    let catalog = table_catalog("ALTER TABLE tenant_record DROP COLUMN tenant_record_id CASCADE;");
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::UnsupportedTableFinalStateMutation),
        "a committed column drop must not leave the historical tenant_record_id declaration authoritative"
    );
}

#[test]
fn later_drop_action_in_one_alter_table_statement_cannot_hide_behind_additive_action() {
    let catalog = table_catalog(
        "ALTER TABLE tenant_record ADD COLUMN auxiliary_flag boolean, DROP COLUMN tenant_record_id CASCADE;",
    );
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::UnsupportedTableFinalStateMutation),
        "PostgreSQL action lists must be evaluated beyond the first additive action"
    );
}

#[test]
fn rolled_back_drop_column_does_not_remove_the_durable_tenant_boundary() {
    let catalog = table_catalog(
        "BEGIN; ALTER TABLE tenant_record DROP COLUMN tenant_record_id CASCADE; ROLLBACK;",
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn create_table_and_rls_only_catalog_remains_supported() {
    let catalog = table_catalog("");
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
