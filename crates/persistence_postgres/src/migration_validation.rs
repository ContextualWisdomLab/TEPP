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

/// Return whether one normalized statement calls PostgreSQL's unqualified `set_config`
/// with the direct `session_replication_role = replica` contract atoms.
///
/// Whitespace is erased only after the shared lexical pass has made strings and
/// comments safe to inspect. The function-name boundary excludes identifier
/// prefixes and schema-qualified lookalikes such as `audit_support.set_config`.
/// More dynamic configuration expressions remain outside this bounded matcher
/// and must be owned by a future execution-context aggregate rather than guessed.
fn statement_calls_replica_set_config(statement: &str) -> bool {
    const CALL: &str = "set_config('session_replication_role','replica',";
    let compact = statement
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    let mut search_from = 0usize;

    while let Some(relative) = compact[search_from..].find(CALL) {
        let start = search_from + relative;
        let previous = compact[..start].chars().next_back();
        let is_identifier_continuation = previous.is_some_and(|ch| {
            ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || !ch.is_ascii()
        });
        if !is_identifier_continuation && previous != Some('.') {
            return true;
        }
        search_from = start + CALL.len();
    }
    false
}

/// Detect committed PostgreSQL replica execution mode that suppresses ordinary triggers.
///
/// The input has already crossed the shared lexical authority and committed-state
/// projection, so comments, opaque bodies, and rolled-back local settings cannot
/// manufacture this state. PostgreSQL permits optional `LOCAL`/`SESSION`, `TO` or
/// `=`, a quoted enum value, and the equivalent `set_config` function. This
/// bounded fold recognizes direct forms without reparsing raw SQL. Entering
/// `replica` is treated as unsafe runtime state because ordinary TEPP append-only
/// and retention triggers do not fire in that mode.
fn committed_replica_trigger_execution_mode(sql: &str) -> bool {
    sql.split(';').any(|statement| {
        if statement_calls_replica_set_config(statement) {
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
/// runtime role. A committed `session_replication_role = replica`, whether via
/// `SET` or the direct unqualified `set_config` equivalent, also fails the
/// runtime-role contract because it suppresses ordinary enforcement triggers
/// even when their durable catalog definitions remain enabled.
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
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; BEGIN; SELECT set_config('session_replication_role', 'replica', true); COMMIT;",
        ] {
            assert_eq!(declares_created_role(sql, "tepp_app_runtime"), Some(false));
        }
    }

    #[test]
    fn rolled_back_replica_mode_and_safe_modes_preserve_runtime_role_safety() {
        for sql in [
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; BEGIN; SET LOCAL session_replication_role = replica; ROLLBACK;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; BEGIN; SELECT set_config('session_replication_role', 'replica', true); ROLLBACK;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SET session_replication_role = origin;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SET SESSION session_replication_role TO local;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SELECT set_config('session_replication_role', 'origin', false);",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; SELECT audit_support.set_config('session_replication_role', 'replica', false);",
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
