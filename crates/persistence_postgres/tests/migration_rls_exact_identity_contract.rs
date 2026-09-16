use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

fn rls_catalog(policy_predicate: &str, tenant_setting: &str) -> MigrationCatalog {
    let up_sql = format!(
        r"
        CREATE TABLE document_record (
            document_record_id uuid PRIMARY KEY,
            tenant_record_id uuid NOT NULL,
            tenant_record_id_shadow uuid NOT NULL,
            system_time timestamptz NOT NULL,
            available_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        ALTER TABLE document_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE document_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY document_record_tenant_isolation ON document_record
            FOR ALL
            USING (
                {policy_predicate}::text = nullif(current_setting('{tenant_setting}', true), '')
            )
            WITH CHECK (
                {policy_predicate}::text = nullif(current_setting('{tenant_setting}', true), '')
            );
        "
    );
    MigrationCatalog::from_sql(&up_sql, "DROP TABLE document_record;")
}

#[test]
fn tenant_policy_requires_the_exact_tenant_record_id_identifier() {
    let catalog = rls_catalog(
        "tenant_record_id_shadow",
        "tepp.current_tenant_record_id",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}

#[test]
fn tenant_policy_requires_the_exact_session_guc_key() {
    let catalog = rls_catalog(
        "tenant_record_id",
        "tepp.current_tenant_record_id_shadow",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingTenantSessionGuc)
    );
}

#[test]
fn tenant_guc_literal_without_current_setting_does_not_satisfy_the_contract() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE document_record (
            document_record_id uuid PRIMARY KEY,
            tenant_record_id uuid NOT NULL,
            system_time timestamptz NOT NULL,
            available_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        SELECT 'tepp.current_tenant_record_id';
        ALTER TABLE document_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE document_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY document_record_tenant_isolation ON document_record
            FOR ALL
            USING (tenant_record_id IS NOT NULL)
            WITH CHECK (tenant_record_id IS NOT NULL);
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingTenantSessionGuc)
    );
}

#[test]
fn tenant_setting_call_outside_policy_cannot_cover_policy() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE document_record (
            document_record_id uuid PRIMARY KEY,
            tenant_record_id uuid NOT NULL,
            system_time timestamptz NOT NULL,
            available_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        SELECT current_setting('tepp.current_tenant_record_id', true);
        ALTER TABLE document_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE document_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY document_record_tenant_isolation ON document_record
            FOR ALL
            USING (tenant_record_id IS NOT NULL)
            WITH CHECK (tenant_record_id IS NOT NULL);
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}

#[test]
fn restrictive_supplemental_policy_need_not_repeat_the_tenant_session_predicate() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE document_record (
            document_record_id uuid PRIMARY KEY,
            tenant_record_id uuid NOT NULL,
            system_time timestamptz NOT NULL,
            available_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        ALTER TABLE document_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE document_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY document_record_tenant_isolation ON document_record
            FOR ALL
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            )
            WITH CHECK (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            );
        CREATE POLICY document_record_visibility_guard ON document_record
            AS RESTRICTIVE
            FOR SELECT
            USING (document_record_id IS NOT NULL);
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}

#[test]
fn restrictive_alias_inside_using_does_not_change_permissive_policy_composition() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE document_record (
            document_record_id uuid PRIMARY KEY,
            tenant_record_id uuid NOT NULL,
            system_time timestamptz NOT NULL,
            available_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        SELECT current_setting('tepp.current_tenant_record_id', true);
        ALTER TABLE document_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE document_record FORCE ROW LEVEL SECURITY;
        CREATE POLICY document_record_visibility_guard ON document_record
            FOR SELECT
            USING (
                tenant_record_id IS NOT NULL
                AND EXISTS (SELECT 1 AS restrictive)
            );
        ",
        "DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}

#[test]
fn tenant_policy_on_a_longer_table_name_cannot_cover_a_prefix_table() {
    let catalog = MigrationCatalog::from_sql(
        r"
        CREATE TABLE document_record (
            document_record_id uuid PRIMARY KEY,
            tenant_record_id uuid NOT NULL,
            system_time timestamptz NOT NULL,
            available_time timestamptz NOT NULL
        );
        CREATE TABLE document_record_archive (
            document_record_archive_id uuid PRIMARY KEY,
            tenant_record_id uuid NOT NULL,
            system_time timestamptz NOT NULL,
            available_time timestamptz NOT NULL
        );
        CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
        ALTER TABLE document_record ENABLE ROW LEVEL SECURITY;
        ALTER TABLE document_record FORCE ROW LEVEL SECURITY;
        ALTER TABLE document_record_archive ENABLE ROW LEVEL SECURITY;
        ALTER TABLE document_record_archive FORCE ROW LEVEL SECURITY;
        CREATE POLICY document_record_archive_tenant_isolation ON document_record_archive
            FOR ALL
            USING (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            )
            WITH CHECK (
                tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
            );
        ",
        "DROP TABLE document_record_archive; DROP TABLE document_record;",
    );

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}
