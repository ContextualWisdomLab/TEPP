use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn tenant_policy_bundle(role_sql: &str, transaction_prefix: &str, transaction_suffix: &str) -> MigrationCatalog {
    let up_sql = format!(
        r#"
        {transaction_prefix}
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        {role_sql}
        ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY tenant_record_tenant_isolation ON tenant_record
            FOR ALL
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            )
            WITH CHECK (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            );
        {transaction_suffix}
        "#
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn rolled_back_runtime_role_hardening_cannot_certify_bypassrls_role() {
    let catalog = tenant_policy_bundle(
        r#"
        CREATE ROLE tepp_app_runtime BYPASSRLS;
        BEGIN;
        ALTER ROLE tepp_app_runtime NOBYPASSRLS;
        ROLLBACK;
        "#,
        "",
        "",
    );
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole),
        "rolled-back NOBYPASSRLS evidence must not change final runtime-role security state"
    );
}

#[test]
fn rolled_back_structural_bundle_cannot_satisfy_final_migration_state() {
    let catalog = tenant_policy_bundle(
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;",
        "BEGIN;",
        "ROLLBACK;",
    );
    assert!(
        validate_migration_catalog(&catalog).is_err(),
        "DDL and RLS evidence that PostgreSQL rolls back must not satisfy the durable migration contract"
    );
}
