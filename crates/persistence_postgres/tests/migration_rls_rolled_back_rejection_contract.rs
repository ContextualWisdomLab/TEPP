use persistence_postgres::{MigrationCatalog, validate_migration_catalog};

#[test]
fn rolled_back_invalid_policy_cannot_reject_valid_final_rls_state() {
    let up_sql = r#"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY tenant_record_tenant_isolation ON tenant_record
            AS PERMISSIVE
            FOR SELECT
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            );

        BEGIN;
        CREATE POLICY tenant_record_rolled_back_bad_policy ON tenant_record
            AS PERMISSIVE
            FOR SELECT
            USING (tenant_record_id IS NOT NULL);
        ROLLBACK;
    "#;
    let catalog = MigrationCatalog::from_sql(up_sql, "DROP TABLE tenant_record;");

    assert_eq!(
        validate_migration_catalog(&catalog),
        Ok(()),
        "policy evidence that PostgreSQL rolls back must not reject a valid final tenant-isolation policy set"
    );
}
