use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn table_body_lookup_cannot_cross_a_create_table_as_statement_boundary() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE tenant_record AS SELECT 1;
        CREATE TABLE tenant_record_archive (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL,
            valid_from timestamptz NOT NULL
        );
        ",
        r"
        DROP TABLE tenant_record;
        DROP TABLE tenant_record_archive;
        ",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingTemporalColumns)
    );
}
