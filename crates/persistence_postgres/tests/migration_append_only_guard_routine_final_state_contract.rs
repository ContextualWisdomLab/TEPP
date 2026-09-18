use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn committed_guard_routine_removal_cannot_reuse_historical_append_only_evidence() {
    for final_sql in [
        "DROP FUNCTION reject_append_only_mutation() CASCADE;",
        "DROP FUNCTION IF EXISTS reject_append_only_mutation() CASCADE;",
        "DROP ROUTINE reject_append_only_mutation() CASCADE;",
        "DROP ROUTINE IF EXISTS reject_append_only_mutation() CASCADE;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::UnsupportedTableFinalStateMutation),
            "committed guard-routine removal must invalidate historical trigger evidence: {final_sql}",
        );
    }
}

#[test]
fn committed_guard_routine_replacement_cannot_reuse_the_original_definition() {
    let catalog = embedded_with(
        r#"
CREATE OR REPLACE FUNCTION reject_append_only_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $tepp$
BEGIN
    RETURN NULL;
END
$tepp$;
"#,
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::UnsupportedTableFinalStateMutation),
    );
}

#[test]
fn rolled_back_guard_routine_mutations_do_not_change_durable_enforcement() {
    for final_sql in [
        "BEGIN; DROP FUNCTION reject_append_only_mutation() CASCADE; ROLLBACK;",
        "BEGIN; DROP ROUTINE reject_append_only_mutation() CASCADE; ROLLBACK;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }

    let replaced = embedded_with(
        r#"
BEGIN;
CREATE OR REPLACE FUNCTION reject_append_only_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $tepp$
BEGIN
    RETURN NULL;
END
$tepp$;
ROLLBACK;
"#,
    );
    assert_eq!(validate_migration_catalog(&replaced), Ok(()));
}

#[test]
fn marker_like_guard_routine_mutations_are_not_statements() {
    let catalog = embedded_with(
        r#"
SELECT 'DROP FUNCTION reject_append_only_mutation() CASCADE';
SELECT 'DROP ROUTINE reject_append_only_mutation() CASCADE';
SELECT $$CREATE OR REPLACE FUNCTION reject_append_only_mutation()$$;
-- DROP FUNCTION reject_append_only_mutation() CASCADE;
-- DROP ROUTINE reject_append_only_mutation() CASCADE;
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
