use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn committed_trigger_weakening_modes_fail_closed() {
    for final_sql in [
        "ALTER TABLE source_artifact DISABLE TRIGGER source_artifact_reject_mutation;",
        "ALTER TABLE source_artifact DISABLE TRIGGER USER;",
        "ALTER TABLE source_artifact DISABLE TRIGGER ALL;",
        "ALTER TABLE source_artifact ENABLE REPLICA TRIGGER source_artifact_reject_mutation;",
        "ALTER TABLE source_artifact ADD COLUMN auxiliary_flag boolean, DISABLE TRIGGER source_artifact_reject_mutation;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::UnsupportedTableFinalStateMutation),
            "committed trigger weakening must not reuse historical CREATE TRIGGER evidence: {final_sql}",
        );
    }
}

#[test]
fn rolled_back_trigger_weakening_does_not_change_durable_state() {
    for final_sql in [
        "BEGIN; ALTER TABLE source_artifact DISABLE TRIGGER source_artifact_reject_mutation; ROLLBACK;",
        "BEGIN; ALTER TABLE source_artifact ENABLE REPLICA TRIGGER source_artifact_reject_mutation; ROLLBACK;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Ok(()),
            "rolled-back trigger mode must not alter durable enforcement: {final_sql}",
        );
    }
}

#[test]
fn ordinary_and_always_enable_modes_remain_supported() {
    for final_sql in [
        "ALTER TABLE source_artifact ENABLE TRIGGER source_artifact_reject_mutation;",
        "ALTER TABLE source_artifact ENABLE ALWAYS TRIGGER source_artifact_reject_mutation;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Ok(()),
            "non-weakening trigger enablement must remain supported: {final_sql}",
        );
    }
}
