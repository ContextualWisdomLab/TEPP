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

fn quoted_current_user_and_pseudo_target_grants() -> &'static str {
    r#"
        CREATE ROLE reporting_owner NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE "current_user" NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        GRANT reporting_owner TO "current_user" WITH INHERIT FALSE, SET FALSE, ADMIN TRUE;
        GRANT reporting_owner TO CURRENT_USER WITH INHERIT FALSE, SET FALSE, ADMIN TRUE;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY "current_user";
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET FALSE, ADMIN FALSE
            GRANTED BY CURRENT_USER;
    "#
}

#[test]
fn quoted_named_grantor_does_not_alias_the_current_user_pseudo_target() {
    let role_sql = format!(
        "{}\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_USER;",
        quoted_current_user_and_pseudo_target_grants()
    );

    let catalog = rls_catalog(&role_sql);
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole),
        "the quoted named grantor remains SET-capable after only CURRENT_USER is revoked"
    );
}

#[test]
fn explicitly_revoking_the_quoted_and_pseudo_grantor_paths_restores_safety() {
    let role_sql = format!(
        "{}\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_USER;\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY \"current_user\";",
        quoted_current_user_and_pseudo_target_grants()
    );

    let catalog = rls_catalog(&role_sql);
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
