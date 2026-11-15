use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn catalog(role_sql: &str) -> MigrationCatalog {
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
            USING (tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), ''))
            WITH CHECK (tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), ''));
        "#
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn quoted_grantor_identity_is_whitespace_insensitive_after_granted_keyword() {
    for separator in ["   ", "\n", "\t"] {
        let role_sql = format!(
            "GRANT reporting_owner TO tepp_app_runtime WITH INHERIT FALSE, SET TRUE, ADMIN FALSE GRANTED{separator}BY \"Grantor-A\";\n\
             GRANT reporting_owner TO tepp_app_runtime WITH INHERIT FALSE, SET FALSE, ADMIN FALSE GRANTED{separator}BY \"Grantor-B\";\n\
             REVOKE reporting_owner FROM tepp_app_runtime GRANTED{separator}BY \"Grantor-B\";"
        );
        assert_eq!(
            validate_migration_catalog(&catalog(&role_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "whitespace between GRANTED and BY must not collapse distinct grantor provenance"
        );
    }
}
