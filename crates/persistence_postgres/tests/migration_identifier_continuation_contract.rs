use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn postgresql_identifier_continuations_cannot_truncate_to_valid_table_prefixes() {
    for table_name in ["tenant_record$shadow", "tenant_record$1", "tenant_record측정"] {
        let up_sql = format!(
            "CREATE TABLE {table_name} (\n\
                 tenant_record_id uuid PRIMARY KEY,\n\
                 system_time timestamptz NOT NULL\n\
             );"
        );
        let down_sql = format!("DROP TABLE {table_name};");
        let catalog = MigrationCatalog::from_sql(&up_sql, &down_sql);

        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "PostgreSQL identifier continuation was truncated for {table_name}"
        );
    }
}
