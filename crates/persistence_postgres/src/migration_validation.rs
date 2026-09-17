//! PostgreSQL lexical normalization facade and role-identity guards.
//!
//! The implementation remains the single lexical/structural authority. This
//! facade keeps lifecycle-specific PostgreSQL pseudo-target handling separate
//! from grantor provenance: exact quoted grantor identity is now emitted by the
//! shared lexer itself rather than reconstructed by a second quoted-name parser.

#[path = "migration_validation_impl.rs"]
mod implementation;

/// Normalize migration SQL for bounded structural contract parsing.
///
/// Exact PostgreSQL quoted grantor identity is preserved only in an explicit
/// `GRANTED BY` slot by the shared lexical authority. Every other quoted
/// identifier follows the historical structural projection, so naming and
/// lifecycle fail-closed behavior is unchanged.
pub(super) fn normalize_migration_sql(sql: &str) -> Option<String> {
    implementation::normalize_migration_sql_with_grantor_identity(sql)
}

/// Return whether the expected runtime role exists in the final migration state
/// and remains subject to PostgreSQL row-level security.
///
/// Lowercase quoted spellings of PostgreSQL's special role specifications are
/// projected to case-distinct quoted spellings only for this lifecycle scan.
/// The existing lexical authority then maps them to its fail-closed quoted-name
/// sentinel, keeping a named role such as `"current_user"` distinct from the
/// unquoted `CURRENT_USER` pseudo-target without changing executable SQL or the
/// general structural-normalization contract.
pub(super) fn declares_created_role(sql: &str, expected_role: &str) -> Option<bool> {
    let lifecycle_sql = preserve_quoted_special_role_specifications(sql);
    implementation::declares_created_role(&lifecycle_sql, expected_role)
}

/// Detect whether normalized migration SQL declares an RLS surface.
pub(super) fn declares_row_level_security(normalized_sql: &str) -> bool {
    implementation::declares_row_level_security(normalized_sql)
}

/// Preserve quoted named-role identity against unquoted PostgreSQL pseudo-targets.
///
/// The replacement is deliberately limited to lowercase quoted spellings, the
/// only form that the lifecycle projection would otherwise dequote as
/// identity-equivalent. Replacements inside comments, literals, or dollar bodies
/// remain inert because the shared lexical pass still owns those regions.
fn preserve_quoted_special_role_specifications(sql: &str) -> String {
    sql.replace("\"current_role\"", "\"CURRENT_ROLE\"")
        .replace("\"current_user\"", "\"CURRENT_USER\"")
        .replace("\"session_user\"", "\"SESSION_USER\"")
}

#[cfg(test)]
mod tests {
    use super::{declares_created_role, normalize_migration_sql};

    #[test]
    fn quoted_special_role_name_stays_distinct_from_unquoted_pseudo_target_in_lifecycle() {
        let sql = r#"
            CREATE ROLE "current_user" BYPASSRLS;
            ALTER ROLE CURRENT_USER NOBYPASSRLS;
            ALTER ROLE "current_user" RENAME TO tepp_app_runtime;
        "#;
        assert_eq!(declares_created_role(sql, "tepp_app_runtime"), Some(false));
    }

    #[test]
    fn grantor_identity_comes_from_the_shared_lexical_authority() {
        let normalized = normalize_migration_sql(
            r#"GRANT reporting_owner TO tepp_app_runtime GRANTED BY "Grantor""A"; CREATE ROLE "Grantor""B";"#,
        )
        .expect("well-formed quoted identities");
        assert!(normalized.contains("GRANTED BY __tepp_quoted_grantor_identity_"));
        assert!(normalized.contains("CREATE TYPE INVALID_QUOTED_IDENTIFIER"));
    }
}
