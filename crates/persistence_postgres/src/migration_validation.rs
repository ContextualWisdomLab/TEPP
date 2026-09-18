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

/// Return whether one character can continue PostgreSQL's bounded unquoted function identity.
fn is_function_identifier_continuation(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || !ch.is_ascii()
}

/// Return whether a `set_config` occurrence is PostgreSQL's unqualified or `pg_catalog` builtin.
///
/// Whitespace before the function name is preserved for identity boundaries;
/// arbitrary schema qualification fails closed as unrelated, while explicit
/// `pg_catalog . set_config` resolves to PostgreSQL's canonical implementation.
fn is_builtin_set_config_occurrence(statement: &str, start: usize) -> bool {
    let before = &statement[..start];
    let trimmed = before.trim_end();
    let had_whitespace_boundary = trimmed.len() != before.len();
    let previous = trimmed.chars().next_back();

    if previous != Some('.') {
        return had_whitespace_boundary
            || previous.is_none_or(|ch| !is_function_identifier_continuation(ch));
    }

    let before_dot = trimmed[..trimmed.len() - 1].trim_end();
    let schema_end = before_dot.len();
    let schema_start = before_dot
        .char_indices()
        .rev()
        .find(|(_, ch)| !is_function_identifier_continuation(*ch))
        .map_or(0, |(index, ch)| index + ch.len_utf8());
    let schema = &before_dot[schema_start..schema_end];
    if !schema.eq_ignore_ascii_case("pg_catalog") {
        return false;
    }
    before_dot[..schema_start]
        .chars()
        .next_back()
        .is_none_or(|ch| ch != '.' && !is_function_identifier_continuation(ch))
}

/// Return whether one normalized statement calls PostgreSQL's `set_config`
/// with the direct `session_replication_role = replica` contract atoms.
///
/// Argument whitespace is erased only after the shared lexical pass has made
/// strings and comments safe to inspect. Function identity is resolved before
/// compaction so statement whitespace cannot merge `SELECT` with `set_config`.
/// The bounded matcher accepts the unqualified builtin and explicit
/// `pg_catalog.set_config`, while excluding identifier prefixes and unrelated
/// schemas such as `audit_support.set_config`. More dynamic configuration
/// expressions remain for a future execution-context aggregate.
fn statement_calls_replica_set_config(statement: &str) -> bool {
    const FUNCTION_NAME: &str = "set_config";
    const CALL: &str = "set_config('session_replication_role','replica',";
    let lower = statement.to_ascii_lowercase();
    let mut search_from = 0usize;

    while let Some(relative) = lower[search_from..].find(FUNCTION_NAME) {
        let start = search_from + relative;
        if is_builtin_set_config_occurrence(statement, start) {
            let compact_call = statement[start..]
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .collect::<String>()
                .to_ascii_lowercase();
            if compact_call.starts_with(CALL) {
                return true;
            }
        }
        search_from = start + FUNCTION_NAME.len();
    }
    false
}

/// Detect an unsafe `pg_settings` update of PostgreSQL's trigger execution mode.
///
/// PostgreSQL documents `UPDATE pg_settings SET setting = ...` as equivalent to
/// `SET`. The input has already crossed the shared lexical authority, so this
/// bounded parser only resolves the canonical unqualified or `pg_catalog`
/// relation identity and a direct `WHERE name = 'session_replication_role'`
/// target. Atomic `origin` and `local` values are proven safe for ordinary
/// triggers; `replica` and any non-atomic value fail closed because the validator
/// cannot prove that protected DML did not execute while triggers were suppressed.
fn statement_updates_unsafe_replication_role_via_pg_settings(statement: &str) -> bool {
    let delimited = statement.replace('.', " . ").replace('=', " = ");
    let tokens = delimited.split_whitespace().collect::<Vec<_>>();
    if !tokens
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("UPDATE"))
    {
        return false;
    }

    let mut index = 1usize;
    if tokens
        .get(index)
        .is_some_and(|token| token.eq_ignore_ascii_case("pg_settings"))
    {
        index += 1;
    } else if tokens
        .get(index)
        .is_some_and(|token| token.eq_ignore_ascii_case("pg_catalog"))
        && tokens.get(index + 1) == Some(&".")
        && tokens
            .get(index + 2)
            .is_some_and(|token| token.eq_ignore_ascii_case("pg_settings"))
    {
        index += 3;
    } else {
        return false;
    }

    if !tokens
        .get(index)
        .is_some_and(|token| token.eq_ignore_ascii_case("SET"))
        || !tokens
            .get(index + 1)
            .is_some_and(|token| token.eq_ignore_ascii_case("setting"))
        || tokens.get(index + 2) != Some(&"=")
    {
        return false;
    }
    let value_start = index + 3;
    let Some(where_index) = tokens[value_start..]
        .iter()
        .position(|token| token.eq_ignore_ascii_case("WHERE"))
        .map(|relative| value_start + relative)
    else {
        return false;
    };

    if !tokens
        .get(where_index + 1)
        .is_some_and(|token| token.eq_ignore_ascii_case("name"))
        || tokens.get(where_index + 2) != Some(&"=")
        || !tokens.get(where_index + 3).is_some_and(|value| {
            value
                .trim_matches('\'')
                .eq_ignore_ascii_case("session_replication_role")
        })
    {
        return false;
    }

    let value_tokens = &tokens[value_start..where_index];
    value_tokens.len() != 1
        || !value_tokens[0]
            .trim_matches('\'')
            .eq_ignore_ascii_case("origin")
            && !value_tokens[0]
                .trim_matches('\'')
                .eq_ignore_ascii_case("local")
}

/// Detect committed PostgreSQL replica execution mode that suppresses ordinary triggers.
///
/// The input has already crossed the shared lexical authority and committed-state
/// projection, so comments, opaque bodies, and rolled-back local settings cannot
/// manufacture this state. PostgreSQL permits optional `LOCAL`/`SESSION`, `TO` or
/// `=`, a quoted enum value, the equivalent `set_config` function, and equivalent
/// writes through `pg_settings.setting`. This bounded fold rejects execution modes
/// that are directly `replica` or cannot be statically proven safe while allowing
/// the ordinary-trigger-safe `origin` and `local` atoms.
fn committed_replica_trigger_execution_mode(sql: &str) -> bool {
    sql.split(';').any(|statement| {
        if statement_calls_replica_set_config(statement)
            || statement_updates_unsafe_replication_role_via_pg_settings(statement)
        {
            return true;
        }

        let delimited = statement.replace('=', " = ");
        let tokens = delimited.split_whitespace().collect::<Vec<_>>();
        if !tokens
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("SET"))
        {
            return false;
        }

        let mut index = 1usize;
        if tokens.get(index).is_some_and(|token| {
            token.eq_ignore_ascii_case("LOCAL") || token.eq_ignore_ascii_case("SESSION")
        }) {
            index += 1;
        }
        if !tokens
            .get(index)
            .is_some_and(|token| token.eq_ignore_ascii_case("session_replication_role"))
        {
            return false;
        }
        index += 1;
        if !tokens.get(index).is_some_and(|token| {
            *token == "=" || token.eq_ignore_ascii_case("TO")
        }) {
            return false;
        }
        index += 1;

        tokens.get(index).is_some_and(|value| {
            value
                .trim_matches('\'')
                .eq_ignore_ascii_case("replica")
        })
    })
}

/// Return whether the expected runtime role exists in PostgreSQL's durable final migration state
/// and remains subject to row-level security.
///
/// Lowercase quoted spellings of PostgreSQL's special role specifications are
/// projected to case-distinct quoted spellings only for this lifecycle scan.
/// The existing lexical authority then maps them to its fail-closed quoted-name
/// sentinel, keeping a named role such as `"current_user"` distinct from the
/// unquoted `CURRENT_USER` pseudo-target without changing executable SQL or the
/// general structural-normalization contract. Transaction outcome is applied
/// after that shared lexical projection and before lifecycle folding, so a
/// rolled-back `ALTER ROLE ... NOBYPASSRLS` cannot certify an actually unsafe
/// runtime role. A committed unsafe `session_replication_role` mutation through
/// `SET`, PostgreSQL's `set_config` equivalent, or canonical `pg_settings` update
/// also fails the runtime-role contract because it can suppress ordinary
/// enforcement triggers while durable catalog definitions remain enabled.
pub(super) fn declares_created_role(sql: &str, expected_role: &str) -> Option<bool> {
    let lifecycle_sql = preserve_quoted_special_role_specifications(sql);
    let normalized_lifecycle = implementation::normalize_migration_sql_with_grantor_identity(
        &lifecycle_sql,
    )?;
    let committed_lifecycle = super::core::project_committed_sql(&normalized_lifecycle)?;
    if committed_replica_trigger_execution_mode(&committed_lifecycle) {
        return Some(false);
    }
    implementation::declares_created_role(&committed_lifecycle, expected_role)
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
    fn rolled_back_runtime_role_hardening_does_not_change_final_lifecycle_state() {
        let sql = r#"
            CREATE ROLE tepp_app_runtime BYPASSRLS;
            BEGIN;
            ALTER ROLE tepp_app_runtime NOBYPASSRLS;
            ROLLBACK;
        "#;
        assert_eq!(declares_created_role(sql, "tepp_app_runtime"), Some(false));
    }

    #[test]
    fn committed_replica_mode_invalidates_runtime_role_safety_after_projection() {
        for sql in [
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SET session_replication_role=replica;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SET SESSION session_replication_role TO 'replica';",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; BEGIN; SET LOCAL session_replication_role = replica; COMMIT;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SELECT set_config('session_replication_role', 'replica', false);",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SELECT pg_catalog . set_config('session_replication_role', 'replica', false);",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; BEGIN; SELECT set_config('session_replication_role', 'replica', true); COMMIT;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; UPDATE pg_catalog . pg_settings SET setting = lower('REPLICA') WHERE name = 'session_replication_role';",
        ] {
            assert_eq!(declares_created_role(sql, "tepp_app_runtime"), Some(false));
        }
    }

    #[test]
    fn rolled_back_replica_mode_and_safe_modes_preserve_runtime_role_safety() {
        for sql in [
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; BEGIN; SET LOCAL session_replication_role = replica; ROLLBACK;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; BEGIN; SELECT set_config('session_replication_role', 'replica', true); ROLLBACK;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; BEGIN; UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role'; ROLLBACK;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SET session_replication_role = origin;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SET SESSION session_replication_role TO local;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SELECT set_config('session_replication_role', 'origin', false);",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SELECT audit_support.set_config('session_replication_role', 'replica', false);",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; UPDATE pg_settings SET setting = 'origin' WHERE name = 'session_replication_role';",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; UPDATE audit_support.pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';",
        ] {
            assert_eq!(declares_created_role(sql, "tepp_app_runtime"), Some(true));
        }
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
