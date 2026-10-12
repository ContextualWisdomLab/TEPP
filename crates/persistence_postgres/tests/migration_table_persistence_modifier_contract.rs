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

#[test]
fn valid_modifier_bearing_tables_reuse_the_existing_table_contract() {
    for modifier in [
        "UNLOGGED",
        "TEMP",
        "TEMPORARY",
        "GLOBAL TEMP",
        "GLOBAL TEMPORARY",
        "LOCAL TEMP",
        "LOCAL TEMPORARY",
    ] {
        let statement = format!(
            "CREATE {modifier} TABLE derived_cache (\n\
                 derived_cache_id uuid PRIMARY KEY,\n\
                 tenant_record_id uuid NOT NULL,\n\
                 system_time timestamptz NOT NULL,\n\
                 available_time timestamptz NOT NULL\n\
             );"
        );
        let catalog = catalog_with(&statement);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Ok(()),
            "valid modifier-bearing table was not routed through the shared contract: {modifier}"
        );
    }
}

#[test]
fn lexical_spacing_before_a_modifier_is_preserved_as_structure() {
    let catalog = catalog_with(
        "CREATE /* persistence class */\nUNLOGGED\tTABLE derived_cache (\n\
             derived_cache_id uuid PRIMARY KEY,\n\
             tenant_record_id uuid NOT NULL,\n\
             system_time timestamptz NOT NULL,\n\
             available_time timestamptz NOT NULL\n\
         );",
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
