use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn unicode_escaped_set_config_builtin_identity_cannot_bypass_replication_role_guard() {
    for final_sql in [
        r#"SELECT U&"set_\0063onfig"('session_replication_role', 'replica', false);"#,
        r#"SELECT U&"pg_\0063atalog".set_config('session_replication_role', 'replica', false);"#,
        r#"SELECT pg_catalog.U&"set_\0063onfig"('session_replication_role', 'replica', false);"#,
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "Unicode-escaped canonical set_config identity must not bypass trigger enforcement: {final_sql}",
        );
    }
}

#[test]
fn unicode_escaped_set_config_safe_values_remain_allowed() {
    for final_sql in [
        r#"SELECT U&"set_\0063onfig"('session_replication_role', 'origin', false);"#,
        r#"SELECT U&"pg_\0063atalog".set_config('session_replication_role', 'local', false);"#,
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn unrelated_unicode_escaped_function_identity_remains_unrelated() {
    for final_sql in [
        r#"SELECT U&"audit_set_config"('session_replication_role', 'replica', false);"#,
        r#"SELECT U&"audit_support".set_config('session_replication_role', 'replica', false);"#,
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn rolled_back_unicode_escaped_set_config_replica_mode_is_not_durable() {
    let final_sql = r#"BEGIN; SELECT U&"set_\0063onfig"('session_replication_role', 'replica', true); ROLLBACK;"#;
    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}

#[test]
fn unicode_escaped_set_config_markers_in_opaque_regions_are_inert() {
    let catalog = embedded_with(
        r#"
SELECT 'U&"set_\0063onfig"(''session_replication_role'', ''replica'', false)';
-- SELECT U&"set_\0063onfig"('session_replication_role', 'replica', false);
SELECT $$U&"set_\0063onfig"('session_replication_role', 'replica', false)$$;
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
