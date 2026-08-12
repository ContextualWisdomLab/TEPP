//! Embedded migration catalog and fail-closed SQL contracts.

use crate::MigrationContractError;
use crate::naming::is_multi_word_snake_case;
use std::collections::BTreeSet;

const FOUNDATION_UP: &str = include_str!("../../../migrations/0001_bitemporal_foundation.up.sql");
const FOUNDATION_DOWN: &str =
    include_str!("../../../migrations/0001_bitemporal_foundation.down.sql");

/// Forward and rollback SQL for one migration unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationCatalog {
    up_sql: String,
    down_sql: String,
}

impl MigrationCatalog {
    /// Load the embedded foundation migrations shipped with this crate.
    ///
    /// # Errors
    ///
    /// Returns [`MigrationContractError::EmptyMigrationSql`] when embedded
    /// sources are unexpectedly empty.
    pub fn from_embedded() -> Result<Self, MigrationContractError> {
        Self::from_sources(FOUNDATION_UP, FOUNDATION_DOWN)
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
/// # Errors
///
/// Returns naming, tenant, temporal, or emptiness failures.
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
        assert!(catalog.down_sql().contains("DROP TABLE"));
    }

    #[test]
    fn helper_predicates_are_exhaustive() {
        use super::{
            has_domain_time_column, has_system_time_column, is_registry_or_audit_table,
            requires_tenant_boundary,
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
        // `from_sql` permits empty down SQL so validate sees the down-empty branch.
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
