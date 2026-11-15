use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn policy_catalog(transaction_outcome: &str) -> MigrationCatalog {
    let up_sql = format!(
        r#"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;

        SELECT current_setting('tepp.current_tenant_record_id', true);

        CREATE POLICY tenant_record_visibility_guard ON tenant_record
            AS RESTRICTIVE
            FOR SELECT
            USING (tenant_record_id IS NOT NULL);

        BEGIN;
        CREATE POLICY tenant_record_tenant_isolation ON tenant_record
            AS PERMISSIVE
            FOR SELECT
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            );
        {transaction_outcome};
        "#
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn rolled_back_permissive_policy_cannot_cover_surviving_restrictive_policy() {
    let catalog = policy_catalog("ROLLBACK");
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy),
        "rolled-back permissive policy must not satisfy final restrictive-policy composition"
    );
}

#[test]
fn committed_permissive_policy_can_cover_surviving_restrictive_policy() {
    let catalog = policy_catalog("COMMIT");
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
