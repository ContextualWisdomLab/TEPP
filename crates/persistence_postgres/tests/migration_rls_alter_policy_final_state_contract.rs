use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn policy_catalog(final_mutation: &str) -> MigrationCatalog {
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
fn committed_alter_policy_cannot_reuse_historical_safe_create_policy_evidence() {
    let catalog = policy_catalog(
        "ALTER POLICY tenant_record_tenant_isolation ON tenant_record USING (true);",
    );
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy),
        "a committed policy mutation must not inherit stale safe CREATE POLICY evidence"
    );
}

#[test]
fn rolled_back_alter_policy_does_not_poison_the_durable_policy_definition() {
    let catalog = policy_catalog(
        "BEGIN; ALTER POLICY tenant_record_tenant_isolation ON tenant_record USING (true); ROLLBACK;",
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn create_policy_only_catalog_remains_supported() {
    let catalog = policy_catalog("");
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
