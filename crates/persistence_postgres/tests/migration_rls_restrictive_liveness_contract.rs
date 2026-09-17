use persistence_postgres::{MigrationCatalog, MigrationContractError, validate_migration_catalog};

const TABLE_AND_ROLE: &str = r"
CREATE TABLE document_record (
    document_record_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL
);
CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS;
ALTER TABLE document_record ENABLE ROW LEVEL SECURITY;
ALTER TABLE document_record FORCE ROW LEVEL SECURITY;
";

const TENANT_BINDING: &str =
    "tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')";

fn catalog_with_policies(policies: &str) -> MigrationCatalog {
    MigrationCatalog::from_sql(
        &format!("{TABLE_AND_ROLE}\n{policies}"),
        "DROP TABLE document_record;",
    )
}

#[test]
fn restrictive_only_policy_cannot_satisfy_the_tenant_access_contract() {
    let catalog = catalog_with_policies(&format!(
        r"
CREATE POLICY document_record_tenant_guard ON document_record
    AS RESTRICTIVE
    FOR ALL
    USING ({TENANT_BINDING})
    WITH CHECK ({TENANT_BINDING});
"
    ));

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}

#[test]
fn restrictive_policy_requires_permissive_coverage_for_the_same_command() {
    let catalog = catalog_with_policies(&format!(
        r"
CREATE POLICY document_record_insert_isolation ON document_record
    AS PERMISSIVE
    FOR INSERT
    WITH CHECK ({TENANT_BINDING});
CREATE POLICY document_record_read_guard ON document_record
    AS RESTRICTIVE
    FOR SELECT
    USING (document_record_id IS NOT NULL);
"
    ));

    assert_eq!(
        validate_migration_catalog(&catalog),
        Err(MigrationContractError::MissingRlsPolicy)
    );
}

#[test]
fn restrictive_policy_may_narrow_matching_permissive_access() {
    let catalog = catalog_with_policies(&format!(
        r"
CREATE POLICY document_record_read_isolation ON document_record
    AS PERMISSIVE
    FOR SELECT
    USING ({TENANT_BINDING});
CREATE POLICY document_record_read_guard ON document_record
    AS RESTRICTIVE
    FOR SELECT
    USING (document_record_id IS NOT NULL);
"
    ));

    assert_eq!(validate_migration_catalog(&catalog), Ok(()));
}
