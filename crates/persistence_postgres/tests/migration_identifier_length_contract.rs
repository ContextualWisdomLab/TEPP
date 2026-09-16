use persistence_postgres::{
    MigrationCatalog, MigrationContractError, validate_migration_catalog,
};

fn catalog_with_table(table_name: &str) -> MigrationCatalog {
    MigrationCatalog::from_sql(
        &format!(
            "CREATE TABLE {table_name} (\n                tenant_record_id uuid NOT NULL,\n                system_time timestamptz NOT NULL,\n                valid_from timestamptz NOT NULL\n            );"
        ),
        &format!("DROP TABLE {table_name};"),
    )
}

#[test]
fn postgres_identifier_byte_limit_is_enforced_before_server_truncation() {
    let maximum_length_name =
        "document_record_projection_snapshot_archive_registry_history_v1";
    let truncated_by_postgres =
        "document_record_projection_snapshot_archive_registry_history_v1x";

    assert_eq!(maximum_length_name.len(), 63);
    assert_eq!(truncated_by_postgres.len(), 64);
    assert_eq!(
        validate_migration_catalog(&catalog_with_table(maximum_length_name)),
        Ok(()),
        "the PostgreSQL default 63-byte identifier boundary remains admissible"
    );
    assert_eq!(
        validate_migration_catalog(&catalog_with_table(truncated_by_postgres)),
        Err(MigrationContractError::SingleWordObjectName),
        "TEPP must reject identifiers PostgreSQL would silently truncate"
    );
}
