use persistence_postgres::{MigrationCatalog, validate_migration_catalog};

#[test]
fn postgres_high_bit_dollar_quote_tags_mask_declaration_shaped_body_text() {
    for tag in ["측정", "€", "😀"] {
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
            Ok(()),
            "valid PostgreSQL dollar-quote tag {tag} leaked body SQL into validation"
        );
    }
}
