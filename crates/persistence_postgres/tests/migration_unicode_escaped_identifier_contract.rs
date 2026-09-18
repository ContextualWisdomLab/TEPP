use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn unicode_escaped_pg_settings_identity_cannot_bypass_replication_role_guard() {
    for final_sql in [
        r#"UPDATE U&"pg_settings" SET setting = 'replica' WHERE name = 'session_replication_role';"#,
        r#"UPDATE U&"pg_\0073ettings" SET setting = 'replica' WHERE name = 'session_replication_role';"#,
        r#"UPDATE U&"pg_!0073ettings" UESCAPE '!' SET setting = 'replica' WHERE name = 'session_replication_role';"#,
        r#"UPDATE pg_catalog . U&"pg_settings" SET setting = 'replica' WHERE name = 'session_replication_role';"#,
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "Unicode-escaped canonical pg_settings identity must not bypass trigger enforcement: {final_sql}",
        );
    }
}

#[test]
fn rolled_back_unicode_escaped_pg_settings_mutation_is_not_durable() {
    let final_sql = r#"BEGIN; UPDATE U&"pg_settings" SET setting = 'replica' WHERE name = 'session_replication_role'; ROLLBACK;"#;
    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}

#[test]
fn unicode_escaped_identifier_markers_in_opaque_regions_are_inert() {
    let catalog = embedded_with(
        r#"
SELECT 'UPDATE U&"pg_settings" SET setting = replica WHERE name = session_replication_role';
-- UPDATE U&"pg_settings" SET setting = 'replica' WHERE name = 'session_replication_role';
SELECT $$UPDATE U&"pg_settings" SET setting = 'replica' WHERE name = 'session_replication_role'$$;
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
