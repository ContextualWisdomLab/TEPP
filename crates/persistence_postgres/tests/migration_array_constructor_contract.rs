use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn array_constructor_commas_are_not_table_element_separators() {
    for up_sql in [
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            retry_schedule integer[] DEFAULT ARRAY[1, 2],
            system_time timestamptz NOT NULL
        );
        ",
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            retry_matrix integer[][] DEFAULT ARRAY[[1, 2], [3, 4]],
            system_time timestamptz NOT NULL
        );
        ",
    ] {
        let catalog = MigrationCatalog::from_sql(up_sql, "DROP TABLE tenant_record;");
        assert_eq!(validate_migration_catalog(&catalog), Ok(()), "{up_sql}");
    }
}

#[test]
fn malformed_square_bracket_nesting_fails_closed() {
    for up_sql in [
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            retry_schedule integer[] DEFAULT ARRAY[1, 2,
            system_time timestamptz NOT NULL
        );
        ",
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            retry_schedule integer[] DEFAULT 1],
            system_time timestamptz NOT NULL
        );
        ",
    ] {
        let catalog = MigrationCatalog::from_sql(up_sql, "DROP TABLE tenant_record;");
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::EmptyMigrationSql),
            "{up_sql}"
        );
    }
}
