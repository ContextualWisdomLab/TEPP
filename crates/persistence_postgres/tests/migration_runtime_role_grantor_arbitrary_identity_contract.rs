use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn rls_catalog(role_sql: &str) -> MigrationCatalog {
    let up_sql = format!(
        r#"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        CREATE ROLE reporting_owner NOSUPERUSER NOBYPASSRLS;
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
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

fn grant(grantor: &str, set_enabled: bool) -> String {
    format!(
        "GRANT reporting_owner TO tepp_app_runtime WITH INHERIT FALSE, SET {set_enabled}, ADMIN FALSE GRANTED BY {grantor};"
    )
}

fn revoke(grantor: &str) -> String {
    format!("REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY {grantor};")
}

#[test]
fn arbitrary_distinct_quoted_grantors_do_not_collapse_to_one_provenance_key() {
    for (unsafe_grantor, safe_grantor) in [
        (r#""Grantor-A""#, r#""Grantor-B""#),
        (r#""권한A""#, r#""권한B""#),
        (r#""Grantor""A""#, r#""Grantor""B""#),
    ] {
        let role_sql = format!(
            "{}\n{}\n{}",
            grant(unsafe_grantor, true),
            grant(safe_grantor, false),
            revoke(safe_grantor),
        );
        let catalog = rls_catalog(&role_sql);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "revoking {safe_grantor} must not erase distinct unsafe grantor {unsafe_grantor}"
        );
    }
}

#[test]
fn revoking_every_arbitrary_quoted_grantor_path_restores_safety() {
    for (unsafe_grantor, safe_grantor) in [
        (r#""Grantor-A""#, r#""Grantor-B""#),
        (r#""권한A""#, r#""권한B""#),
        (r#""Grantor""A""#, r#""Grantor""B""#),
    ] {
        let role_sql = format!(
            "{}\n{}\n{}\n{}",
            grant(unsafe_grantor, true),
            grant(safe_grantor, false),
            revoke(safe_grantor),
            revoke(unsafe_grantor),
        );
        let catalog = rls_catalog(&role_sql);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Ok(()),
            "every distinct grantor row has been revoked for {unsafe_grantor} / {safe_grantor}"
        );
    }
}
