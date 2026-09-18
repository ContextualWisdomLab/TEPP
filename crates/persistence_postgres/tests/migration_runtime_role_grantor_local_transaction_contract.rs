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

fn role_declarations() -> &'static str {
    r#"
        CREATE ROLE reporting_owner NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE grantor_a NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE grantor_b NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
    "#
}

#[test]
fn commit_restores_session_role_after_local_role_grantor() {
    let role_sql = format!(
        r#"
        {}
        SET ROLE grantor_a;
        BEGIN;
        SET LOCAL ROLE grantor_b;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY CURRENT_USER;
        COMMIT;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_USER;
        "#,
        role_declarations()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole),
        "post-COMMIT grantor_a revoke must not erase the unsafe row recorded under local grantor_b"
    );
}

#[test]
fn explicit_local_role_grantor_cleanup_restores_safety() {
    let role_sql = format!(
        r#"
        {}
        SET ROLE grantor_a;
        BEGIN;
        SET LOCAL ROLE grantor_b;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY CURRENT_USER;
        COMMIT;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_USER;
        SET ROLE grantor_b;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_USER;
        "#,
        role_declarations()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn commit_restores_session_authorization_after_local_override() {
    let role_sql = format!(
        r#"
        {}
        SET SESSION AUTHORIZATION grantor_a;
        BEGIN;
        SET LOCAL SESSION AUTHORIZATION grantor_b;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY SESSION_USER;
        COMMIT;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY SESSION_USER;
        "#,
        role_declarations()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole),
        "post-COMMIT grantor_a revoke must not erase the unsafe row recorded under local session grantor_b"
    );
}

#[test]
fn explicit_local_session_grantor_cleanup_restores_safety() {
    let role_sql = format!(
        r#"
        {}
        SET SESSION AUTHORIZATION grantor_a;
        BEGIN;
        SET LOCAL SESSION AUTHORIZATION grantor_b;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY SESSION_USER;
        COMMIT;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY SESSION_USER;
        SET SESSION AUTHORIZATION grantor_b;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY SESSION_USER;
        "#,
        role_declarations()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
