//! PostgreSQL lexical normalization facade and role-identity guards.
//!
//! The implementation remains the single lexical/structural authority. This
//! facade preserves bounded pieces of PostgreSQL grammar information that would
//! otherwise be lost when quoted role identifiers are projected onto the
//! structural parser's fail-closed representation. `CURRENT_ROLE`,
//! `CURRENT_USER`, and `SESSION_USER` are special unquoted `role_specification`
//! values, while quoted spellings remain ordinary named roles.

use std::fmt::Write as _;

#[path = "migration_validation_impl.rs"]
mod implementation;

const QUOTED_CURRENT_ROLE_GRANTOR: &str = "__tepp_quoted_grantor_current_role@";
const QUOTED_CURRENT_USER_GRANTOR: &str = "__tepp_quoted_grantor_current_user@";
const QUOTED_SESSION_USER_GRANTOR: &str = "__tepp_quoted_grantor_session_user@";
const CASE_DISTINCT_GRANTOR_PREFIX: &str = "__tepp_quoted_grantor_case_";
const INVALID_QUOTED_IDENTIFIER: &str = "INVALID_QUOTED_IDENTIFIER";

/// Normalize migration SQL for bounded structural contract parsing.
///
/// Grantor-only preprojections preserve PostgreSQL role identity that the
/// general fail-closed quoted-identifier projection would otherwise erase.
/// The shared lexical/structural implementation still owns comments, literals,
/// dollar bodies, doubled-quote handling, and every ordinary quoted identifier.
/// After that authority runs, temporary grantor sentinels are retained only in
/// the `GRANTED BY` role-specification slot and are restored to the historical
/// structural representation everywhere else.
pub(super) fn normalize_migration_sql(sql: &str) -> Option<String> {
    if sql.contains(CASE_DISTINCT_GRANTOR_PREFIX) {
        // A caller-supplied token that collides with validation-only provenance
        // state could manufacture or erase grantor evidence. Reject it rather
        // than attempting to distinguish its origin after lexical normalization.
        return None;
    }
    let projected = preserve_quoted_special_grantor_specifications(sql);
    let projected = preserve_case_distinct_quoted_grantor_candidates(&projected);
    let normalized = implementation::normalize_migration_sql(&projected)?;
    let restored = restore_non_grantor_case_distinct_sentinels(&normalized);
    Some(restore_non_grantor_special_role_sentinels(&restored))
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
/// quoted spellings follow the separate grantor-provenance projection below.
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

/// Encode one simple case-distinct quoted role spelling as a provenance token.
///
/// Hex encoding preserves PostgreSQL's exact quoted case while keeping the
/// temporary token whitespace-free. The trailing `@` keeps it outside the
/// grammar of an unquoted PostgreSQL identifier; the public normalization entry
/// point rejects caller-supplied occurrences of the reserved prefix.
fn case_distinct_grantor_sentinel(identifier: &[u8]) -> String {
    let mut sentinel = String::with_capacity(CASE_DISTINCT_GRANTOR_PREFIX.len() + identifier.len() * 2 + 1);
    sentinel.push_str(CASE_DISTINCT_GRANTOR_PREFIX);
    for byte in identifier {
        write!(&mut sentinel, "{byte:02x}").expect("writing to String cannot fail");
    }
    sentinel.push('@');
    sentinel
}

/// Preserve simple mixed/uppercase quoted identifiers through the shared lexer.
///
/// This is not a second SQL lexer. It deliberately recognizes only complete
/// ASCII alphanumeric/underscore quoted spellings whose contents include an
/// uppercase byte. Such spellings are guaranteed to collapse to the general
/// lexer's invalid quoted-name sentinel, so non-grantor occurrences can be
/// restored exactly to that historical fail-closed representation afterwards.
/// Replacements inside comments, strings, or dollar bodies do not donate
/// evidence because the shared lexer still masks those regions. Doubled-quote
/// identifiers are left untouched and remain owned by that lexer.
fn preserve_case_distinct_quoted_grantor_candidates(sql: &str) -> String {
    let bytes = sql.as_bytes();
    let mut projected = String::with_capacity(sql.len());
    let mut copied_through = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] != b'"' || bytes.get(index.wrapping_sub(1)) == Some(&b'"') {
            index += 1;
            continue;
        }

        let content_start = index + 1;
        let mut end = content_start;
        while let Some(byte) = bytes.get(end) {
            if *byte == b'"' {
                break;
            }
            if !byte.is_ascii_alphanumeric() && *byte != b'_' {
                break;
            }
            end += 1;
        }

        let complete = end > content_start
            && bytes.get(end) == Some(&b'"')
            && bytes.get(end + 1) != Some(&b'"');
        let case_distinct = complete
            && bytes[content_start..end]
                .iter()
                .any(u8::is_ascii_uppercase);
        if !case_distinct {
            index += 1;
            continue;
        }

        projected.push_str(&sql[copied_through..index]);
        projected.push_str(&case_distinct_grantor_sentinel(
            &bytes[content_start..end],
        ));
        copied_through = end + 1;
        index = end + 1;
    }

    projected.push_str(&sql[copied_through..]);
    projected
}

/// Keep case-distinct quoted identity only in an explicit `GRANTED BY` slot.
///
/// Every projected candidate would historically have become
/// `INVALID_QUOTED_IDENTIFIER` in an ordinary object/lifecycle position because
/// it contains uppercase quoted content. Retaining the encoded token only after
/// `GRANTED BY` therefore strengthens provenance evidence without weakening the
/// general fail-closed naming boundary.
fn restore_non_grantor_case_distinct_sentinels(normalized_sql: &str) -> String {
    let mut restored = normalized_sql.to_owned();
    let mut search_from = 0usize;

    while let Some(relative) = restored[search_from..].find(CASE_DISTINCT_GRANTOR_PREFIX) {
        let start = search_from + relative;
        let suffix_start = start + CASE_DISTINCT_GRANTOR_PREFIX.len();
        let Some(relative_end) = restored[suffix_start..].find('@') else {
            break;
        };
        let end = suffix_start + relative_end + 1;
        let is_explicit_grantor = restored[..start]
            .trim_end()
            .to_ascii_lowercase()
            .ends_with("granted by");
        if is_explicit_grantor {
            search_from = end;
            continue;
        }
        restored.replace_range(start..end, INVALID_QUOTED_IDENTIFIER);
        search_from = start + INVALID_QUOTED_IDENTIFIER.len();
    }

    restored
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
        CASE_DISTINCT_GRANTOR_PREFIX, INVALID_QUOTED_IDENTIFIER, QUOTED_CURRENT_USER_GRANTOR,
        normalize_migration_sql, preserve_case_distinct_quoted_grantor_candidates,
        preserve_quoted_special_grantor_specifications,
    };

    #[test]
    fn grantor_projection_does_not_split_a_larger_doubled_quote_identifier() {
        let sql = r#"GRANT reporting_owner TO tepp_app_runtime GRANTED BY "prefix""current_user""suffix";"#;
        let projected = preserve_quoted_special_grantor_specifications(sql);
        assert_eq!(projected, sql);
        assert!(!projected.contains(QUOTED_CURRENT_USER_GRANTOR));
    }

    #[test]
    fn case_distinct_projection_does_not_split_doubled_quote_identifiers() {
        let sql = r#"GRANT reporting_owner TO tepp_app_runtime GRANTED BY "Grantor""A";"#;
        let projected = preserve_case_distinct_quoted_grantor_candidates(sql);
        assert_eq!(projected, sql);
        assert!(!projected.contains(CASE_DISTINCT_GRANTOR_PREFIX));
    }

    #[test]
    fn case_distinct_identity_survives_only_in_the_grantor_slot() {
        let normalized = normalize_migration_sql(
            r#"GRANT reporting_owner TO tepp_app_runtime GRANTED BY "GrantorA"; CREATE ROLE "GrantorB";"#,
        )
        .expect("well-formed SQL");
        assert!(normalized.contains(CASE_DISTINCT_GRANTOR_PREFIX));
        assert!(normalized.contains("CREATE TYPE INVALID_QUOTED_IDENTIFIER"));
        assert_eq!(
            normalized.matches(CASE_DISTINCT_GRANTOR_PREFIX).count(),
            1,
            "only the explicit grantor keeps exact quoted identity"
        );
    }

    #[test]
    fn projected_text_inside_literals_remains_owned_by_the_shared_lexer() {
        let normalized = normalize_migration_sql(
            r#"SELECT 'GRANTED BY "GrantorA"'; GRANT reporting_owner TO tepp_app_runtime GRANTED BY "GrantorB";"#,
        )
        .expect("well-formed SQL");
        assert!(!normalized.contains("4772616e746f7241"));
        assert!(normalized.contains("4772616e746f7242"));
        assert!(!normalized.contains(INVALID_QUOTED_IDENTIFIER));
    }
}
