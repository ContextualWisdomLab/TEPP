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

fn role_switching_current_user_grants() -> &'static str {
    r#"
        CREATE ROLE reporting_owner NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE grantor_a NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE grantor_b NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;

        GRANT reporting_owner TO grantor_a WITH INHERIT FALSE, SET FALSE, ADMIN TRUE;
        GRANT reporting_owner TO grantor_b WITH INHERIT FALSE, SET FALSE, ADMIN TRUE;

        SET ROLE grantor_a;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY CURRENT_USER;
        RESET ROLE;

        SET ROLE grantor_b;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET FALSE, ADMIN FALSE
            GRANTED BY CURRENT_USER;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_USER;
        RESET ROLE;
    "#
}

#[test]
fn current_user_grantor_tracks_the_effective_role_after_set_role() {
    let catalog = rls_catalog(role_switching_current_user_grants());
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole),
        "revoking grantor_b must not erase the earlier SET-capable grant recorded under grantor_a"
    );
}

#[test]
fn explicitly_revoking_each_effective_current_user_grantor_restores_safety() {
    let role_sql = format!(
        "{}\nSET ROLE grantor_a;\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_USER;\nRESET ROLE;",
        role_switching_current_user_grants()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
