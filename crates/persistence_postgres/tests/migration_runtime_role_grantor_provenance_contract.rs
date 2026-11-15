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

fn grantor_roles() -> &'static str {
    r#"
        CREATE ROLE reporting_owner NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE grantor_a NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE grantor_b NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        GRANT reporting_owner TO grantor_a WITH INHERIT FALSE, SET FALSE, ADMIN TRUE;
        GRANT reporting_owner TO grantor_b WITH INHERIT FALSE, SET FALSE, ADMIN TRUE;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET TRUE, ADMIN FALSE
            GRANTED BY grantor_a;
        GRANT reporting_owner TO tepp_app_runtime
            WITH INHERIT FALSE, SET FALSE, ADMIN FALSE
            GRANTED BY grantor_b;
    "#
}

#[test]
fn revoking_one_grantor_does_not_erase_an_unsafe_alternative_membership_grant() {
    let role_sql = format!(
        "{}\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY grantor_b;",
        grantor_roles()
    );

    let catalog = rls_catalog(&role_sql);
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole),
        "a SET-capable grantor_a membership remains after only grantor_b is revoked"
    );
}

#[test]
fn explicitly_revoking_every_grantor_path_restores_runtime_membership_safety() {
    let role_sql = format!(
        "{}\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY grantor_b;\nREVOKE reporting_owner FROM tepp_app_runtime GRANTED BY grantor_a;",
        grantor_roles()
    );

    let catalog = rls_catalog(&role_sql);
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
