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
fn rolled_back_replica_execution_mode_does_not_change_durable_migration_effects() {
    for final_sql in [
        "BEGIN; SET LOCAL session_replication_role = replica; TRUNCATE TABLE source_artifact; ROLLBACK;",
        "BEGIN; SELECT set_config('session_replication_role', 'replica', true); TRUNCATE TABLE source_artifact; ROLLBACK;",
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
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
