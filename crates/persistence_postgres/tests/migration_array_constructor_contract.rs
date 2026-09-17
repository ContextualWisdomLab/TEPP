use persistence_postgres::{MigrationCatalog, validate_migration_catalog};

#[test]
fn array_constructor_commas_are_not_table_element_separators() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            retry_schedule integer[] DEFAULT ARRAY[1, 2],
            system_time timestamptz NOT NULL
        );
        ",
        "DROP TABLE tenant_record;",
    );

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
