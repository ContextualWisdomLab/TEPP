use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn committed_do_blocks_fail_closed_when_execution_context_is_opaque() {
    for final_sql in [
        r#"
DO $$
BEGIN
    PERFORM set_config('session_replication_role', 'replica', false);
END
$$;
"#,
        r#"
DO LANGUAGE plpgsql $$
BEGIN
    PERFORM set_config('session_replication_role', 'replica', false);
END
$$;
"#,
        r#"
DO $$
BEGIN
    PERFORM set_config('session_replication_role', 'replica', false);
END
$$ LANGUAGE plpgsql;
"#,
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "committed DO body must fail closed because immediate procedural execution is opaque: {final_sql}",
        );
    }
}

#[test]
fn rolled_back_do_block_does_not_change_durable_migration_effects() {
    let final_sql = r#"
BEGIN;
DO $$
BEGIN
    PERFORM set_config('session_replication_role', 'replica', true);
END
$$;
ROLLBACK;
"#;

    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}

#[test]
fn dollar_quoted_or_comment_marker_text_does_not_impersonate_an_executed_do_block() {
    let final_sql = r#"
SELECT $marker$DO $$ BEGIN PERFORM set_config('session_replication_role', 'replica', false); END $$;$marker$;
SELECT 'DO LANGUAGE plpgsql';
-- DO $$ BEGIN PERFORM set_config('session_replication_role', 'replica', false); END $$;
"#;

    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}

#[test]
fn opaque_function_definition_is_not_immediate_do_execution() {
    let final_sql = r#"
CREATE FUNCTION audit_support_helper()
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
    PERFORM set_config('session_replication_role', 'replica', false);
END
$$;
"#;

    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}
