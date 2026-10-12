use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn table_body_lookup_must_not_reuse_a_longer_table_name_prefix() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE tenant_record_archive (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL,
            valid_from timestamptz NOT NULL
        );
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY
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
