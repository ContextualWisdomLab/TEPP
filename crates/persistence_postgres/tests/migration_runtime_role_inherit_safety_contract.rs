use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn rls_catalog(role_sql: &str) -> MigrationCatalog {
    let up_sql = format!(
        r#"
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
        "#
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn inherited_membership_without_owner_safety_evidence_fails_closed() {
    for role_sql in [
        "CREATE ROLE reporting_owner NOSUPERUSER NOBYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT reporting_owner TO tepp_app_runtime WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;",
        "CREATE ROLE reporting_owner NOSUPERUSER NOBYPASSRLS;\nCREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nGRANT reporting_owner TO tepp_app_runtime WITH SET FALSE, ADMIN FALSE;",
    ] {
        let catalog = rls_catalog(role_sql);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "{role_sql}"
        );
    }
}
