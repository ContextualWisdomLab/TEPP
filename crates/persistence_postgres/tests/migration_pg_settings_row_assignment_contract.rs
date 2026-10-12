use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn pg_settings_row_assignment_cannot_bypass_replica_guard() {
    for final_sql in [
        "UPDATE pg_settings SET (setting) = ('replica') WHERE name = 'session_replication_role';",
        "UPDATE pg_catalog . pg_settings AS p SET (setting) = ROW('replica') WHERE p.name = 'session_replication_role';",
        "WITH marker AS (SELECT 1) UPDATE ONLY pg_settings AS p SET (setting) = (SELECT 'replica') WHERE p.name = 'session_replication_role';",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "PostgreSQL row assignment to pg_settings.setting is still a configuration mutation: {final_sql}",
        );
    }
}

#[test]
fn safe_pg_settings_row_assignment_modes_remain_accepted() {
    for final_sql in [
        "UPDATE pg_settings SET (setting) = ('origin') WHERE name = 'session_replication_role';",
        "UPDATE pg_settings AS p SET (setting) = ROW('local') WHERE p.name = 'session_replication_role';",
    ] {
        assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
    }
}

#[test]
fn unrelated_pg_settings_row_assignment_remains_outside_the_guard() {
    let final_sql =
        "UPDATE pg_settings SET (setting) = ('replica') WHERE name = 'application_name';";
    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}

#[test]
fn rolled_back_pg_settings_row_assignment_is_non_durable() {
    let final_sql = "BEGIN; UPDATE pg_settings SET (setting) = ('replica') WHERE name = 'session_replication_role'; ROLLBACK;";
    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}
