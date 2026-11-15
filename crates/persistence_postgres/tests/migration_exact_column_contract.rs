use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn tenant_boundary_requires_the_exact_tenant_record_id_column() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE document_record (
            document_record_id uuid PRIMARY KEY,
            tenant_record_id_shadow uuid NOT NULL,
            system_time timestamptz NOT NULL,
            available_time timestamptz NOT NULL
        );
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingTenantBoundary)
    );
}

#[test]
fn system_time_requires_an_exact_supported_column_name() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time_shadow timestamptz NOT NULL
        );
        ",
        "DROP TABLE tenant_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingTemporalColumns)
    );
}

#[test]
fn domain_time_requires_an_exact_supported_column_name() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE document_record (
            document_record_id uuid PRIMARY KEY,
            tenant_record_id uuid NOT NULL,
            system_time timestamptz NOT NULL,
            available_time_shadow timestamptz NOT NULL
        );
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingTemporalColumns)
    );
}
