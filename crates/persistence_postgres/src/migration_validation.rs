//! PostgreSQL lexical normalization facade and role-lifecycle identity guard.
//!
//! The implementation remains the single lexical/structural authority. This
//! facade preserves one piece of PostgreSQL grammar information that would
//! otherwise be lost when identity-equivalent lowercase quoted identifiers are
//! projected onto bare tokens: `CURRENT_ROLE`, `CURRENT_USER`, and
//! `SESSION_USER` are special unquoted `role_specification` values for role
//! attribute changes, while the same lowercase spellings in double quotes are
//! ordinary named roles.

#[path = "migration_validation_impl.rs"]
mod implementation;

/// Normalize migration SQL for bounded structural contract parsing.
pub(super) fn normalize_migration_sql(sql: &str) -> Option<String> {
    implementation::normalize_migration_sql(sql)
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
/// only form that the shared lexer would otherwise dequote as identity-equivalent.
/// Replacements inside comments, literals, or dollar bodies remain inert because
/// the shared lexical pass still owns masking of those regions. Mixed/uppercase
/// quoted spellings already fail closed through the existing quoted-identifier
/// sentinel and need no special treatment here.
fn preserve_quoted_special_role_specifications(sql: &str) -> String {
    sql.replace("\"current_role\"", "\"CURRENT_ROLE\"")
        .replace("\"current_user\"", "\"CURRENT_USER\"")
        .replace("\"session_user\"", "\"SESSION_USER\"")
}
