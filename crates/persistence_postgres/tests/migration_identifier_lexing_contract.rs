use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn conforming_catalog(extra_sql: &str) -> MigrationCatalog {
    let up_sql = format!(
        "CREATE TABLE tenant_record (\n\
             tenant_record_id uuid PRIMARY KEY,\n\
             system_time timestamptz NOT NULL\n\
         );\n{extra_sql}"
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn quoted_created_object_cannot_bypass_the_naming_contract() {
    let catalog = conforming_catalog(
        "CREATE INDEX \"Bad\" ON tenant_record (tenant_record_id);",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::SingleWordObjectName)
    );
}

#[test]
fn quoted_identifier_with_escaped_quote_cannot_truncate_to_a_valid_prefix() {
    let catalog = conforming_catalog(
        "CREATE INDEX \"good_index\"\"suffix\" ON tenant_record (tenant_record_id);",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::SingleWordObjectName)
    );
}

#[test]
fn quoted_column_cannot_bypass_the_naming_contract() {
    let catalog = MigrationCatalog::from_sql(
        "CREATE TABLE tenant_record (\n\
             tenant_record_id uuid PRIMARY KEY,\n\
             system_time timestamptz NOT NULL,\n\
             \"Bad\" text\n\
         );",
        "DROP TABLE tenant_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::SingleWordObjectName)
    );
}

#[test]
fn quoted_column_named_like_a_table_constraint_keyword_is_still_an_identifier() {
    for keyword in [
        "constraint",
        "primary",
        "foreign",
        "unique",
        "check",
        "exclude",
        "like",
    ] {
        let up_sql = format!(
            "CREATE TABLE tenant_record (\n\
                 tenant_record_id uuid PRIMARY KEY,\n\
                 system_time timestamptz NOT NULL,\n\
                 \"{keyword}\" uuid\n\
             );"
        );
        let catalog = MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;");

        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "quoted identifier {keyword} was reinterpreted as table syntax"
        );
    }
}

#[test]
fn dollar_quote_like_bytes_inside_identifiers_do_not_bypass_the_naming_contract() {
    for statement in [
        "CREATE INDEX good_index$tag$bad$tag$ ON tenant_record (tenant_record_id);",
        "CREATE INDEX bad_index$tag$ ON tenant_record (tenant_record_id);",
        "CREATE INDEX bad_index$$tag$ ON tenant_record (tenant_record_id);",
        "CREATE INDEX bad_측정$tag$ ON tenant_record (tenant_record_id);",
    ] {
        let catalog = conforming_catalog(statement);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "{statement}"
        );
    }
}

#[test]
fn declaration_shaped_text_inside_sql_trivia_is_not_an_object() {
    let catalog = conforming_catalog(
        "-- CREATE INDEX Bad ON tenant_record (tenant_record_id);\n\
         SELECT 'CREATE INDEX Bad ON tenant_record (tenant_record_id);';\n\
         /* CREATE INDEX Bad ON tenant_record (tenant_record_id); */",
    );

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn adjacent_atomic_literals_cannot_splice_a_create_keyword() {
    let catalog = conforming_catalog(
        "SELECT 'CREATE'\n\
                'INDEX' AS literal_text;",
    );

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn materialized_view_names_are_covered_by_the_object_naming_contract() {
    let catalog = conforming_catalog(
        "CREATE MATERIALIZED VIEW Bad AS SELECT tenant_record_id FROM tenant_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::SingleWordObjectName)
    );
}

#[test]
fn replaceable_view_names_are_covered_by_the_object_naming_contract() {
    let catalog = conforming_catalog(
        "CREATE OR REPLACE VIEW Bad AS SELECT tenant_record_id FROM tenant_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::SingleWordObjectName)
    );
}

#[test]
fn qualified_created_object_cannot_hide_an_invalid_object_segment() {
    for statement in [
        "CREATE VIEW audit_schema.Bad AS SELECT tenant_record_id FROM tenant_record;",
        "CREATE VIEW \"audit_schema\".\"Bad\" AS SELECT tenant_record_id FROM tenant_record;",
    ] {
        let catalog = conforming_catalog(statement);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "{statement}"
        );
    }
}

#[test]
fn created_role_aliases_are_covered_by_the_object_naming_contract() {
    for statement in [
        "CREATE ROLE Bad NOSUPERUSER NOBYPASSRLS;",
        "CREATE USER Bad NOSUPERUSER NOBYPASSRLS;",
        "CREATE GROUP Bad NOSUPERUSER NOBYPASSRLS;",
    ] {
        let catalog = conforming_catalog(statement);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "{statement}"
        );
    }
}

#[test]
fn create_user_mapping_is_not_a_role_alias() {
    let catalog = conforming_catalog(
        "CREATE USER MAPPING FOR CURRENT_USER SERVER foreign_server;",
    );

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn runtime_role_reference_does_not_substitute_for_role_declaration() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        GRANT SELECT ON TABLE tenant_record TO tepp_app_runtime;
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
        ",
        "DROP TABLE tenant_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole)
    );
}

#[test]
fn runtime_role_must_still_exist_after_the_forward_migration() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        DROP ROLE tepp_app_runtime;
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
        ",
        "DROP TABLE tenant_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingAppRuntimeRole)
    );
}

#[test]
fn adjacent_statement_delimiters_cannot_hide_runtime_role_drops() {
    for drop_alias in ["DROP ROLE", "DROP USER", "DROP GROUP"] {
        let up_sql = format!(
            "CREATE TABLE tenant_record (\n\
                 tenant_record_id uuid PRIMARY KEY,\n\
                 system_time timestamptz NOT NULL\n\
             );\n\
             CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;{drop_alias} tepp_app_runtime;\n\
             ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;\n\
             ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;\n\
             CREATE POLICY tenant_record_tenant_isolation ON tenant_record\n\
                 FOR ALL\n\
                 USING (tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), ''))\n\
                 WITH CHECK (tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), ''));"
        );
        let catalog = MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;");

        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "{drop_alias}"
        );
    }
}
