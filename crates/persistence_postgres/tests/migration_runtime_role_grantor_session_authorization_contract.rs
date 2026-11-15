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

fn session_switching_grants() -> &'static str {
    r#"
        CREATE ROLE reporting_owner NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE grantor_a NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE grantor_b NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;

        SET SESSION AUTHORIZATION grantor_a;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY SESSION_USER;

        SET SESSION AUTHORIZATION grantor_b;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET FALSE, ADMIN FALSE
            GRANTED BY SESSION_USER;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY SESSION_USER;
    "#
}

#[test]
fn session_user_grantor_tracks_set_session_authorization() {
    let catalog = rls_catalog(session_switching_grants());
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole),
        "revoking grantor_b must not erase the earlier SET-capable row recorded under grantor_a"
    );
}

#[test]
fn explicitly_revoking_each_session_user_grantor_restores_safety() {
    let role_sql = format!(
        "{}\nSET SESSION AUTHORIZATION grantor_a;\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY SESSION_USER;\nRESET SESSION AUTHORIZATION;",
        session_switching_grants()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
