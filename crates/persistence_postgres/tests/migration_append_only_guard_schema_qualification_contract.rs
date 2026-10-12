use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn whitespace_separated_schema_qualification_cannot_hide_guard_routine_removal() {
    for final_sql in [
        "DROP FUNCTION public . reject_append_only_mutation() CASCADE;",
        "DROP ROUTINE IF EXISTS public . reject_append_only_mutation() CASCADE;",
        "DROP FUNCTION IF EXISTS unrelated_helper(), public . reject_append_only_mutation() CASCADE;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::UnsupportedTableFinalStateMutation),
            "schema qualification whitespace must not hide guard-routine removal: {final_sql}",
        );
    }
}

#[test]
fn whitespace_separated_schema_qualification_cannot_hide_guard_routine_replacement() {
    let catalog = embedded_with(
        r#"
CREATE OR REPLACE FUNCTION public . reject_append_only_mutation()
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
fn rolled_back_schema_qualified_guard_mutations_do_not_change_durable_enforcement() {
    for final_sql in [
        "BEGIN; DROP FUNCTION public . reject_append_only_mutation() CASCADE; ROLLBACK;",
        "BEGIN; DROP ROUTINE public . reject_append_only_mutation() CASCADE; ROLLBACK;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }

    let replaced = embedded_with(
        r#"
BEGIN;
CREATE OR REPLACE FUNCTION public . reject_append_only_mutation()
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
fn schema_qualified_guard_markers_inside_opaque_regions_are_not_mutations() {
    let catalog = embedded_with(
        r#"
SELECT 'DROP FUNCTION public . reject_append_only_mutation() CASCADE';
SELECT $$DROP ROUTINE public . reject_append_only_mutation() CASCADE$$;
-- CREATE OR REPLACE FUNCTION public . reject_append_only_mutation();
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
