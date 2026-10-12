use persistence_postgres::{MigrationCatalog, validate_migration_catalog};

#[test]
fn non_ascii_dollar_quote_body_cannot_declare_migration_objects() {
    let catalog = MigrationCatalog::from_sql(
        "CREATE TABLE tenant_record (\n\
             tenant_record_id uuid PRIMARY KEY,\n\
             system_time timestamptz NOT NULL\n\
         );\n\
         SELECT $측정$ CREATE INDEX Bad ON tenant_record (tenant_record_id); $측정$;",
        "DROP TABLE tenant_record;",
    );

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
