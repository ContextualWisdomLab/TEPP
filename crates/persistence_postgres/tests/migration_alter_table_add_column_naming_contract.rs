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
fn committed_add_column_must_apply_the_durable_naming_contract() {
    for mutation in [
        "ALTER TABLE tenant_record ADD COLUMN flag boolean;",
        "ALTER TABLE tenant_record ADD COLUMN IF NOT EXISTS flag boolean;",
        "ALTER TABLE tenant_record ADD flag boolean;",
    ] {
        let catalog = table_catalog(mutation);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "ALTER TABLE ADD COLUMN must not bypass the CREATE TABLE column naming contract: {mutation}"
        );
    }
}

#[test]
fn later_add_column_action_cannot_hide_behind_a_safe_first_action() {
    let catalog = table_catalog(
        "ALTER TABLE tenant_record ADD COLUMN auxiliary_flag boolean, ADD COLUMN flag boolean;",
    );
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::SingleWordObjectName),
    );
}

#[test]
fn rolled_back_add_column_does_not_change_durable_naming_evidence() {
    let catalog = table_catalog("BEGIN; ALTER TABLE tenant_record ADD COLUMN flag boolean; ROLLBACK;");
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn safe_add_column_and_table_constraint_forms_remain_supported() {
    for mutation in [
        "ALTER TABLE tenant_record ADD COLUMN auxiliary_flag boolean;",
        "ALTER TABLE tenant_record ADD CONSTRAINT tenant_record_shape CHECK (tenant_record_id IS NOT NULL);",
        "ALTER TABLE tenant_record ADD CHECK (tenant_record_id IS NOT NULL);",
        "ALTER TABLE tenant_record ADD FOREIGN KEY (tenant_record_id) REFERENCES tenant_record (tenant_record_id);",
    ] {
        let catalog = table_catalog(mutation);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Ok(()),
            "table-constraint forms must not be misclassified as added columns: {mutation}"
        );
    }
}
