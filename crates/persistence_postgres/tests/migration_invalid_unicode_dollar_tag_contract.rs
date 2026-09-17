use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

#[test]
fn non_identifier_unicode_cannot_mask_declarations_as_dollar_quote_tags() {
    for tag in ["€", "😀"] {
        let up_sql = format!(
            "CREATE TABLE tenant_record (\n\
                 tenant_record_id uuid PRIMARY KEY,\n\
                 system_time timestamptz NOT NULL\n\
             );\n\
             SELECT ${tag}$ CREATE INDEX Bad ON tenant_record (tenant_record_id); ${tag}$;"
        );
        let catalog = MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;");

        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName),
            "non-identifier tag {tag} masked declaration-shaped SQL"
        );
    }
}
