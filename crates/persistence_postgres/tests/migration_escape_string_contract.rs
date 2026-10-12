use persistence_postgres::{MigrationCatalog, validate_migration_catalog};

fn conforming_forward(extra_sql: &str) -> String {
    format!(
        "CREATE TABLE tenant_record (\n\
             tenant_record_id uuid PRIMARY KEY,\n\
             system_time timestamptz NOT NULL\n\
         );\n{extra_sql}"
    )
}

#[test]
fn postgres_escape_strings_do_not_break_forward_lexical_validation() {
    let up_sql = conforming_forward(
        r"SELECT E'it\'s forward metadata'; SELECT e'can\'t declare objects';",
    );
    let catalog = MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;");

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn postgres_escape_strings_do_not_break_rollback_lexical_validation() {
    let up_sql = conforming_forward("");
    let catalog = MigrationCatalog::from_sql(
        &up_sql,
        r"SELECT E'it\'s rollback metadata'; DROP TABLE tenant_record;",
    );

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
