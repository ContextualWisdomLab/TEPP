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

fn distinct_quoted_grantor_paths() -> &'static str {
    r#"
        CREATE ROLE reporting_owner NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY "GrantorA";
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET FALSE, ADMIN FALSE
            GRANTED BY "GrantorB";
    "#
}

#[test]
fn revoking_one_case_distinct_quoted_grantor_does_not_erase_another_unsafe_path() {
    let role_sql = format!(
        "{}\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY \"GrantorB\";",
        distinct_quoted_grantor_paths()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole),
        "GrantorA and GrantorB are distinct PostgreSQL quoted role identities"
    );
}

#[test]
fn revoking_both_case_distinct_quoted_grantor_paths_restores_safety() {
    let role_sql = format!(
        "{}\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY \"GrantorB\";\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY \"GrantorA\";",
        distinct_quoted_grantor_paths()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
