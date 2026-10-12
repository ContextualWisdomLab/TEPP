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
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
    "#
}

#[test]
fn rolled_back_membership_revoke_cannot_donate_safety() {
    let role_sql = format!(
        r#"
        {}
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE;
        BEGIN;
        REVOKE reporting_owner FROM tepp_app_runtime;
        ROLLBACK;
        "#,
        role_declarations()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole)
    );
}

#[test]
fn committed_membership_revoke_restores_safety() {
    let role_sql = format!(
        r#"
        {}
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE;
        BEGIN;
        REVOKE reporting_owner FROM tepp_app_runtime;
        COMMIT;
        "#,
        role_declarations()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn rolled_back_explicit_grantor_revoke_cannot_donate_safety() {
    let role_sql = format!(
        r#"
        {}
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY grantor_a;
        BEGIN;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY grantor_a;
        ROLLBACK;
        "#,
        role_declarations()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole)
    );
}

#[test]
fn committed_explicit_grantor_revoke_restores_safety() {
    let role_sql = format!(
        r#"
        {}
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY grantor_a;
        BEGIN;
        REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY grantor_a;
        COMMIT;
        "#,
        role_declarations()
    );
    let catalog = rls_catalog(&role_sql);
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
