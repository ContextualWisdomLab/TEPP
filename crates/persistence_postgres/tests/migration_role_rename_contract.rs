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
fn renaming_the_runtime_role_away_invalidates_final_state_evidence() {
    for alter_alias in ["ALTER ROLE", "ALTER USER", "ALTER GROUP"] {
        let catalog = rls_catalog(&format!(
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\n{alter_alias} tepp_app_runtime RENAME TO archived_runtime_role;"
        ));

        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "{alter_alias}"
        );
    }
}

#[test]
fn role_rename_target_cannot_bypass_the_object_naming_contract() {
    let catalog = rls_catalog(
        "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;\nALTER ROLE tepp_app_runtime RENAME TO Bad;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::SingleWordObjectName)
    );
}
