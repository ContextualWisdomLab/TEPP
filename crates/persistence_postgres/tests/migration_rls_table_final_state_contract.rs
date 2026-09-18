use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn rls_catalog(final_mutations: &str) -> MigrationCatalog {
    let up_sql = format!(
        r#"
        CREATE TABLE tenant_record (
            tenant_record_id uuid PRIMARY KEY,
            system_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY tenant_record_tenant_isolation ON tenant_record
            FOR ALL
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            );
        {final_mutations}
        "#
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE tenant_record;")
}

#[test]
fn committed_disable_row_level_security_removes_final_enablement() {
    let catalog = rls_catalog("ALTER TABLE tenant_record DISABLE ROW LEVEL SECURITY;");
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsEnable),
        "historical ENABLE evidence must not survive a committed final DISABLE"
    );
}

#[test]
fn committed_no_force_row_level_security_removes_final_owner_enforcement() {
    let catalog = rls_catalog("ALTER TABLE tenant_record NO FORCE ROW LEVEL SECURITY;");
    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsEnable),
        "historical FORCE evidence must not survive a committed final NO FORCE"
    );
}

#[test]
fn rolled_back_disable_does_not_change_the_durable_rls_state() {
    let catalog = rls_catalog(
        "BEGIN; ALTER TABLE tenant_record DISABLE ROW LEVEL SECURITY; ROLLBACK;",
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn later_enable_and_force_restore_the_required_final_state() {
    let catalog = rls_catalog(
        r#"
        ALTER TABLE tenant_record DISABLE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record NO FORCE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
        "#,
    );
    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn whitespace_qualified_sibling_table_cannot_repair_disabled_rls_state() {
    let embedded = MigrationCatalog::from_embedded().expect("embedded catalog must load");
    let up_sql = format!(
        "{}\nALTER TABLE public . tenant_record DISABLE ROW LEVEL SECURITY;\nALTER TABLE public . event_instance ENABLE ROW LEVEL SECURITY;\nALTER TABLE public . event_instance FORCE ROW LEVEL SECURITY;",
        embedded.up_sql()
    );
    let catalog = MigrationCatalog::from_sql(&up_sql, embedded.down_sql());

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsEnable),
        "schema qualification must not collapse tenant_record and event_instance into one RLS state bucket"
    );
}

#[test]
fn rolled_back_whitespace_qualified_disable_remains_non_durable() {
    let embedded = MigrationCatalog::from_embedded().expect("embedded catalog must load");
    let up_sql = format!(
        "{}\nBEGIN; ALTER TABLE public . tenant_record DISABLE ROW LEVEL SECURITY; ROLLBACK;",
        embedded.up_sql()
    );
    let catalog = MigrationCatalog::from_sql(&up_sql, embedded.down_sql());

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
