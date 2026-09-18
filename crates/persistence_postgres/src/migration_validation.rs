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

/// Return whether one normalized statement can set `session_replication_role`
/// to a value that is not statically proven safe for ordinary triggers.
///
/// The shared lexical pass already made comments, quoted marker text, and dollar
/// bodies opaque. Function identity is resolved before whitespace compaction so
/// identifier prefixes and unrelated schemas cannot impersonate PostgreSQL's
/// builtin. Positional, named (`=>` / `:=`), and mixed notation are folded into
/// the canonical `setting_name` / `new_value` slots. A direct quoted setting name
/// can prove an unrelated target only when it is one lexical atom; compacted
/// adjacent string constants retain an interior quote and therefore fail closed.
/// A dynamic setting-name expression cannot prove an unrelated target either.
/// For direct `session_replication_role`, only direct `origin` and `local` values
/// are proven safe; other or dynamic values fail closed because `set_config` is
/// PostgreSQL's function equivalent of `SET`.
fn statement_calls_unsafe_set_config(statement: &str) -> bool {
    const FUNCTION_NAME: &str = "set_config";
    const CALL_PREFIX: &str = "set_config(";
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

            if let Some(arguments_and_rest) = compact_call.strip_prefix(CALL_PREFIX) {
                let bytes = arguments_and_rest.as_bytes();
                let mut arguments = Vec::new();
                let mut argument_start = 0usize;
                let mut parenthesis_depth = 0usize;
                let mut bracket_depth = 0usize;
                let mut in_single_quote = false;
                let mut call_closed = false;
                let mut index = 0usize;

                while index < bytes.len() {
                    match bytes[index] {
                        b'\'' => {
                            if in_single_quote
                                && bytes.get(index + 1).is_some_and(|next| *next == b'\'')
                            {
                                index += 2;
                                continue;
                            }
                            in_single_quote = !in_single_quote;
                        }
                        b'(' if !in_single_quote => {
                            parenthesis_depth = parenthesis_depth.saturating_add(1);
                        }
                        b')' if !in_single_quote && parenthesis_depth > 0 => {
                            parenthesis_depth -= 1;
                        }
                        b'[' if !in_single_quote => {
                            bracket_depth = bracket_depth.saturating_add(1);
                        }
                        b']' if !in_single_quote && bracket_depth > 0 => {
                            bracket_depth -= 1;
                        }
                        b',' if !in_single_quote
                            && parenthesis_depth == 0
                            && bracket_depth == 0 =>
                        {
                            arguments.push(&arguments_and_rest[argument_start..index]);
                            argument_start = index + 1;
                        }
                        b')' if !in_single_quote
                            && parenthesis_depth == 0
                            && bracket_depth == 0 =>
                        {
                            arguments.push(&arguments_and_rest[argument_start..index]);
                            call_closed = true;
                            break;
                        }
                        _ => {}
                    }
                    index += 1;
                }

                if call_closed {
                    let mut positional_index = 0usize;
                    let mut setting_name = None;
                    let mut new_value = None;

                    for argument in arguments {
                        let argument = argument.trim();
                        if let Some(value) = argument
                            .strip_prefix("setting_name=>")
                            .or_else(|| argument.strip_prefix("setting_name:="))
                        {
                            setting_name = Some(value);
                            continue;
                        }
                        if let Some(value) = argument
                            .strip_prefix("new_value=>")
                            .or_else(|| argument.strip_prefix("new_value:="))
                        {
                            new_value = Some(value);
                            continue;
                        }
                        if argument.contains("=>") || argument.contains(":=") {
                            continue;
                        }

                        match positional_index {
                            0 => setting_name = Some(argument),
                            1 => new_value = Some(argument),
                            _ => {}
                        }
                        positional_index += 1;
                    }

                    match setting_name {
                        Some("'session_replication_role'") => {
                            let safe = matches!(new_value, Some("'origin'") | Some("'local'"));
                            if !safe {
                                return true;
                            }
                        }
                        Some(name)
                            if name.len() >= 2
                                && name.starts_with('\'')
                                && name.ends_with('\'')
                                && !name[1..name.len() - 1].contains('\'') => {}
                        Some(_) | None => return true,
                    }
                }
            }
        }
        search_from = start + FUNCTION_NAME.len();
    }
    false
}

/// Return whether an exact `UPDATE` command token starts at one byte offset.
///
/// The shared lexical pass has already masked comments, strings, dollar bodies,
/// and unsafe quoted identifiers. This boundary therefore only prevents a
/// substring such as `my_update` or `updates` from becoming a DML candidate.
fn is_update_command_token(statement: &str, start: usize) -> bool {
    let before = statement[..start].chars().next_back();
    let after_start = start + "update".len();
    let after = statement[after_start..].chars().next();
    before.is_none_or(|ch| !is_function_identifier_continuation(ch))
        && after.is_none_or(|ch| !is_function_identifier_continuation(ch))
}

/// Find a keyword token outside nested parenthesized or bracketed expressions.
///
/// This helper operates only after lexical normalization and punctuation
/// delimiting, so quoted/comment/dollar-body text cannot contribute keyword
/// tokens. It is used to distinguish the UPDATE target `WHERE` from a `WHERE`
/// inside a scalar subquery or array expression in the assignment value.
fn top_level_keyword_index(tokens: &[&str], start: usize, keyword: &str) -> Option<usize> {
    let mut parenthesis_depth = 0usize;
    let mut bracket_depth = 0usize;

    for (index, token) in tokens.iter().enumerate().skip(start) {
        match *token {
            "(" => parenthesis_depth = parenthesis_depth.saturating_add(1),
            ")" => {
                if parenthesis_depth == 0 {
                    return None;
                }
                parenthesis_depth -= 1;
            }
            "[" => bracket_depth = bracket_depth.saturating_add(1),
            "]" => {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            }
            _ if parenthesis_depth == 0
                && bracket_depth == 0
                && token.eq_ignore_ascii_case(keyword) =>
            {
                return Some(index);
            }
            _ => {}
        }
    }
    None
}

/// Detect one unsafe `pg_settings` update from an exact normalized `UPDATE` token.
///
/// PostgreSQL documents `UPDATE pg_settings SET setting = ...` as equivalent to
/// `SET`. The input has already crossed the shared lexical authority, so this
/// bounded parser resolves only the canonical unqualified or `pg_catalog`
/// relation identity plus PostgreSQL's optional `ONLY`, `*`, and target alias
/// forms. Direct `origin` and `local` assignment atoms are always safe for
/// ordinary triggers. For any other value, only a complete direct equality to
/// one unrelated quoted setting name proves that `session_replication_role` is
/// excluded; protected equality is recognized in either operand order, and any
/// unsupported predicate shape fails closed instead of being treated as unrelated.
fn update_targets_unsafe_replication_role_via_pg_settings(update_statement: &str) -> bool {
    let delimited = update_statement
        .replace('.', " . ")
        .replace('=', " = ")
        .replace('(', " ( ")
        .replace(')', " ) ")
        .replace('[', " [ ")
        .replace(']', " ] ");
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
        .is_some_and(|token| token.eq_ignore_ascii_case("ONLY"))
    {
        index += 1;
    }
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

    if tokens.get(index) == Some(&"*") {
        index += 1;
    }
    let alias = if tokens
        .get(index)
        .is_some_and(|token| token.eq_ignore_ascii_case("AS"))
    {
        let alias = tokens.get(index + 1).copied();
        index += 2;
        alias
    } else if tokens
        .get(index)
        .is_some_and(|token| !token.eq_ignore_ascii_case("SET"))
    {
        let alias = tokens.get(index).copied();
        index += 1;
        alias
    } else {
        None
    };

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
    let where_index = top_level_keyword_index(&tokens, value_start, "WHERE");
    let value_end = where_index
        .or_else(|| top_level_keyword_index(&tokens, value_start, "RETURNING"))
        .unwrap_or(tokens.len());
    let value_tokens = &tokens[value_start..value_end];
    let value_is_safe = value_tokens.len() == 1
        && (value_tokens[0]
            .trim_matches('\'')
            .eq_ignore_ascii_case("origin")
            || value_tokens[0]
                .trim_matches('\'')
                .eq_ignore_ascii_case("local"));
    if value_is_safe {
        return false;
    }

    let Some(where_index) = where_index else {
        return true;
    };
    let predicate_end = top_level_keyword_index(&tokens, where_index + 1, "RETURNING")
        .unwrap_or(tokens.len());

    let name_operand_end = |start: usize| -> Option<usize> {
        if tokens.get(start + 1) == Some(&".") {
            let qualifier = *tokens.get(start)?;
            let qualifier_matches = alias
                .is_some_and(|expected| qualifier.eq_ignore_ascii_case(expected))
                || alias.is_none() && qualifier.eq_ignore_ascii_case("pg_settings");
            if !qualifier_matches
                || !tokens
                    .get(start + 2)
                    .is_some_and(|token| token.eq_ignore_ascii_case("name"))
            {
                return None;
            }
            Some(start + 3)
        } else if tokens
            .get(start)
            .is_some_and(|token| token.eq_ignore_ascii_case("name"))
        {
            Some(start + 1)
        } else {
            None
        }
    };

    let quoted_setting_name = |token: &str| -> Option<&str> {
        if token.len() >= 2
            && token.starts_with('\'')
            && token.ends_with('\'')
            && !token[1..token.len() - 1].contains('\'')
        {
            Some(&token[1..token.len() - 1])
        } else {
            None
        }
    };

    let predicate_start = where_index + 1;
    if let Some(after_name) = name_operand_end(predicate_start) {
        if tokens.get(after_name) == Some(&"=")
            && after_name + 2 == predicate_end
            && let Some(setting_name) = tokens
                .get(after_name + 1)
                .and_then(|token| quoted_setting_name(token))
        {
            return setting_name.eq_ignore_ascii_case("session_replication_role");
        }
        return true;
    }

    if let Some(setting_name) = tokens
        .get(predicate_start)
        .and_then(|token| quoted_setting_name(token))
        && tokens.get(predicate_start + 1) == Some(&"=")
        && let Some(after_name) = name_operand_end(predicate_start + 2)
        && after_name == predicate_end
    {
        return setting_name.eq_ignore_ascii_case("session_replication_role");
    }

    true
}

/// Detect an unsafe `pg_settings` update anywhere in one normalized statement.
///
/// PostgreSQL permits a leading `WITH` clause before `UPDATE` and permits
/// data-modifying statements inside a CTE. Because lexical opacity is already
/// resolved upstream, scanning exact `UPDATE` command-token candidates lets both
/// executable forms reuse one target/assignment/predicate authority without a
/// second SQL lexer. Identifier-prefixed occurrences are ignored, and a candidate
/// must still parse as canonical `pg_settings` before it can affect this policy.
fn statement_updates_unsafe_replication_role_via_pg_settings(statement: &str) -> bool {
    const UPDATE: &str = "update";
    let lower = statement.to_ascii_lowercase();
    let mut search_from = 0usize;

    while let Some(relative) = lower[search_from..].find(UPDATE) {
        let start = search_from + relative;
        if is_update_command_token(statement, start)
            && update_targets_unsafe_replication_role_via_pg_settings(&statement[start..])
        {
            return true;
        }
        search_from = start + UPDATE.len();
    }
    false
}

/// Detect committed PostgreSQL replica execution mode that suppresses ordinary triggers.
///
/// The input has already crossed the shared lexical authority and committed-state
/// projection, so comments, opaque bodies, and rolled-back local settings cannot
/// manufacture this state. PostgreSQL permits optional `LOCAL`/`SESSION`, `TO` or
/// `=`, a quoted enum value, the equivalent `set_config` function, and equivalent
/// writes through `pg_settings.setting`, including CTE-wrapped UPDATE commands.
/// This bounded fold rejects execution modes that are directly `replica` or cannot
/// be statically proven safe while allowing the ordinary-trigger-safe `origin`
/// and `local` atoms.
fn committed_replica_trigger_execution_mode(sql: &str) -> bool {
    sql.split(';').any(|statement| {
        if statement_calls_unsafe_set_config(statement)
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
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; UPDATE ONLY pg_settings AS p SET setting = 'replica' WHERE p.name = 'session_replication_role';",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; WITH changed_setting AS (UPDATE pg_settings SET setting = 'replica' WHERE name = 'session_replication_role' RETURNING name) SELECT count(*) FROM changed_setting;",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = (SELECT 'replica' WHERE true) WHERE name = 'session_replication_role';",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; WITH changed_setting AS (UPDATE pg_settings SET setting = (SELECT 'replica' WHERE true) WHERE name = 'session_replication_role' RETURNING name) SELECT count(*) FROM changed_setting;",
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
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; UPDATE pg_settings AS p SET setting = 'local' WHERE p.name = 'session_replication_role';",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; UPDATE audit_support.pg_settings SET setting = 'replica' WHERE name = 'session_replication_role';",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; WITH marker AS (SELECT 1) UPDATE pg_settings SET setting = 'origin' WHERE name = 'session_replication_role';",
            "CREATE ROLE tepp_app_runtime NOSUPERUSER NOBYPASSRLS; WITH changed_setting AS (UPDATE pg_settings SET setting = 'local' WHERE name = 'session_replication_role' RETURNING name) SELECT count(*) FROM changed_setting;",
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
