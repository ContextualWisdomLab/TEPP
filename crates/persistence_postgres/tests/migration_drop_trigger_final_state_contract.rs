use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn committed_drop_trigger_cannot_reuse_historical_create_trigger_evidence() {
    for final_sql in [
        "DROP TRIGGER source_artifact_reject_mutation ON source_artifact;",
        "DROP TRIGGER IF EXISTS source_artifact_reject_mutation ON source_artifact RESTRICT;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::UnsupportedTableFinalStateMutation),
            "committed DROP TRIGGER must invalidate historical trigger evidence: {final_sql}",
        );
    }
}

#[test]
fn rolled_back_drop_trigger_does_not_change_durable_trigger_state() {
    let catalog = embedded_with(
        "BEGIN; DROP TRIGGER source_artifact_reject_mutation ON source_artifact; ROLLBACK;",
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn marker_like_drop_trigger_text_is_not_a_statement() {
    let catalog = embedded_with(
        "SELECT 'DROP TRIGGER source_artifact_reject_mutation ON source_artifact'; -- DROP TRIGGER audit_event_reject_mutation ON audit_event",
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
