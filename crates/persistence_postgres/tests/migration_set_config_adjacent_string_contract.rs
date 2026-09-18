use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn embedded_with(final_sql: &str) -> MigrationCatalog {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migrations must load");
    MigrationCatalog::from_sql(
        &format!("{}\n{final_sql}", embedded.up_sql()),
        embedded.down_sql(),
    )
}

#[test]
fn newline_concatenated_setting_name_cannot_bypass_replication_role_guard() {
    for final_sql in [
        "SELECT set_config(\n  'session_'\n  'replication_role',\n  'replica',\n  false\n);",
        "SELECT pg_catalog . set_config(\n  new_value => 'replica',\n  setting_name => 'session_'\n                  'replication_role',\n  is_local => false\n);",
    ] {
        assert_eq!(
            validate_migration_catalog(&embedded_with(final_sql)),
            Err(MigrationContractError::MissingAppRuntimeRole),
            "PostgreSQL newline-concatenated string constants must not hide session_replication_role: {final_sql}",
        );
    }
}

#[test]
fn rolled_back_newline_concatenated_setting_name_is_not_durable() {
    let final_sql = "BEGIN; SELECT set_config(\n  'session_'\n  'replication_role',\n  'replica',\n  true\n); ROLLBACK;";
    assert_eq!(validate_migration_catalog(&embedded_with(final_sql)), Ok(()));
}
