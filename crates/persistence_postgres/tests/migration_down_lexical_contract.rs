use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn malformed_rollback_sql_fails_closed_at_the_same_lexical_boundary() {
    let embedded = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    let malformed = MigrationCatalog::from_sql(
        embedded.up_sql(),
        "DROP TABLE tenant_record; /* unterminated rollback comment",
    );

    assert_eq!(
        validate_migration_catalog(&malformed),
        Err(MigrationContractError::EmptyMigrationSql)
    );
}
