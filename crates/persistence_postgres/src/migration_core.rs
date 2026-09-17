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
        // Lookups below match case-folded SQL; the contract check above used
        // the declared spelling so `Document_Record` cannot pass as lowercase.
        let folded = table.to_ascii_lowercase();
        let body = table_body(catalog.up_sql(), &folded)
            .ok_or(MigrationContractError::EmptyMigrationSql)?;
        validate_table_body(&folded, body)?;
    }

    for object in parse_created_object_names(catalog.up_sql()) {
        if !is_multi_word_snake_case(&object) {
            return Err(MigrationContractError::SingleWordObjectName);
        }
    }
    for constraint in parse_constraint_names(catalog.up_sql()) {
        if !is_multi_word_snake_case(&constraint) {
            return Err(MigrationContractError::SingleWordObjectName);
        }
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
    if has_unbalanced_square_brackets(body) || has_empty_table_element(body) {
        return Err(MigrationContractError::EmptyMigrationSql);
    }
    let columns = parse_column_names(body);
    for column in &columns {
        if !is_multi_word_snake_case(column) {
            return Err(MigrationContractError::SingleWordObjectName);
        }
    }
    if requires_tenant_boundary(table) && !columns.contains("tenant_record_id") {
        return Err(MigrationContractError::MissingTenantBoundary);
    }

    if !has_system_time_column(body) {
        return Err(MigrationContractError::MissingTemporalColumns);
    }

    // Registry and immutable audit tables may omit availability/valid windows.
    if is_registry_or_audit_table(table) {
        return Ok(());
    }

    if !has_domain_time_column(body) {
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
    if !lower.contains("'tepp.current_tenant_record_id'") {
        return Err(MigrationContractError::MissingTenantSessionGuc);
    }

    // Policy names are contract-checked with every other created object in
    // `validate_migration_catalog`; this scan only proves a policy exists.
    let policies = parse_create_policy_names(up_sql);
    if policies.is_empty() {
        return Err(MigrationContractError::MissingRlsPolicy);
    }
    for table in tables {
        let folded = table.to_ascii_lowercase();
        if !table_has_rls_enabled(&lower, &folded) {
            return Err(MigrationContractError::MissingRlsEnable);
        }
        if !table_has_tenant_policy(&lower, &folded) {
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
    let mut search_from = 0usize;
    while let Some(rel) = lower_sql[search_from..].find("create policy") {
        let abs = search_from + rel;
        let after_policy = &lower_sql[abs..];
        let window_end = after_policy[13..]
            .find("create policy")
            .map_or(after_policy.len(), |idx| 13 + idx);
        let window = &after_policy[..window_end];
        if policy_targets_table(window, table)
            && contains_unquoted_identifier(window, "tenant_record_id")
        {
            return true;
        }
        search_from = abs + "create policy".len();
    }
    false
}

fn policy_targets_table(policy_sql: &str, table: &str) -> bool {
    let needle = format!(" on {table}");
    let mut search_from = 0usize;
    while let Some(rel) = policy_sql[search_from..].find(&needle) {
        let end = search_from + rel + needle.len();
        if !identifier_continues_after(policy_sql, end) {
            return true;
        }
        search_from = end;
    }
    false
}

fn contains_unquoted_identifier(sql: &str, identifier: &str) -> bool {
    let mut search_from = 0usize;
    while let Some(rel) = sql[search_from..].find(identifier) {
        let start = search_from + rel;
        let end = start + identifier.len();
        let inside_atomic_literal = sql[..start]
            .bytes()
            .filter(|byte| *byte == b'\'')
            .count()
            % 2
            == 1;
        if !inside_atomic_literal
            && is_word_start(sql, start)
            && !identifier_continues_after(sql, end)
        {
            return true;
        }
        search_from = end;
    }
    false
}

fn requires_tenant_boundary(table: &str) -> bool {
    table != "tenant_record"
}

fn is_registry_or_audit_table(table: &str) -> bool {
    table == "tenant_record" || table == "audit_event"
}

fn has_system_time_column(body: &str) -> bool {
    let columns = parse_column_names(body);
    columns.contains("system_time")
        || columns.contains("system_from")
        || columns.contains("recorded_system_time")
}

fn has_domain_time_column(body: &str) -> bool {
    let columns = parse_column_names(body);
    columns.contains("available_time") || columns.contains("valid_from")
}

/// Object kinds whose `CREATE` statements name a database object.
const CREATE_KEYWORDS: [&str; 10] = [
    "CREATE TABLE",
    "CREATE POLICY",
    "CREATE INDEX",
    "CREATE UNIQUE INDEX",
    "CREATE TRIGGER",
    "CREATE FUNCTION",
    "CREATE OR REPLACE FUNCTION",
    "CREATE TYPE",
    "CREATE VIEW",
    "CREATE SEQUENCE",
];

/// Leading words of a table-level constraint clause, which names no column.
const TABLE_CONSTRAINT_KEYWORDS: [&str; 7] = [
    "constraint",
    "primary",
    "foreign",
    "unique",
    "check",
    "exclude",
    "like",
];

/// Return whether `index` starts a keyword rather than continuing a word.
fn is_word_start(sql: &str, index: usize) -> bool {
    sql[..index]
        .chars()
        .next_back()
        .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
}

/// Return the identifier at the start of `rest`, skipping an existence clause.
fn leading_identifier(rest: &str) -> String {
    let rest = rest.trim_start();
    let lower = rest.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("if not exists")
        .or_else(|| lower.strip_prefix("if exists"))
        .map_or(rest, |stripped| &rest[rest.len() - stripped.len()..])
        .trim_start();
    rest.chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect()
}

/// Return the declared names that follow each occurrence of `keyword`.
///
/// Names keep their declared spelling so the `snake_case` half of the naming
/// contract stays observable; callers fold their own lookup keys.
fn parse_names_after(sql: &str, keyword: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let upper = sql.to_ascii_uppercase();
    let mut search_from = 0usize;
    while let Some(rel) = upper[search_from..].find(keyword) {
        let keyword_start = search_from + rel;
        let abs = keyword_start + keyword.len();
        search_from = abs;
        // Reject `integrity_constraint_violation` and `CREATE TABLEX`: the
        // keyword must stand alone on both sides.
        if !is_word_start(sql, keyword_start) {
            continue;
        }
        if sql[abs..]
            .chars()
            .next()
            .is_some_and(|ch| !ch.is_whitespace())
        {
            continue;
        }
        let name = leading_identifier(&sql[abs..]);
        if !name.is_empty() {
            names.insert(name);
        }
    }
    names
}

fn parse_create_table_names(sql: &str) -> BTreeSet<String> {
    parse_names_after(sql, "CREATE TABLE")
}

fn parse_create_policy_names(sql: &str) -> BTreeSet<String> {
    parse_names_after(sql, "CREATE POLICY")
}

/// Return every object name declared by a `CREATE` statement in `sql`.
fn parse_created_object_names(sql: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for keyword in CREATE_KEYWORDS {
        names.extend(parse_names_after(sql, keyword));
    }
    names
}

/// Return every explicitly named constraint in `sql`.
fn parse_constraint_names(sql: &str) -> BTreeSet<String> {
    parse_names_after(sql, "CONSTRAINT")
}

/// Split one explicit `CREATE TABLE (...)` body at structural element commas.
///
/// Parenthesized type arguments and table-constraint column lists remain within
/// their owning element because their commas occur below the outer table-body
/// depth. Square-bracket array constructors/subscripts are tracked separately,
/// so `ARRAY[1, 2]` cannot manufacture a pseudo table element.
fn split_table_elements(body: &str) -> Vec<&str> {
    let mut depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut start = 0usize;
    let mut segments = Vec::new();
    for (index, ch) in body.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            ',' if depth == 1 && bracket_depth == 0 => {
                segments.push(&body[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    segments.push(&body[start..]);
    segments
}

/// Return whether square-bracket nesting is malformed in an explicit table body.
///
/// The lexical facade has already masked quoted/comment content, so remaining
/// brackets are structural PostgreSQL array constructor, subscript, or type
/// syntax. Rejecting underflow/unclosed nesting prevents a malformed `[` from
/// swallowing later real table-element separators.
fn has_unbalanced_square_brackets(body: &str) -> bool {
    let mut depth = 0i32;
    for ch in body.chars() {
        match ch {
            '[' => depth += 1,
            ']' if depth == 0 => return true,
            ']' => depth -= 1,
            _ => {}
        }
    }
    depth != 0
}

/// Return whether a comma-separated `CREATE TABLE` body contains an empty
/// element rather than a column, table constraint, or `LIKE` clause.
///
/// PostgreSQL permits an entirely empty element list (`CREATE TABLE x ()`), but
/// once a comma is present each side must contain an element. Stripping only the
/// single outer table-body parenthesis from the first/last segment preserves
/// nested type and constraint parentheses while exposing leading, interior, and
/// trailing comma gaps.
fn has_empty_table_element(body: &str) -> bool {
    let segments = split_table_elements(body);
    if segments.len() < 2 {
        return false;
    }
    let last = segments.len() - 1;
    segments.iter().enumerate().any(|(index, segment)| {
        let mut payload = segment.trim();
        if index == 0 {
            payload = payload.strip_prefix('(').unwrap_or(payload).trim_start();
        }
        if index == last {
            payload = payload.strip_suffix(')').unwrap_or(payload).trim_end();
        }
        payload.is_empty()
    })
}

/// Return the column names declared directly in a `CREATE TABLE` body.
///
/// Table-level constraint clauses name no column and are skipped.
fn parse_column_names(body: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for segment in split_table_elements(body) {
        let segment = segment.trim_start_matches(['(', ')']).trim();
        let name = leading_identifier(segment);
        let lowered = name.to_ascii_lowercase();
        if !name.is_empty() && !TABLE_CONSTRAINT_KEYWORDS.contains(&lowered.as_str()) {
            names.insert(name);
        }
    }
    names
}

fn identifier_continues_after(sql: &str, end: usize) -> bool {
    sql[end..]
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn find_table_declaration_end(lower_sql: &str, needle: &str) -> Option<usize> {
    let mut search_from = 0usize;
    while let Some(rel) = lower_sql[search_from..].find(needle) {
        let start = search_from + rel;
        let end = start + needle.len();
        if !identifier_continues_after(lower_sql, end) {
            return Some(end);
        }
        search_from = end;
    }
    None
}

fn starts_with_keyword(sql: &str, keyword: &str) -> bool {
    sql.get(..keyword.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(keyword))
        && sql[keyword.len()..]
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
}

fn table_body<'a>(sql: &'a str, table: &str) -> Option<&'a str> {
    let lower = sql.to_ascii_lowercase();
    let needles = [
        format!("create table if not exists {table}"),
        format!("create table {table}"),
    ];
    let declaration_end = needles
        .iter()
        .find_map(|needle| find_table_declaration_end(&lower, needle))?;
    let after = sql[declaration_end..].trim_start();
    if !after.starts_with('(') {
        // `CREATE TABLE ... AS query` has no explicit column body. Represent it
        // as an empty local body so temporal/tenant contracts fail closed,
        // rather than borrowing a parenthesis from a later SQL statement.
        return starts_with_keyword(after, "AS").then_some("");
    }
    let mut depth = 0i32;
    for (idx, ch) in after.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&after[..=idx]);
                }
            }
            ';' => return None,
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

    /// A valid single-table migration that the added clause is appended to.
    fn conforming_up_sql(extra: &str) -> String {
        format!(
            "CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            {extra}"
        )
    }

    #[test]
    fn every_created_object_kind_must_be_multi_word_snake_case() {
        for clause in [
            "CREATE INDEX idx ON tenant_record (tenant_record_id);",
            "CREATE UNIQUE INDEX Tenant_Idx ON tenant_record (tenant_record_id);",
            "CREATE TRIGGER guard BEFORE UPDATE ON tenant_record;",
            "CREATE FUNCTION reject() RETURNS trigger;",
            "CREATE OR REPLACE FUNCTION Reject_Mutation() RETURNS trigger;",
            "CREATE TYPE kind AS ENUM ('a');",
            "CREATE VIEW records AS SELECT 1;",
            "CREATE SEQUENCE counter;",
            "CREATE POLICY isolation ON tenant_record FOR ALL USING (true);",
        ] {
            let catalog =
                MigrationCatalog::from_sql(&conforming_up_sql(clause), "DROP TABLE tenant_record;");
            assert_eq!(
                validate_migration_catalog(&catalog),
                Err(MigrationContractError::SingleWordObjectName),
                "{clause} was accepted"
            );
        }
    }

    #[test]
    fn column_and_constraint_names_must_be_multi_word_snake_case() {
        let single_word_column = MigrationCatalog::from_sql(
            "CREATE TABLE tenant_record (id uuid PRIMARY KEY, system_time timestamptz NOT NULL);",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&single_word_column),
            Err(MigrationContractError::SingleWordObjectName)
        );

        let mixed_case_column = MigrationCatalog::from_sql(
            "CREATE TABLE tenant_record (Tenant_Id uuid PRIMARY KEY, system_time timestamptz NOT NULL);",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&mixed_case_column),
            Err(MigrationContractError::SingleWordObjectName)
        );

        let named_constraint = MigrationCatalog::from_sql(
            "CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL,
                CONSTRAINT pk UNIQUE (tenant_record_id)
            );",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&named_constraint),
            Err(MigrationContractError::SingleWordObjectName)
        );
    }

    #[test]
    fn parenthesised_types_and_table_constraints_do_not_shift_column_names() {
        let catalog = MigrationCatalog::from_sql(
            "CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                run_cost numeric(12, 4) NOT NULL,
                system_time timestamptz NOT NULL,
                PRIMARY KEY (tenant_record_id),
                CHECK (run_cost > 0)
            );",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(validate_migration_catalog(&catalog), Ok(()));
    }

    #[test]
    fn keywords_inside_identifiers_and_literals_name_no_object() {
        // `integrity_constraint_violation` embeds CONSTRAINT; `CREATE TABLEX`
        // embeds CREATE TABLE. Neither declares an object.
        let catalog = MigrationCatalog::from_sql(
            &conforming_up_sql(
                "RAISE EXCEPTION 'x' USING ERRCODE = 'integrity_constraint_violation';
                 -- CREATE TABLEX nothing;
                 -- 1CONSTRAINT digit_prefixed_word;
                 ALTER TABLE tenant_record DROP CONSTRAINT IF EXISTS tenant_record_unique;",
            ),
            "DROP TABLE tenant_record;",
        );
        assert_eq!(validate_migration_catalog(&catalog), Ok(()));
    }

    #[test]
    fn a_create_keyword_with_no_following_name_declares_nothing() {
        let catalog = MigrationCatalog::from_sql(
            &conforming_up_sql("CREATE VIEW (broken;"),
            "DROP TABLE tenant_record;",
        );
        assert_eq!(validate_migration_catalog(&catalog), Ok(()));
    }

    #[test]
    fn mixed_case_table_names_are_rejected() {
        let catalog = MigrationCatalog::from_sql(
            "CREATE TABLE Document_Record (document_record_id uuid PRIMARY KEY, system_time timestamptz NOT NULL, valid_from timestamptz NOT NULL);",
            "DROP TABLE Document_Record;",
        );
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName)
        );
    }

    #[test]
    fn mixed_case_policy_names_are_rejected() {
        let catalog = MigrationCatalog::from_sql(
            r"
            CREATE TABLE tenant_record (
                tenant_record_id uuid PRIMARY KEY,
                system_time timestamptz NOT NULL
            );
            GRANT SELECT ON tenant_record TO tepp_app_runtime;
            ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY;
            ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;
            CREATE POLICY Tenant_Isolation ON tenant_record
                FOR ALL USING (
                    tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
                );
            ",
            "DROP TABLE tenant_record;",
        );
        assert_eq!(
            validate_migration_catalog(&catalog),
            Err(MigrationContractError::SingleWordObjectName)
        );
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
            SELECT current_setting('tepp.current_tenant_record_id', true);
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
            SELECT current_setting('tepp.current_tenant_record_id', true);
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
