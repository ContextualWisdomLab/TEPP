use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn reversed_pg_settings_name_equality_cannot_bypass_replica_guard() {
    for final_sql in [
        "UPDATE pg_settings SET setting = 'replica' WHERE 'session_replication_role' = name;",
        "UPDATE pg_settings AS p SET setting = 'replica' WHERE 'session_replication_role' = p.name;",
        "WITH marker AS (SELECT 1) UPDATE pg_settings AS p SET setting = 'replica' WHERE 'session_replication_role' = p.name;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "operand-reversed equality targets the same protected pg_settings row: {final_sql}",
        );
    }
}

#[test]
fn unproven_pg_settings_predicate_fails_closed_for_unsafe_value() {
    for final_sql in [
        "UPDATE pg_settings SET setting = 'replica' WHERE name IN ('session_replication_role');",
        "UPDATE pg_settings SET setting = lower('REPLICA') WHERE name LIKE 'session_replication_role';",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "bounded predicate parsing must not treat an unproven target as unrelated: {final_sql}",
        );
    }
}

#[test]
fn direct_unrelated_pg_settings_equality_remains_outside_the_guard() {
    for final_sql in [
        "UPDATE pg_settings SET setting = 'replica' WHERE name = 'application_name';",
        "UPDATE pg_settings AS p SET setting = lower('REPLICA') WHERE 'application_name' = p.name;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn safe_replication_modes_remain_accepted_for_broad_predicates() {
    for final_sql in [
        "UPDATE pg_settings SET setting = 'origin' WHERE name IN ('session_replication_role');",
        "UPDATE pg_settings SET setting = 'local' WHERE 'session_replication_role' = name;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn rolled_back_reversed_pg_settings_mutation_is_non_durable() {
    let final_sql = "BEGIN; UPDATE pg_settings SET setting = 'replica' WHERE 'session_replication_role' = name; ROLLBACK;";
    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}
