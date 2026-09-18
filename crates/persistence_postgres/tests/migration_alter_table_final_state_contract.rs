use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn table_catalog(final_mutation: &str) -> MigrationCatalog {
    let up_sql = format!(
        r#"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY tenant_record_tenant_isolation ON tenant_record
            FOR ALL
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            );
        {final_mutation}
        "#
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn committed_table_rename_cannot_reuse_historical_table_evidence() {
    let catalog = table_catalog("ALTER TABLE tenant_record RENAME TO tenant_record_archive;");
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::UnsupportedTableFinalStateMutation),
        "a committed table rename must not leave the historical CREATE TABLE identity authoritative"
    );
}

#[test]
fn committed_tenant_column_rename_cannot_reuse_historical_column_evidence() {
    let catalog = table_catalog(
        "ALTER TABLE tenant_record RENAME COLUMN tenant_record_id TO tenant_key;",
    );
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::UnsupportedTableFinalStateMutation),
        "a committed tenant-column rename must not reuse the historical tenant_record_id declaration"
    );
}

#[test]
fn committed_table_schema_move_cannot_reuse_historical_relation_identity() {
    for mutation in [
        "ALTER TABLE tenant_record SET SCHEMA archive;",
        "ALTER TABLE IF EXISTS tenant_record SET SCHEMA archive;",
    ] {
        let catalog = table_catalog(mutation);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::UnsupportedTableFinalStateMutation),
            "moving the relation to another schema must invalidate historical unqualified table evidence"
        );
    }
}

#[test]
fn rolled_back_table_identity_mutations_do_not_change_the_durable_contract() {
    for mutation in [
        "BEGIN; ALTER TABLE tenant_record RENAME TO tenant_record_archive; ROLLBACK;",
        "BEGIN; ALTER TABLE tenant_record RENAME COLUMN tenant_record_id TO tenant_key; ROLLBACK;",
        "BEGIN; ALTER TABLE tenant_record SET SCHEMA archive; ROLLBACK;",
    ] {
        let catalog = table_catalog(mutation);
        assert_eq!(validate_migration_catalog(&catalog), Ok(()));
    }
}

#[test]
fn set_schema_marker_text_outside_structure_remains_inert() {
    let catalog = table_catalog(
        "SELECT 'ALTER TABLE tenant_record SET SCHEMA archive'; -- ALTER TABLE tenant_record SET SCHEMA archive",
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn create_table_and_rls_only_catalog_remains_supported() {
    let catalog = table_catalog("");
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
