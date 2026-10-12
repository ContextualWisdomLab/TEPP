use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn committed_call_statements_fail_closed_when_procedure_effects_are_unproven() {
    for final_sql in [
        "CALL disable_enforcement();",
        "CALL audit_support.disable_enforcement();",
        "CALL audit_support.disable_enforcement(mode => 'replica');",
        "SELECT 1;CALL audit_support.disable_enforcement();",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "committed CALL must fail closed because called procedure effects are not owned: {final_sql}",
        );
    }
}

#[test]
fn rolled_back_call_does_not_change_durable_migration_effects() {
    let final_sql = r#"
BEGIN;
CALL audit_support.disable_enforcement();
ROLLBACK;
"#;

    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}

#[test]
fn call_marker_text_in_opaque_regions_is_not_executed_procedure_evidence() {
    let final_sql = r#"
SELECT 'CALL audit_support.disable_enforcement()';
SELECT $marker$CALL audit_support.disable_enforcement();$marker$;
-- CALL audit_support.disable_enforcement();
"#;

    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}

#[test]
fn opaque_procedure_definition_is_not_immediate_call_execution() {
    let final_sql = r#"
CREATE PROCEDURE audit_support_helper()
LANGUAGE plpgsql
AS $$
BEGIN
    PERFORM set_config('session_replication_role', 'replica', false);
END
$$;
"#;

    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}
