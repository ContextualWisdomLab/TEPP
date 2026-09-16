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
