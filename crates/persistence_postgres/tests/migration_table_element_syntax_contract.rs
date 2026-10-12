use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn assert_empty_table_element_rejected(up_sql: &str) {
    let catalog = MigrationCatalog::from_sql(up_sql, "DROP TABLE tenant_record;");
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::EmptyMigrationSql),
        "invalid CREATE TABLE element list was accepted: {up_sql}"
    );
}

#[test]
fn create_table_element_lists_fail_closed_on_empty_elements() {
    for up_sql in [
        r"
        CREATE TABLE tenant_record (
            ,
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        ",
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            ,
            system_time timestamptz NOT NULL
        );
        ",
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL,
        );
        ",
    ] {
        assert_empty_table_element_rejected(up_sql);
    }
}
