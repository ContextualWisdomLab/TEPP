use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn conforming_catalog(extra_sql: &str) -> MigrationCatalog {
    let up_sql = format!(
        "CREATE TABLE tenant_record (\n\
             tenant_record_id uuid PRIMARY KEY,\n\
             system_time timestamptz NOT NULL\n\
         );\n{extra_sql}"
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn concurrently_modifier_does_not_hide_a_valid_index_name() {
    for statement in [
        "CREATE INDEX CONCURRENTLY tenant_record_lookup_index ON tenant_record (tenant_record_id);",
        "CREATE INDEX CONCURRENTLY IF NOT EXISTS tenant_record_lookup_index ON tenant_record (tenant_record_id);",
        "CREATE UNIQUE INDEX CONCURRENTLY tenant_record_lookup_index ON tenant_record (tenant_record_id);",
        "CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS tenant_record_lookup_index ON tenant_record (tenant_record_id);",
    ] {
        let catalog = conforming_catalog(statement);
        assert_eq!(validate_migration_catalog(&catalog), Ok(()), "{statement}");
    }
}

#[test]
fn concurrently_modifier_is_recognized_after_statement_delimiter_without_whitespace() {
    let catalog = MigrationCatalog::from_sql(
        "CREATE TABLE tenant_record (tenant_record_id uuid PRIMARY KEY, system_time timestamptz NOT NULL);CREATE INDEX CONCURRENTLY tenant_record_lookup_index ON tenant_record (tenant_record_id);",
        "DROP TABLE tenant_record;",
    );

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn concurrently_modifier_cannot_hide_an_invalid_index_name() {
    for statement in [
        "CREATE INDEX CONCURRENTLY Bad ON tenant_record (tenant_record_id);",
        "CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS Bad ON tenant_record (tenant_record_id);",
    ] {
        let catalog = conforming_catalog(statement);
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "{statement}"
        );
    }
}
