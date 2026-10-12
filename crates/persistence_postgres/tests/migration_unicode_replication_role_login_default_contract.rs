use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn unicode_escaped_runtime_role_identity_cannot_bypass_persistent_default_guard() {
    for final_sql in [
        r#"ALTER ROLE U&"tepp_app_runtime" SET session_replication_role = replica;"#,
        r#"ALTER USER U&"tepp_app_runtime" IN DATABASE tepp_database SET session_replication_role = replica;"#,
        r#"ALTER ROLE U&"tepp_app!005fruntime" UESCAPE '!' SET session_replication_role = replica;"#,
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "Unicode-escaped runtime role identity must not bypass persistent trigger-default enforcement: {final_sql}",
        );
    }
}

#[test]
fn unicode_escaped_replication_parameter_identity_is_protected_on_all_default_surfaces() {
    for final_sql in [
        r#"ALTER ROLE tepp_app_runtime SET U&"session_replication_role" = replica;"#,
        r#"ALTER USER tepp_app_runtime SET U&"session_replication_\0072ole" TO replica;"#,
        r#"ALTER DATABASE tepp_database SET U&"session_replication_role" = replica;"#,
        r#"ALTER SYSTEM SET U&"session_replication!005frole" UESCAPE '!' = replica;"#,
        r#"ALTER ROLE tepp_app_runtime RESET U&"session_replication_role";"#,
        r#"ALTER USER tepp_app_runtime RESET U&"session_replication_\0072ole";"#,
        r#"ALTER DATABASE tepp_database RESET U&"session_replication_role";"#,
        r#"ALTER SYSTEM RESET U&"session_replication!005frole" UESCAPE '!';"#,
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "Unicode-escaped session_replication_role identity must not bypass persistent defaults: {final_sql}",
        );
    }
}

#[test]
fn unicode_escaped_persistent_defaults_preserve_safe_and_unrelated_controls() {
    for final_sql in [
        r#"ALTER ROLE U&"tepp_app_runtime" SET U&"session_replication_role" = origin;"#,
        r#"ALTER USER U&"tepp_app_runtime" SET U&"session_replication_role" TO local;"#,
        r#"ALTER ROLE U&"audit_runtime" SET session_replication_role = replica;"#,
        r#"ALTER ROLE U&"current_user" SET session_replication_role = replica;"#,
        r#"ALTER ROLE U&"current_role" UESCAPE '!' SET session_replication_role = replica;"#,
        r#"ALTER USER U&"session_user" SET session_replication_role = replica;"#,
        r#"ALTER ROLE tepp_app_runtime SET U&"application_name" = replica;"#,
        r#"ALTER DATABASE tepp_database SET U&"application_name" = replica;"#,
        r#"ALTER SYSTEM SET U&"application_name" = replica;"#,
        r#"ALTER ROLE tepp_app_runtime RESET U&"application_name";"#,
        r#"ALTER DATABASE tepp_database RESET U&"application_name";"#,
        r#"ALTER SYSTEM RESET U&"application_name";"#,
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn rolled_back_unicode_persistent_default_is_not_durable() {
    for final_sql in [
        r#"BEGIN; ALTER ROLE U&"tepp_app_runtime" SET session_replication_role = replica; ROLLBACK;"#,
        r#"BEGIN; ALTER USER tepp_app_runtime SET U&"session_replication_role" = replica; ROLLBACK;"#,
        r#"BEGIN; ALTER DATABASE tepp_database SET U&"session_replication_role" = replica; ROLLBACK;"#,
        r#"BEGIN; ALTER ROLE tepp_app_runtime RESET U&"session_replication_role"; ROLLBACK;"#,
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn unicode_persistent_default_markers_in_opaque_regions_are_inert() {
    let catalog = embedded_with(
        r#"
SELECT 'ALTER ROLE U&"tepp_app_runtime" SET session_replication_role = replica';
-- ALTER USER tepp_app_runtime SET U&"session_replication_role" = replica;
SELECT $$ALTER DATABASE tepp_database SET U&"session_replication_role" = replica$$;
SELECT 'ALTER SYSTEM SET U&"session_replication_role" = replica';
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
