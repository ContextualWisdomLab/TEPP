use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn committed_role_replica_login_defaults_cannot_bypass_runtime_trigger_enforcement() {
    for final_sql in [
        "ALTER ROLE tepp_app_runtime SET session_replication_role = replica;",
        "ALTER ROLE tepp_app_runtime IN DATABASE tepp_database SET session_replication_role TO replica;",
        "ALTER USER tepp_app_runtime SET session_replication_role = replica;",
        "ALTER USER tepp_app_runtime IN DATABASE tepp_database SET session_replication_role TO replica;",
        "ALTER ROLE ALL SET session_replication_role = replica;",
        "ALTER USER ALL IN DATABASE tepp_database SET session_replication_role = replica;",
        "ALTER ROLE ALL IN DATABASE tepp_database SET session_replication_role TO replica;",
        "ALTER ROLE tepp_app_runtime SET session_replication_role FROM CURRENT;",
        "ALTER USER tepp_app_runtime SET session_replication_role FROM CURRENT;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "persistent role login default must not suppress ordinary trigger enforcement in later sessions: {final_sql}",
        );
    }
}

#[test]
fn committed_database_or_system_replica_defaults_fail_closed() {
    for final_sql in [
        "ALTER DATABASE tepp_database SET session_replication_role = replica;",
        "ALTER SYSTEM SET session_replication_role = replica;",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "persistent database or system default must not suppress ordinary trigger enforcement in later sessions: {final_sql}",
        );
    }
}

#[test]
fn ordinary_trigger_safe_login_defaults_remain_accepted() {
    for final_sql in [
        "ALTER ROLE tepp_app_runtime SET session_replication_role = origin;",
        "ALTER USER tepp_app_runtime SET session_replication_role = origin;",
        "ALTER ROLE tepp_app_runtime SET session_replication_role TO local;",
        "ALTER USER ALL IN DATABASE tepp_database SET session_replication_role = local;",
        "ALTER ROLE ALL IN DATABASE tepp_database SET session_replication_role = origin;",
        "ALTER DATABASE tepp_database SET session_replication_role = local;",
        "ALTER SYSTEM SET session_replication_role = origin;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn unrelated_persistent_defaults_do_not_impersonate_replication_role() {
    for final_sql in [
        "ALTER ROLE tepp_app_runtime SET application_name = 'replica';",
        "ALTER USER tepp_app_runtime SET application_name = 'replica';",
        "ALTER ROLE audit_runtime SET session_replication_role = replica;",
        "ALTER USER audit_runtime SET session_replication_role = replica;",
        "ALTER USER MAPPING FOR tepp_app_runtime SERVER foreign_server OPTIONS (SET user 'replica');",
        "ALTER DATABASE tepp_database SET application_name = 'replica';",
        "ALTER SYSTEM SET application_name = 'replica';",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn rolled_back_role_or_database_default_mutation_is_not_durable() {
    for final_sql in [
        "BEGIN; ALTER ROLE tepp_app_runtime SET session_replication_role = replica; ROLLBACK;",
        "BEGIN; ALTER USER tepp_app_runtime SET session_replication_role = replica; ROLLBACK;",
        "BEGIN; ALTER ROLE ALL IN DATABASE tepp_database SET session_replication_role = replica; ROLLBACK;",
        "BEGIN; ALTER DATABASE tepp_database SET session_replication_role = replica; ROLLBACK;",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn marker_like_persistent_default_text_is_not_a_configuration_change() {
    let catalog = embedded_with(
        r#"
SELECT 'ALTER ROLE tepp_app_runtime SET session_replication_role = replica';
-- ALTER USER tepp_app_runtime SET session_replication_role = replica;
SELECT $$ALTER DATABASE tepp_database SET session_replication_role = replica$$;
SELECT 'ALTER SYSTEM SET session_replication_role = replica';
"#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
