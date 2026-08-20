//! Embedded migration catalog and fail-closed SQL contracts.

use crate::MigrationContractError;
use crate::naming::is_multi_word_snake_case;
use std::collections::BTreeSet;

const FOUNDATION_UP: &str = include_str!("../../../migrations/0001_bitemporal_foundation.up.sql");
const FOUNDATION_DOWN: &str =
    include_str!("../../../migrations/0001_bitemporal_foundation.down.sql");
const RLS_UP: &str = include_str!("../../../migrations/0002_tenant_row_level_security.up.sql");
const RLS_DOWN: &str = include_str!("../../../migrations/0002_tenant_row_level_security.down.sql");
const MODEL_RUN_UP: &str = include_str!("../../../migrations/0003_model_run_artifact_chain.up.sql");
const MODEL_RUN_DOWN: &str =
    include_str!("../../../migrations/0003_model_run_artifact_chain.down.sql");
const APPEND_ONLY_UP: &str =
    include_str!("../../../migrations/0004_append_only_immutability_triggers.up.sql");
const APPEND_ONLY_DOWN: &str =
    include_str!("../../../migrations/0004_append_only_immutability_triggers.down.sql");
const TEMPORAL_ORDER_UP: &str =
    include_str!("../../../migrations/0005_temporal_interval_ordering.up.sql");
const TEMPORAL_ORDER_DOWN: &str =
    include_str!("../../../migrations/0005_temporal_interval_ordering.down.sql");
const MEMBERSHIP_UP: &str =
    include_str!("../../../migrations/0006_typed_membership_assignment.up.sql");
const MEMBERSHIP_DOWN: &str =
    include_str!("../../../migrations/0006_typed_membership_assignment.down.sql");
const RETENTION_UP: &str =
    include_str!("../../../migrations/0007_retention_deletion_legal_hold.up.sql");
const RETENTION_DOWN: &str =
    include_str!("../../../migrations/0007_retention_deletion_legal_hold.down.sql");

/// Forward and rollback SQL for one migration unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationCatalog {
    up_sql: String,
    down_sql: String,
}

impl MigrationCatalog {
    /// Load the embedded foundation and tenant RLS migrations shipped with this crate.
    ///
    /// # Errors
    ///
    /// Returns [`MigrationContractError::EmptyMigrationSql`] when embedded
    /// sources are unexpectedly empty.
    pub fn from_embedded() -> Result<Self, MigrationContractError> {
        let up_sql = format!(
            "{FOUNDATION_UP}\n{RLS_UP}\n{MODEL_RUN_UP}\n{APPEND_ONLY_UP}\n{TEMPORAL_ORDER_UP}\n{MEMBERSHIP_UP}\n{RETENTION_UP}"
        );
        let down_sql = format!(
            "{RETENTION_DOWN}\n{MEMBERSHIP_DOWN}\n{TEMPORAL_ORDER_DOWN}\n{APPEND_ONLY_DOWN}\n{MODEL_RUN_DOWN}\n{RLS_DOWN}\n{FOUNDATION_DOWN}"
        );
        Self::from_sources(&up_sql, &down_sql)
    }

    fn from_sources(up_sql: &str, down_sql: &str) -> Result<Self, MigrationContractError> {
        if up_sql.trim().is_empty() || down_sql.trim().is_empty() {
            return Err(MigrationContractError::EmptyMigrationSql);
        }
        Ok(Self {
            up_sql: up_sql.to_owned(),
            down_sql: down_sql.to_owned(),
        })
    }

    /// Construct a catalog from raw SQL strings (used by contract tests).
    #[must_use]
    pub fn from_sql(up_sql: &str, down_sql: &str) -> Self {
        Self {
            up_sql: up_sql.to_owned(),
            down_sql: down_sql.to_owned(),
        }
    }

    /// Borrow the forward migration SQL.
    #[must_use]
    pub fn up_sql(&self) -> &str {
        &self.up_sql
    }

    /// Borrow the rollback migration SQL.
    #[must_use]
    pub fn down_sql(&self) -> &str {
        &self.down_sql
    }
}

/// Validate migration SQL against TEPP persistence contracts.
///
/// When the catalog declares row-level security, every tenant-scoped table must
/// enable RLS and name multi-word isolation policies.
///
/// # Errors
///
/// Returns naming, tenant, temporal, RLS, or emptiness failures.
pub fn validate_migration_catalog(
    catalog: &MigrationCatalog,
) -> Result<(), MigrationContractError> {
    if catalog.up_sql.trim().is_empty() || catalog.down_sql.trim().is_empty() {
        return Err(MigrationContractError::EmptyMigrationSql);
    }

    let tables = parse_create_table_names(catalog.up_sql());
    if tables.is_empty() {
        return Err(MigrationContractError::EmptyMigrationSql);
    }

    for table in &tables {
        if !is_multi_word_snake_case(table) {
            return Err(MigrationContractError::SingleWordObjectName);
        }
        let body =
            table_body(catalog.up_sql(), table).ok_or(MigrationContractError::EmptyMigrationSql)?;
        validate_table_body(table, body)?;
    }

    if declares_row_level_security(catalog.up_sql()) {
        validate_tenant_rls_contract(catalog.up_sql(), &tables)?;
    }
    if declares_append_only_immutability(catalog.up_sql()) {
        validate_append_only_immutability(catalog.up_sql())?;
    }
    if declares_temporal_interval_ordering(catalog.up_sql()) {
        validate_temporal_interval_ordering(catalog.up_sql())?;
    }
    if declares_retention_legal_hold(catalog.up_sql()) {
        validate_retention_legal_hold(catalog.up_sql())?;
    }

    Ok(())
}

fn declares_append_only_immutability(up_sql: &str) -> bool {
    let lower = up_sql.to_ascii_lowercase();
    lower.contains("reject_append_only_mutation") || lower.contains("_reject_mutation")
}

fn validate_append_only_immutability(up_sql: &str) -> Result<(), MigrationContractError> {
    let lower = up_sql.to_ascii_lowercase();
    if !lower.contains("create or replace function reject_append_only_mutation") {
        return Err(MigrationContractError::MissingAppendOnlyTrigger);
    }
    let required = [
        "source_artifact",
        "audit_event",
        "reproducibility_manifest",
        "corpus_split_manifest",
        "model_run",
        "model_artifact",
    ];
    for table in required {
        let trigger = format!("{table}_reject_mutation");
        if !lower.contains(&format!("create trigger {trigger}")) {
            return Err(MigrationContractError::MissingAppendOnlyTrigger);
        }
        if !lower.contains(&format!("revoke update, delete on table {table}")) {
            return Err(MigrationContractError::MissingAppendOnlyTrigger);
        }
    }
    Ok(())
}

fn declares_temporal_interval_ordering(up_sql: &str) -> bool {
    let lower = up_sql.to_ascii_lowercase();
    lower.contains("_valid_order") || lower.contains("_system_order")
}

fn validate_temporal_interval_ordering(up_sql: &str) -> Result<(), MigrationContractError> {
    let lower = up_sql.to_ascii_lowercase();
    let required = [
        "document_record_valid_order",
        "document_record_system_order",
        "document_record_revision_positive",
        "event_instance_valid_order",
        "event_instance_system_order",
        "membership_assignment_valid_order",
    ];
    for constraint in required {
        if !lower.contains(&format!("constraint {constraint}")) {
            return Err(MigrationContractError::MissingTemporalIntervalConstraint);
        }
    }
    if !lower.contains("valid_to is null or valid_from <=") {
        return Err(MigrationContractError::MissingTemporalIntervalConstraint);
    }
    if !lower.contains("system_to is null or system_from <=") {
        return Err(MigrationContractError::MissingTemporalIntervalConstraint);
    }
    if !lower.contains("revision_number > 0") {
        return Err(MigrationContractError::MissingTemporalIntervalConstraint);
    }
    Ok(())
}

fn declares_retention_legal_hold(up_sql: &str) -> bool {
    let lower = up_sql.to_ascii_lowercase();
    lower.contains("retention_policy")
        || lower.contains("legal_hold")
        || lower.contains("evidence_tombstone")
}

fn validate_retention_legal_hold(up_sql: &str) -> Result<(), MigrationContractError> {
    let lower = up_sql.to_ascii_lowercase();
    let required_tables = [
        "retention_policy",
        "legal_hold",
        "deletion_request",
        "evidence_tombstone",
    ];
    for table in required_tables {
        if !lower.contains(&format!("create table {table}")) {
            return Err(MigrationContractError::MissingRetentionLegalHold);
        }
    }
    if !lower.contains("create or replace function reject_held_evidence_deletion") {
        return Err(MigrationContractError::MissingRetentionLegalHold);
    }
    if !lower.contains("create or replace function reject_tombstoned_evidence_restore") {
        return Err(MigrationContractError::MissingRetentionLegalHold);
    }
    if !lower.contains("create trigger deletion_request_reject_held_deletion") {
        return Err(MigrationContractError::MissingRetentionLegalHold);
    }
    if !lower.contains("create trigger document_record_reject_tombstone_restore") {
        return Err(MigrationContractError::MissingRetentionLegalHold);
    }
    if !lower.contains("constraint retention_policy_period_positive") {
        return Err(MigrationContractError::MissingRetentionLegalHold);
    }
    if !lower.contains("constraint legal_hold_document_scope_consistent") {
        return Err(MigrationContractError::MissingRetentionLegalHold);
    }
    Ok(())
}

fn validate_table_body(table: &str, body: &str) -> Result<(), MigrationContractError> {
    let lower = body.to_ascii_lowercase();
    if requires_tenant_boundary(table) && !lower.contains("tenant_record_id") {
        return Err(MigrationContractError::MissingTenantBoundary);
    }

    if !has_system_time_column(&lower) {
        return Err(MigrationContractError::MissingTemporalColumns);
    }

    // Registry and immutable audit tables may omit availability/valid windows.
    if is_registry_or_audit_table(table) {
        return Ok(());
    }

    if !has_domain_time_column(&lower) {
        return Err(MigrationContractError::MissingTemporalColumns);
    }
    Ok(())
}

fn validate_tenant_rls_contract(
    up_sql: &str,
    tables: &BTreeSet<String>,
) -> Result<(), MigrationContractError> {
    let lower = up_sql.to_ascii_lowercase();
    if !lower.contains("tepp_app_runtime") {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    }
    if !lower.contains("tepp.current_tenant_record_id") {
        return Err(MigrationContractError::MissingTenantSessionGuc);
    }

    let policies = parse_create_policy_names(up_sql);
    if policies.is_empty() {
        return Err(MigrationContractError::MissingRlsPolicy);
    }
    for policy in &policies {
        if !is_multi_word_snake_case(policy) {
            return Err(MigrationContractError::SingleWordObjectName);
        }
    }

    for table in tables {
        if !table_has_rls_enabled(&lower, table) {
            return Err(MigrationContractError::MissingRlsEnable);
        }
        if !table_has_tenant_policy(&lower, table) {
            return Err(MigrationContractError::MissingRlsPolicy);
        }
    }
    Ok(())
}

fn declares_row_level_security(up_sql: &str) -> bool {
    let lower = up_sql.to_ascii_lowercase();
    let has_enable = lower.contains("enable row level security");
    let has_policy = lower.contains("create policy");
    has_enable | has_policy
}

fn table_has_rls_enabled(lower_sql: &str, table: &str) -> bool {
    let enable = format!("alter table {table} enable row level security");
    let force = format!("alter table {table} force row level security");
    lower_sql.contains(&enable) & lower_sql.contains(&force)
}

fn table_has_tenant_policy(lower_sql: &str, table: &str) -> bool {
    let on_table = format!(" on {table}");
    let mut search_from = 0usize;
    while let Some(rel) = lower_sql[search_from..].find("create policy") {
        let abs = search_from + rel;
        let after_policy = &lower_sql[abs..];
        let window_end = after_policy[13..]
            .find("create policy")
            .map_or(after_policy.len(), |idx| 13 + idx);
        let window = &after_policy[..window_end];
        if window.contains(&on_table) && window.contains("tenant_record_id") {
            return true;
        }
        search_from = abs + "create policy".len();
    }
    false
}

fn requires_tenant_boundary(table: &str) -> bool {
    table != "tenant_record"
}

fn is_registry_or_audit_table(table: &str) -> bool {
    table == "tenant_record" || table == "audit_event"
}

fn has_system_time_column(lower_body: &str) -> bool {
    let has_system_time = lower_body.contains("system_time");
    let has_system_from = lower_body.contains("system_from");
    let has_recorded_system_time = lower_body.contains("recorded_system_time");
    has_system_time | has_system_from | has_recorded_system_time
}

fn has_domain_time_column(lower_body: &str) -> bool {
    let has_available = lower_body.contains("available_time");
    let has_valid_from = lower_body.contains("valid_from");
    has_available | has_valid_from
}

fn parse_create_table_names(sql: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let upper = sql.to_ascii_uppercase();
    let mut search_from = 0usize;
    while let Some(rel) = upper[search_from..].find("CREATE TABLE") {
        let abs = search_from + rel + "CREATE TABLE".len();
        let rest = sql[abs..].trim_start();
        let rest = rest
            .strip_prefix("IF NOT EXISTS")
            .or_else(|| rest.strip_prefix("if not exists"))
            .map_or(rest, str::trim_start);
        let name: String = rest
            .chars()
            .take_while(|ch| {
                let alphanumeric = ch.is_ascii_alphanumeric();
                let underscore = *ch == '_';
                alphanumeric | underscore
            })
            .collect();
        if name.is_empty() {
            search_from = abs;
            continue;
        }
        names.insert(name.to_ascii_lowercase());
        search_from = abs;
    }
    names
}

fn parse_create_policy_names(sql: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let upper = sql.to_ascii_uppercase();
    let mut search_from = 0usize;
    while let Some(rel) = upper[search_from..].find("CREATE POLICY") {
        let abs = search_from + rel + "CREATE POLICY".len();
        let rest = sql[abs..].trim_start();
        let name: String = rest
            .chars()
            .take_while(|ch| {
                let alphanumeric = ch.is_ascii_alphanumeric();
                let underscore = *ch == '_';
                alphanumeric | underscore
            })
            .collect();
        if !name.is_empty() {
            names.insert(name.to_ascii_lowercase());
        }
        search_from = abs;
    }
    names
}

fn table_body<'a>(sql: &'a str, table: &str) -> Option<&'a str> {
    let lower = sql.to_ascii_lowercase();
    let needles = [
        format!("create table if not exists {table}"),
        format!("create table {table}"),
    ];
    let start = needles
        .iter()
        .find_map(|needle| lower.find(needle).map(|idx| (idx, needle.len())))?;
    let after = &sql[start.0 + start.1..];
    let open = after.find('(')?;
    let mut depth = 0i32;
    for (idx, ch) in after[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&after[open..=open + idx]);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{MigrationCatalog, validate_migration_catalog};
    use crate::MigrationContractError;

    #[test]
    fn embedded_catalog_is_non_empty_and_valid() {
        let catalog = MigrationCatalog::from_embedded().expect("embedded");
        validate_migration_catalog(&catalog).expect("valid");
        assert!(catalog.up_sql().contains("CREATE TABLE"));
        assert!(catalog.up_sql().contains("ENABLE ROW LEVEL SECURITY"));
        assert!(catalog.up_sql().contains("CREATE POLICY"));
        assert!(catalog.up_sql().contains("tepp_app_runtime"));
        assert!(catalog.up_sql().contains("tepp.current_tenant_record_id"));
        assert!(catalog.down_sql().contains("DROP TABLE"));
        assert!(catalog.down_sql().contains("DROP POLICY"));
        assert!(catalog.down_sql().contains("DROP ROLE"));
    }

    #[test]
    fn helper_predicates_are_exhaustive() {
        use super::{
            declares_row_level_security, has_domain_time_column, has_system_time_column,
            is_registry_or_audit_table, parse_create_policy_names, requires_tenant_boundary,
            table_has_rls_enabled, table_has_tenant_policy,
        };
        assert!(requires_tenant_boundary("document_record"));
        assert!(!requires_tenant_boundary("tenant_record"));
        assert!(is_registry_or_audit_table("tenant_record"));
        assert!(is_registry_or_audit_table("audit_event"));
        assert!(!is_registry_or_audit_table("document_record"));
        assert!(has_system_time_column("system_time timestamptz"));
        assert!(has_system_time_column("system_from timestamptz"));
        assert!(has_system_time_column("recorded_system_time timestamptz"));
        assert!(!has_system_time_column("available_time timestamptz"));
        assert!(has_domain_time_column("available_time timestamptz"));
        assert!(has_domain_time_column("valid_from timestamptz"));
        assert!(!has_domain_time_column("system_time timestamptz"));
        assert!(declares_row_level_security("ENABLE ROW LEVEL SECURITY"));
        assert!(declares_row_level_security("CREATE POLICY x ON y"));
        assert!(!declares_row_level_security(
            "CREATE TABLE document_record ()"
        ));
        assert!(table_has_rls_enabled(
            "alter table document_record enable row level security; alter table document_record force row level security;",
            "document_record"
        ));
        assert!(!table_has_rls_enabled(
            "alter table document_record enable row level security;",
            "document_record"
        ));
        assert!(table_has_tenant_policy(
            "create policy document_record_tenant_isolation on document_record using (tenant_record_id = 'x'::uuid)",
            "document_record"
        ));
        assert!(!table_has_tenant_policy(
            "create policy other_table_policy on other_table using (tenant_record_id = 'x'::uuid)",
            "document_record"
        ));
        let policies = parse_create_policy_names(
            "CREATE POLICY document_record_tenant_isolation ON document_record FOR ALL USING (true);",
        );
        assert!(policies.contains("document_record_tenant_isolation"));
    }

    #[test]
    fn naming_and_column_contracts_fail_closed() {
        let single_word = MigrationCatalog::from_sql(
            "CREATE TABLE documents (document_id uuid PRIMARY KEY);",
            "DROP TABLE documents;",
        );
        assert_eq!(
            validate_migration_catalog(&single_word),
            Err(MigrationContractError::SingleWordObjectName)
        );
        let no_tenant = MigrationCatalog::from_sql(
            r"
            CREATE TABLE document_record (
                document_record_id uuid PRIMARY KEY,
                available_time timestamptz NOT NULL,
                system_time timestamptz NOT NULL
            );
            ",
            "DROP TABLE document_record;",
        );
        assert_eq!(
            validate_migration_catalog(&no_tenant),
            Err(MigrationContractError::MissingTenantBoundary)
        );
        let no_system = MigrationCatalog::from_sql(
            r"
            CREATE TABLE document_record (
                document_record_id uuid PRIMARY KEY,
                tenant_record_id uuid NOT NULL,
                available_time timestamptz NOT NULL
            );
            ",
            "DROP TABLE document_record;",
        );
        assert_eq!(
            validate_migration_catalog(&no_system),
            Err(MigrationContractError::MissingTemporalColumns)
        );
        let missing_domain_time = MigrationCatalog::from_sql(
            r"
            CREATE TABLE document_record (
                document_record_id uuid PRIMARY KEY,
                tenant_record_id uuid NOT NULL,
                system_time timestamptz NOT NULL
            );
            ",
            "DROP TABLE document_record;",
        );
        assert_eq!(
            validate_migration_catalog(&missing_domain_time),
            Err(MigrationContractError::MissingTemporalColumns)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn rls_contracts_fail_closed_when_declared() {
        let missing_role = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
            ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
            CREATE POLICY tenant_record_tenant_isolation ON tenant_record
                FOR ALL USING (
                    tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
                );
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&missing_role),
            Err(MigrationContractError::MissingAppRuntimeRole)
        );

        let missing_guc = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            CREATE ROLE tepp_app_runtime NOSUPERUSER;
            ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
            ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
            CREATE POLICY tenant_record_tenant_isolation ON tenant_record
                FOR ALL USING (tenant_record_id IS NOT NULL);
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&missing_guc),
            Err(MigrationContractError::MissingTenantSessionGuc)
        );

        let missing_enable = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            CREATE ROLE tepp_app_runtime NOSUPERUSER;
            CREATE POLICY tenant_record_tenant_isolation ON tenant_record
                FOR ALL USING (
                    tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
                );
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&missing_enable),
            Err(MigrationContractError::MissingRlsEnable)
        );

        let single_word_policy = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            CREATE ROLE tepp_app_runtime NOSUPERUSER;
            ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
            ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
            CREATE POLICY isolation ON tenant_record
                FOR ALL USING (
                    tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
                );
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&single_word_policy),
            Err(MigrationContractError::SingleWordObjectName)
        );

        let missing_policy = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            CREATE ROLE tepp_app_runtime NOSUPERUSER;
            -- tepp.current_tenant_record_id referenced for GUC scan; isolation policy omitted
            ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
            ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&missing_policy),
            Err(MigrationContractError::MissingRlsPolicy)
        );

        // Policy exists and is multi-word, but does not mention tenant_record_id.
        let policy_without_tenant_predicate = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            CREATE ROLE tepp_app_runtime NOSUPERUSER;
            -- bind GUC name for scan: tepp.current_tenant_record_id
            ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
            ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
            CREATE POLICY tenant_record_tenant_isolation ON tenant_record
                FOR ALL USING (true);
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&policy_without_tenant_predicate),
            Err(MigrationContractError::MissingRlsPolicy)
        );

        // Second CREATE POLICY window + IF NOT EXISTS / empty policy name edges.
        assert!(!super::table_has_tenant_policy(
            "create policy other_table_isolation on other_table using (tenant_record_id = 1); \
             create policy tenant_record_tenant_isolation on tenant_record using (true);",
            "tenant_record",
        ));
        assert!(super::table_has_tenant_policy(
            "create policy other_table_isolation on other_table using (true); \
             create policy tenant_record_tenant_isolation on tenant_record using (tenant_record_id = 1);",
            "tenant_record",
        ));
        assert!(super::parse_create_policy_names("CREATE POLICY \"weird\" ON t;").is_empty());
        assert!(super::table_body(
            "CREATE TABLE IF NOT EXISTS tenant_record (tenant_record_id uuid PRIMARY KEY, system_time timestamptz NOT NULL);",
            "tenant_record",
        )
        .is_some());
        assert!(
            super::table_body("CREATE TABLE tenant_record NO_PARENS;", "tenant_record").is_none()
        );
    }

    #[test]
    fn append_only_immutability_contract_fails_closed() {
        let missing_function = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            CREATE TRIGGER source_artifact_reject_mutation
                BEFORE UPDATE ON source_artifact
                FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&missing_function),
            Err(MigrationContractError::MissingAppendOnlyTrigger)
        );

        let missing_trigger = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            CREATE OR REPLACE FUNCTION reject_append_only_mutation()
            RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NEW; END $$;
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&missing_trigger),
            Err(MigrationContractError::MissingAppendOnlyTrigger)
        );

        // All triggers present; REVOKE omitted only for model_artifact so the
        // last revoke branch returns MissingAppendOnlyTrigger.
        let missing_revoke = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            CREATE OR REPLACE FUNCTION reject_append_only_mutation()
            RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NEW; END $$;
            CREATE TRIGGER source_artifact_reject_mutation
                BEFORE UPDATE ON source_artifact
                FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();
            REVOKE UPDATE, DELETE ON TABLE source_artifact FROM tepp_app_runtime;
            CREATE TRIGGER audit_event_reject_mutation
                BEFORE UPDATE ON audit_event
                FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();
            REVOKE UPDATE, DELETE ON TABLE audit_event FROM tepp_app_runtime;
            CREATE TRIGGER reproducibility_manifest_reject_mutation
                BEFORE UPDATE ON reproducibility_manifest
                FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();
            REVOKE UPDATE, DELETE ON TABLE reproducibility_manifest FROM tepp_app_runtime;
            CREATE TRIGGER corpus_split_manifest_reject_mutation
                BEFORE UPDATE ON corpus_split_manifest
                FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();
            REVOKE UPDATE, DELETE ON TABLE corpus_split_manifest FROM tepp_app_runtime;
            CREATE TRIGGER model_run_reject_mutation
                BEFORE UPDATE ON model_run
                FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();
            REVOKE UPDATE, DELETE ON TABLE model_run FROM tepp_app_runtime;
            CREATE TRIGGER model_artifact_reject_mutation
                BEFORE UPDATE ON model_artifact
                FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&missing_revoke),
            Err(MigrationContractError::MissingAppendOnlyTrigger)
        );

        assert!(super::declares_append_only_immutability(
            "CREATE TRIGGER source_artifact_reject_mutation"
        ));
        assert!(!super::declares_append_only_immutability("CREATE TABLE x"));
        assert_eq!(
            super::validate_append_only_immutability(
                "CREATE TRIGGER source_artifact_reject_mutation BEFORE UPDATE ON source_artifact \
                 FOR EACH ROW EXECUTE FUNCTION reject_append_only_mutation();"
            ),
            Err(MigrationContractError::MissingAppendOnlyTrigger)
        );
    }

    #[test]
    fn temporal_interval_ordering_contract_fails_closed() {
        assert!(super::declares_temporal_interval_ordering(
            "CONSTRAINT document_record_valid_order CHECK (true)"
        ));
        assert!(!super::declares_temporal_interval_ordering(
            "CREATE TABLE x"
        ));

        assert_eq!(
            super::validate_temporal_interval_ordering(
                "CONSTRAINT document_record_valid_order CHECK (valid_to IS NULL OR valid_from <= valid_to)"
            ),
            Err(MigrationContractError::MissingTemporalIntervalConstraint)
        );

        // Named constraints present; fail each predicate branch independently.
        let names = r"
            CONSTRAINT document_record_valid_order CHECK (true)
            CONSTRAINT document_record_system_order CHECK (true)
            CONSTRAINT document_record_revision_positive CHECK (true)
            CONSTRAINT event_instance_valid_order CHECK (true)
            CONSTRAINT event_instance_system_order CHECK (true)
            CONSTRAINT membership_assignment_valid_order CHECK (true)
        ";
        assert_eq!(
            super::validate_temporal_interval_ordering(names),
            Err(MigrationContractError::MissingTemporalIntervalConstraint)
        );
        let missing_system_order = format!(
            "{names}\nCHECK (valid_to IS NULL OR valid_from <= valid_to)\n\
             CHECK (revision_number > 0)"
        );
        assert_eq!(
            super::validate_temporal_interval_ordering(&missing_system_order),
            Err(MigrationContractError::MissingTemporalIntervalConstraint)
        );
        let missing_revision = format!(
            "{names}\nCHECK (valid_to IS NULL OR valid_from <= valid_to)\n\
             CHECK (system_to IS NULL OR system_from <= system_to)"
        );
        assert_eq!(
            super::validate_temporal_interval_ordering(&missing_revision),
            Err(MigrationContractError::MissingTemporalIntervalConstraint)
        );

        // Predicates present but last named constraint missing.
        let missing_membership = r"
            CONSTRAINT document_record_valid_order CHECK (valid_to IS NULL OR valid_from <= valid_to)
            CONSTRAINT document_record_system_order CHECK (system_to IS NULL OR system_from <= system_to)
            CONSTRAINT document_record_revision_positive CHECK (revision_number > 0)
            CONSTRAINT event_instance_valid_order CHECK (valid_to IS NULL OR valid_from <= valid_to)
            CONSTRAINT event_instance_system_order CHECK (system_to IS NULL OR system_from <= system_to)
        ";
        assert_eq!(
            super::validate_temporal_interval_ordering(missing_membership),
            Err(MigrationContractError::MissingTemporalIntervalConstraint)
        );

        let complete = r"
            CONSTRAINT document_record_valid_order CHECK (valid_to IS NULL OR valid_from <= valid_to)
            CONSTRAINT document_record_system_order CHECK (system_to IS NULL OR system_from <= system_to)
            CONSTRAINT document_record_revision_positive CHECK (revision_number > 0)
            CONSTRAINT event_instance_valid_order CHECK (valid_to IS NULL OR valid_from <= valid_to)
            CONSTRAINT event_instance_system_order CHECK (system_to IS NULL OR system_from <= system_to)
            CONSTRAINT membership_assignment_valid_order CHECK (valid_to IS NULL OR valid_from <= valid_to)
        ";
        assert_eq!(super::validate_temporal_interval_ordering(complete), Ok(()));
    }

    #[test]
    fn retention_legal_hold_contract_fails_closed() {
        assert!(super::declares_retention_legal_hold(
            "CREATE TABLE retention_policy (retention_policy_id uuid PRIMARY KEY)"
        ));
        assert!(super::declares_retention_legal_hold(
            "CREATE TABLE legal_hold ()"
        ));
        assert!(super::declares_retention_legal_hold(
            "CREATE TABLE evidence_tombstone ()"
        ));
        assert!(!super::declares_retention_legal_hold("CREATE TABLE x"));

        let missing_table = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            CREATE TABLE retention_policy (
                retention_policy_id uuid PRIMARY KEY,
                tenant_record_id uuid NOT NULL,
                system_time timestamptz NOT NULL,
                available_time timestamptz NOT NULL
            );
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&missing_table),
            Err(MigrationContractError::MissingRetentionLegalHold)
        );

        let tables_only = r"
            CREATE TABLE retention_policy (x int);
            CREATE TABLE legal_hold (x int);
            CREATE TABLE deletion_request (x int);
            CREATE TABLE evidence_tombstone (x int);
        ";
        assert_eq!(
            super::validate_retention_legal_hold(tables_only),
            Err(MigrationContractError::MissingRetentionLegalHold)
        );
        let with_hold_fn =
            format!("{tables_only} CREATE OR REPLACE FUNCTION reject_held_evidence_deletion()");
        assert_eq!(
            super::validate_retention_legal_hold(&with_hold_fn),
            Err(MigrationContractError::MissingRetentionLegalHold)
        );
        let with_restore_fn = format!(
            "{with_hold_fn} CREATE OR REPLACE FUNCTION reject_tombstoned_evidence_restore()"
        );
        assert_eq!(
            super::validate_retention_legal_hold(&with_restore_fn),
            Err(MigrationContractError::MissingRetentionLegalHold)
        );
        let with_hold_trigger =
            format!("{with_restore_fn} CREATE TRIGGER deletion_request_reject_held_deletion");
        assert_eq!(
            super::validate_retention_legal_hold(&with_hold_trigger),
            Err(MigrationContractError::MissingRetentionLegalHold)
        );
        let with_restore_trigger =
            format!("{with_hold_trigger} CREATE TRIGGER document_record_reject_tombstone_restore");
        assert_eq!(
            super::validate_retention_legal_hold(&with_restore_trigger),
            Err(MigrationContractError::MissingRetentionLegalHold)
        );
        let with_period =
            format!("{with_restore_trigger} CONSTRAINT retention_policy_period_positive");
        assert_eq!(
            super::validate_retention_legal_hold(&with_period),
            Err(MigrationContractError::MissingRetentionLegalHold)
        );
        let complete = format!("{with_period} CONSTRAINT legal_hold_document_scope_consistent");
        super::validate_retention_legal_hold(&complete).expect("complete 0007 contract");
    }

    #[test]
    fn empty_and_malformed_sql_fail_closed() {
        let empty = MigrationCatalog::from_sql("   ", "DROP TABLE x;");
        assert_eq!(
            validate_migration_catalog(&empty),
            Err(MigrationContractError::EmptyMigrationSql)
        );
        assert_eq!(
            MigrationCatalog::from_sources("", "DROP TABLE x_y;"),
            Err(MigrationContractError::EmptyMigrationSql)
        );
        assert_eq!(
            MigrationCatalog::from_sources(
                "CREATE TABLE tenant_record (tenant_record_id uuid PRIMARY KEY, system_time timestamptz NOT NULL);",
                "",
            ),
            Err(MigrationContractError::EmptyMigrationSql)
        );
        let empty_down = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            ",
            "   ",
        );
        assert_eq!(
            validate_migration_catalog(&empty_down),
            Err(MigrationContractError::EmptyMigrationSql)
        );
        let no_tables = MigrationCatalog::from_sql(
            "-- comment only without table definitions",
            "DROP TABLE IF EXISTS none_present;",
        );
        assert_eq!(
            validate_migration_catalog(&no_tables),
            Err(MigrationContractError::EmptyMigrationSql)
        );
        let if_not_exists = MigrationCatalog::from_sql(
            r"
            CREATE TABLE IF NOT EXISTS tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            ",
            "DROP TABLE tenant_record;",
        );
        validate_migration_catalog(&if_not_exists).expect("if not exists parse");
        let unclosed = MigrationCatalog::from_sql(
            "CREATE TABLE broken_table (tenant_record_id uuid, system_time timestamptz",
            "DROP TABLE broken_table;",
        );
        assert_eq!(
            validate_migration_catalog(&unclosed),
            Err(MigrationContractError::EmptyMigrationSql)
        );
        let nested = MigrationCatalog::from_sql(
            r"
            CREATE TABLE document_record (
                document_record_id uuid PRIMARY KEY,
                tenant_record_id uuid NOT NULL,
                system_time timestamptz NOT NULL,
                available_time timestamptz NOT NULL,
                CONSTRAINT document_record_positive CHECK (revision_number > 0)
            );
            ",
            "DROP TABLE document_record;",
        );
        validate_migration_catalog(&nested).expect("nested parentheses");
        let trailing = MigrationCatalog::from_sql("CREATE TABLE ", "DROP TABLE none_present;");
        assert_eq!(
            validate_migration_catalog(&trailing),
            Err(MigrationContractError::EmptyMigrationSql)
        );
    }
}
