use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn unicode_escaped_set_parameter_identity_cannot_bypass_replication_role_guard() {
    for final_sql in [
        r#"SET U&"session_replication_role" = replica;"#,
        r#"SET SESSION U&"session_replication_\0072ole" TO replica;"#,
        r#"SET LOCAL U&"session_replication_!0072ole" UESCAPE '!' = replica;"#,
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "Unicode-escaped canonical SET parameter must not bypass trigger enforcement: {final_sql}",
        );
    }
}

#[test]
fn unicode_escaped_set_safe_modes_and_unrelated_parameter_remain_allowed() {
    for final_sql in [
        r#"SET U&"session_replication_role" = origin;"#,
        r#"SET SESSION U&"session_replication_\0072ole" TO local;"#,
        r#"SET U&"application_name" = replica;"#,
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn rolled_back_unicode_escaped_set_replica_mode_is_not_durable() {
    let final_sql = r#"BEGIN; SET U&"session_replication_role" = replica; ROLLBACK;"#;
    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}

#[test]
fn unicode_escaped_set_markers_in_opaque_regions_are_inert() {
    let catalog = embedded_with(
        r#"
SELECT 'SET U&"session_replication_role" = replica';
-- SET U&"session_replication_role" = replica;
SELECT $$SET U&"session_replication_role" = replica$$;
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
