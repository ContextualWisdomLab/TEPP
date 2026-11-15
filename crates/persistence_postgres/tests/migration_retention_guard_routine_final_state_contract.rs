use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn committed_retention_guard_removal_cannot_reuse_historical_retention_evidence() {
    for final_sql in [
        "DROP FUNCTION reject_held_evidence_deletion() CASCADE;",
        "DROP ROUTINE IF EXISTS public . reject_held_evidence_deletion() CASCADE;",
        "DROP FUNCTION reject_tombstoned_evidence_restore() CASCADE;",
        "DROP ROUTINE IF EXISTS public . reject_tombstoned_evidence_restore() CASCADE;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingRetentionLegalHold),
            "committed retention guard removal must invalidate historical retention evidence: {final_sql}",
        );
    }
}

#[test]
fn committed_retention_guard_replacement_cannot_reuse_the_original_definition() {
    for guard in [
        "reject_held_evidence_deletion",
        "reject_tombstoned_evidence_restore",
    ] {
        let catalog = embedded_with(&format!(
            r#"
CREATE OR REPLACE FUNCTION public . {guard}()
RETURNS trigger
LANGUAGE plpgsql
AS $tepp$
BEGIN
    RETURN NEW;
END
$tepp$;
"#,
        ));
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::MissingRetentionLegalHold),
            "committed replacement must invalidate historical retention guard evidence: {guard}",
        );
    }
}

#[test]
fn unrelated_schema_routine_mutations_do_not_target_public_retention_guards() {
    let dropped = embedded_with(
        "DROP FUNCTION audit_support.reject_held_evidence_deletion() CASCADE;",
    );
    assert_eq!(validate_migration_catalog(&dropped), Ok(()));

    let replaced = embedded_with(
        r#"
CREATE OR REPLACE FUNCTION audit_support . reject_tombstoned_evidence_restore()
RETURNS trigger
LANGUAGE plpgsql
AS $tepp$
BEGIN
    RETURN NEW;
END
$tepp$;
"#,
    );
    assert_eq!(validate_migration_catalog(&replaced), Ok(()));
}

#[test]
fn rolled_back_retention_guard_mutations_do_not_change_durable_enforcement() {
    for final_sql in [
        "BEGIN; DROP FUNCTION reject_held_evidence_deletion() CASCADE; ROLLBACK;",
        "BEGIN; DROP ROUTINE reject_tombstoned_evidence_restore() CASCADE; ROLLBACK;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }

    let replaced = embedded_with(
        r#"
BEGIN;
CREATE OR REPLACE FUNCTION reject_held_evidence_deletion()
RETURNS trigger
LANGUAGE plpgsql
AS $tepp$
BEGIN
    RETURN NEW;
END
$tepp$;
ROLLBACK;
"#,
    );
    assert_eq!(validate_migration_catalog(&replaced), Ok(()));
}

#[test]
fn marker_like_retention_guard_mutations_are_not_statements() {
    let catalog = embedded_with(
        r#"
SELECT 'DROP FUNCTION reject_held_evidence_deletion() CASCADE';
SELECT $$CREATE OR REPLACE FUNCTION reject_tombstoned_evidence_restore()$$;
-- DROP ROUTINE reject_tombstoned_evidence_restore() CASCADE;
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
