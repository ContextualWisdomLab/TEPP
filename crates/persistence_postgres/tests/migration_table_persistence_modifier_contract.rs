use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn catalog_with(extra_sql: &str) -> MigrationCatalog {
    let up_sql = format!(
        "CREATE TABLE tenant_record (\n\
             tenant_record_id uuid PRIMARY KEY,\n\
             system_time timestamptz NOT NULL\n\
         );\n{extra_sql}"
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn table_persistence_modifiers_cannot_bypass_object_naming() {
    for statement in [
        "CREATE UNLOGGED TABLE Bad (bad_id uuid PRIMARY KEY, tenant_record_id uuid NOT NULL, system_time timestamptz NOT NULL, available_time timestamptz NOT NULL);",
        "CREATE TEMP TABLE Bad (bad_id uuid PRIMARY KEY, tenant_record_id uuid NOT NULL, system_time timestamptz NOT NULL, available_time timestamptz NOT NULL);",
        "CREATE TEMPORARY TABLE Bad (bad_id uuid PRIMARY KEY, tenant_record_id uuid NOT NULL, system_time timestamptz NOT NULL, available_time timestamptz NOT NULL);",
        "CREATE GLOBAL TEMP TABLE Bad (bad_id uuid PRIMARY KEY, tenant_record_id uuid NOT NULL, system_time timestamptz NOT NULL, available_time timestamptz NOT NULL);",
        "CREATE GLOBAL TEMPORARY TABLE Bad (bad_id uuid PRIMARY KEY, tenant_record_id uuid NOT NULL, system_time timestamptz NOT NULL, available_time timestamptz NOT NULL);",
        "CREATE LOCAL TEMP TABLE Bad (bad_id uuid PRIMARY KEY, tenant_record_id uuid NOT NULL, system_time timestamptz NOT NULL, available_time timestamptz NOT NULL);",
        "CREATE LOCAL TEMPORARY TABLE Bad (bad_id uuid PRIMARY KEY, tenant_record_id uuid NOT NULL, system_time timestamptz NOT NULL, available_time timestamptz NOT NULL);",
    ] {
        let catalog = catalog_with(statement);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "modifier-bearing table escaped the naming contract: {statement}"
        );
    }
}

#[test]
fn unlogged_table_still_traverses_table_local_tenant_contracts() {
    let catalog = catalog_with(
        "CREATE UNLOGGED TABLE derived_cache (\n\
             derived_cache_id uuid PRIMARY KEY,\n\
             system_time timestamptz NOT NULL,\n\
             available_time timestamptz NOT NULL\n\
         );",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingTenantBoundary)
    );
}
