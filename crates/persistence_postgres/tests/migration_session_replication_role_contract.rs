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
fn rolled_back_replica_execution_mode_does_not_change_durable_migration_effects() {
    let catalog = embedded_with(
        "BEGIN; SET LOCAL session_replication_role = replica; TRUNCATE TABLE source_artifact; ROLLBACK;",
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn origin_and_local_execution_modes_preserve_ordinary_trigger_enforcement() {
    for final_sql in [
        "SET session_replication_role = origin;",
        "SET SESSION session_replication_role TO local;",
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
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
