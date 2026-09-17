//! PostgreSQL lexical normalization facade and role-identity guards.
//!
//! The implementation remains the single lexical/structural authority. This
//! facade preserves bounded pieces of PostgreSQL grammar information that would
//! otherwise be lost when identity-equivalent lowercase quoted identifiers are
//! projected onto bare tokens. `CURRENT_ROLE`, `CURRENT_USER`, and
//! `SESSION_USER` are special unquoted `role_specification` values, while the
//! same lowercase spellings in double quotes are ordinary named roles.

#[path = "migration_validation_impl.rs"]
mod implementation;

const QUOTED_CURRENT_ROLE_GRANTOR: &str = "__tepp_quoted_grantor_current_role@";
const QUOTED_CURRENT_USER_GRANTOR: &str = "__tepp_quoted_grantor_current_user@";
const QUOTED_SESSION_USER_GRANTOR: &str = "__tepp_quoted_grantor_session_user@";

/// Normalize migration SQL for bounded structural contract parsing.
///
/// A grantor-only preprojection preserves the identity of lowercase quoted
/// spellings that PostgreSQL otherwise distinguishes from unquoted special role
/// specifications. After the shared lexical/structural pass, those sentinels are
/// restored everywhere except the `GRANTED BY` role-specification slot, so the
/// general object/lifecycle parser sees the same normalized names as before.
pub(super) fn normalize_migration_sql(sql: &str) -> Option<String> {
    let projected = preserve_quoted_special_grantor_specifications(sql);
    let normalized = implementation::normalize_migration_sql(&projected)?;
    Some(restore_non_grantor_special_role_sentinels(&normalized))
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

/// Replace one complete quoted spelling without entering a doubled-quote escape.
///
/// PostgreSQL escapes a double quote inside a quoted identifier by doubling it.
/// The grantor preprojection must therefore ignore a target spelling whose
/// opening or closing quote is adjacent to another quote; the shared lexer will
/// later parse that larger identifier as one token. Comments and literal bodies
/// still remain the shared lexer's responsibility.
fn replace_complete_quoted_spelling(sql: &str, quoted: &str, replacement: &str) -> String {
    let mut projected = sql.to_owned();
    let mut search_from = 0usize;
    while let Some(relative) = projected[search_from..].find(quoted) {
        let start = search_from + relative;
        let end = start + quoted.len();
        let joins_doubled_quote = projected.as_bytes().get(start.wrapping_sub(1)) == Some(&b'"')
            || projected.as_bytes().get(end) == Some(&b'"');
        if joins_doubled_quote {
            search_from = end;
            continue;
        }
        projected.replace_range(start..end, replacement);
        search_from = start + replacement.len();
    }
    projected
}

/// Preserve the three lowercase quoted special-role spellings through the lexer.
///
/// `@` cannot occur in an unquoted PostgreSQL identifier, so these validation
/// sentinels cannot collide with a real unquoted role. Replacements performed
/// inside comments or literal bodies remain non-structural because the shared
/// lexical authority masks those regions afterwards.
fn preserve_quoted_special_grantor_specifications(sql: &str) -> String {
    let projected = replace_complete_quoted_spelling(
        sql,
        "\"current_role\"",
        QUOTED_CURRENT_ROLE_GRANTOR,
    );
    let projected = replace_complete_quoted_spelling(
        &projected,
        "\"current_user\"",
        QUOTED_CURRENT_USER_GRANTOR,
    );
    replace_complete_quoted_spelling(
        &projected,
        "\"session_user\"",
        QUOTED_SESSION_USER_GRANTOR,
    )
}

/// Keep quoted-role sentinels only in the PostgreSQL `GRANTED BY` grammar slot.
///
/// The shared structural normalizer has already collapsed whitespace and masked
/// opaque bodies. A sentinel outside this exact slot represents an ordinary
/// named role and is restored to the lowercase spelling that the general lexer
/// historically produced. Grantor evidence retains the sentinel so it cannot
/// alias the unquoted pseudo-target with the same spelling.
fn restore_non_grantor_special_role_sentinels(normalized_sql: &str) -> String {
    let mut restored = normalized_sql.to_owned();
    for (sentinel, role_name) in [
        (QUOTED_CURRENT_ROLE_GRANTOR, "current_role"),
        (QUOTED_CURRENT_USER_GRANTOR, "current_user"),
        (QUOTED_SESSION_USER_GRANTOR, "session_user"),
    ] {
        let mut search_from = 0usize;
        while let Some(relative) = restored[search_from..].find(sentinel) {
            let start = search_from + relative;
            let is_explicit_grantor = restored[..start]
                .trim_end()
                .to_ascii_lowercase()
                .ends_with("granted by");
            if is_explicit_grantor {
                search_from = start + sentinel.len();
                continue;
            }
            restored.replace_range(start..start + sentinel.len(), role_name);
            search_from = start + role_name.len();
        }
    }
    restored
}

#[cfg(test)]
mod tests {
    use super::{
        QUOTED_CURRENT_USER_GRANTOR, preserve_quoted_special_grantor_specifications,
    };

    #[test]
    fn grantor_projection_does_not_split_a_larger_doubled_quote_identifier() {
        let sql = r#"GRANT reporting_owner TO tepp_app_runtime GRANTED BY "prefix""current_user""suffix";"#;
        let projected = preserve_quoted_special_grantor_specifications(sql);
        assert_eq!(projected, sql);
        assert!(!projected.contains(QUOTED_CURRENT_USER_GRANTOR));
    }
}
