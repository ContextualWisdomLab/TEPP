use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn committed_replica_execution_mode_cannot_bypass_runtime_trigger_enforcement() {
    for final_sql in [
        "SET session_replication_role = replica;",
        "SET SESSION session_replication_role TO replica;",
        "BEGIN; SET LOCAL session_replication_role = replica; COMMIT;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "committed replica execution mode must invalidate the runtime-role safety contract: {final_sql}",
        );
    }
}

#[test]
fn committed_set_config_replica_mode_cannot_bypass_runtime_trigger_enforcement() {
    for final_sql in [
        "SELECT set_config('session_replication_role', 'replica', false);",
        "SELECT pg_catalog . set_config('session_replication_role', 'replica', false);",
        "BEGIN; SELECT set_config('session_replication_role', 'replica', true); COMMIT;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "committed set_config replica mode must invalidate runtime-role safety: {final_sql}",
        );
    }
}

#[test]
fn committed_pg_settings_replica_mode_cannot_bypass_runtime_trigger_enforcement() {
    for final_sql in [
        "UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';",
        "UPDATE pg_catalog . pg_settings SET setting='replica' WHERE name='session_replication_role';",
        "UPDATE pg_settings SET setting = lower('REPLICA') WHERE name = 'session_replication_role';",
        "UPDATE pg_settings AS p SET setting = 'replica' WHERE p . name = 'session_replication_role';",
        "UPDATE ONLY pg_catalog . pg_settings AS p SET setting = lower('REPLICA') WHERE p.name = 'session_replication_role';",
        "WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';",
        "WITH changed_setting AS (UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role' RETURNING name) SELECT count(*) FROM changed_setting;",
        "WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = (SELECT 'replica' WHERE true) WHERE name = 'session_replication_role';",
        "WITH changed_setting AS (UPDATE pg_settings SET setting = (SELECT 'replica' WHERE true) WHERE name = 'session_replication_role' RETURNING name) SELECT count(*) FROM changed_setting;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "committed pg_settings mutation must not suppress ordinary trigger enforcement: {final_sql}",
        );
    }
}

#[test]
fn rolled_back_replica_execution_mode_does_not_change_durable_migration_effects() {
    for final_sql in [
        "BEGIN; SET LOCAL session_replication_role = replica; TRUNCATE TABLE source_artifact; ROLLBACK;",
        "BEGIN; SELECT set_config('session_replication_role', 'replica', true); TRUNCATE TABLE source_artifact; ROLLBACK;",
        "BEGIN; UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role'; TRUNCATE TABLE source_artifact; ROLLBACK;",
        "BEGIN; UPDATE pg_settings AS p SET setting = 'replica' WHERE p.name = 'session_replication_role'; ROLLBACK;",
        "BEGIN; WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role'; ROLLBACK;",
        "BEGIN; WITH changed_setting AS (UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role' RETURNING name) SELECT count(*) FROM changed_setting; ROLLBACK;",
        "BEGIN; WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = (SELECT 'replica' WHERE true) WHERE name = 'session_replication_role'; ROLLBACK;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn origin_and_local_execution_modes_preserve_ordinary_trigger_enforcement() {
    for final_sql in [
        "SET session_replication_role = origin;",
        "SET SESSION session_replication_role TO local;",
        "SELECT set_config('session_replication_role', 'origin', false);",
        "SELECT set_config('session_replication_role', 'local', false);",
        "UPDATE pg_settings SET setting = 'origin' WHERE name = 'session_replication_role';",
        "UPDATE pg_catalog . pg_settings SET setting = 'local' WHERE name = 'session_replication_role';",
        "UPDATE pg_settings AS p SET setting = 'origin' WHERE p.name = 'session_replication_role';",
        "WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = 'origin' WHERE name = 'session_replication_role';",
        "WITH changed_setting AS (UPDATE pg_settings SET setting = 'local' WHERE name = 'session_replication_role' RETURNING name) SELECT count(*) FROM changed_setting;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn unrelated_pg_settings_identity_or_parameter_does_not_impersonate_replica_mode_change() {
    for final_sql in [
        "UPDATE audit_support.pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';",
        "UPDATE pg_catalog_shadow.pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';",
        "UPDATE pg_settings SET setting = 'replica' WHERE name = 'application_name';",
        "WITH marker AS (SELECT 1) UPDATE audit_support.pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';",
        "WITH changed_setting AS (UPDATE pg_settings SET setting = 'replica' WHERE name = 'application_name' RETURNING name) SELECT count(*) FROM changed_setting;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn keyword_and_parameter_prefixes_do_not_impersonate_replica_mode_changes() {
    for final_sql in [
        "SETSESSION session_replication_role = replica;",
        "SET session_replication_role_shadow = replica;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn unrelated_function_identity_does_not_impersonate_the_postgresql_builtin() {
    for final_sql in [
        "SELECT audit_support.set_config('session_replication_role', 'replica', false);",
        "SELECT pg_catalog_shadow.set_config('session_replication_role', 'replica', false);",
        "SELECT myset_config('session_replication_role', 'replica', false);",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn marker_like_replication_role_text_is_not_an_execution_mode_change() {
    let catalog = embedded_with(
        r#"
SELECT 'SET session_replication_role = replica';
-- SET session_replication_role = replica;
SELECT $$SET LOCAL session_replication_role TO replica$$;
SELECT 'set_config(session_replication_role, replica, false)';
-- SELECT set_config('session_replication_role', 'replica', false);
SELECT 'UPDATE pg_settings SET setting = replica WHERE name = session_replication_role';
-- UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';
SELECT 'WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = replica WHERE name = session_replication_role';
-- WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
