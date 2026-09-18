//! Embedded migration catalog boundary and PostgreSQL table-declaration canonicalization.
//!
//! The lexical facade normalizes comments and quoted regions before this module
//! runs. PostgreSQL permits persistence modifiers between `CREATE` and `TABLE`;
//! those modifiers are validation syntax, not part of the durable table name.
//! This boundary canonicalizes only those bounded declaration prefixes, then
//! delegates all naming, tenant, temporal, RLS, and table-body invariants to the
//! existing migration-core implementation.

#[path = "migration_core_impl.rs"]
mod implementation;
#[path = "migration_rls_table_state.rs"]
mod rls_table_state;
#[path = "migration_runtime_role_executor.rs"]
mod runtime_role_executor;
#[path = "migration_runtime_role_grantor.rs"]
mod runtime_role_grantor;
#[path = "migration_runtime_role_membership.rs"]
mod runtime_role_membership;
#[path = "migration_transaction_projection.rs"]
mod transaction_projection;

use crate::MigrationContractError;
use crate::naming::is_multi_word_snake_case;
pub use implementation::MigrationCatalog;

/// Project already-normalized SQL onto the statements that survive PostgreSQL transaction outcome.
///
/// This is the shared transaction authority for the facade lifecycle/RLS checks
/// and the structural core. Returning `None` means final durable state cannot be
/// proven locally because the input contains savepoint/two-phase ambiguity or an
/// unterminated explicit transaction.
pub(super) fn project_committed_sql(sql: &str) -> Option<String> {
    transaction_projection::project_committed_statements(sql)
}

/// Detect a committed policy-definition mutation not yet owned by the final-policy state model.
///
/// PostgreSQL `ALTER POLICY` can independently replace the role list, `USING`,
/// and `WITH CHECK` clauses while omitted clauses retain prior state. `DROP POLICY`
/// removes the policy definition entirely. Until this bounded validator owns the
/// policy identity/state fold, accepting historical `CREATE POLICY` evidence after
/// either mutation would be fail-open. The input is already lexically normalized
/// and transaction-projected, so statement-first token matching is sufficient and
/// comments/literals cannot manufacture these markers.
fn contains_unsupported_policy_mutation(sql: &str) -> bool {
    sql.split(';').any(|statement| {
        let mut tokens = statement.split_whitespace();
        let Some(verb) = tokens.next() else {
            return false;
        };
        matches!(verb.to_ascii_uppercase().as_str(), "ALTER" | "DROP")
            && tokens
                .next()
                .is_some_and(|token| token.eq_ignore_ascii_case("POLICY"))
    })
}

/// Return the next whitespace-delimited token span beginning at or after `from`.
///
/// Input has already crossed the shared PostgreSQL lexical authority. This
/// cursor exists only to retain the exact byte boundary after an ALTER TABLE
/// target; it does not interpret comments, quoted bodies, or identifiers again.
fn next_sql_token_span(sql: &str, from: usize) -> Option<(usize, usize)> {
    let tail = sql.get(from..)?;
    let mut token_start = None;

    for (offset, ch) in tail.char_indices() {
        if token_start.is_none() {
            if !ch.is_whitespace() {
                token_start = Some(from + offset);
            }
            continue;
        }
        if ch.is_whitespace() {
            if let Some(start) = token_start {
                return Some((start, from + offset));
            }
        }
    }

    token_start.map(|start| (start, sql.len()))
}

/// Compare one normalized token span with an ASCII PostgreSQL keyword.
fn token_span_eq(sql: &str, span: (usize, usize), keyword: &str) -> bool {
    sql.get(span.0..span.1)
        .is_some_and(|token| token.eq_ignore_ascii_case(keyword))
}

/// Return whether one normalized token denotes TEPP's append-only guard routine.
///
/// The shared lexical authority has already handled quoted identifiers. This
/// helper only strips a schema qualifier and attached argument-list punctuation
/// so the final-state guard can recognize both `reject_append_only_mutation()`
/// and schema-qualified spellings without reparsing SQL bodies.
fn token_span_names_append_only_guard_routine(sql: &str, span: (usize, usize)) -> bool {
    let Some(token) = sql.get(span.0..span.1) else {
        return false;
    };
    let name = token.split_once('(').map_or(token, |(name, _)| name);
    name.rsplit('.')
        .next()
        .is_some_and(|part| part.eq_ignore_ascii_case("reject_append_only_mutation"))
}

/// Return whether one committed DROP FUNCTION statement targets the append-only guard routine.
///
/// PostgreSQL permits multiple function targets separated by top-level commas;
/// commas inside function signatures are not target boundaries. Malformed
/// parenthesis structure fails closed because this bounded authority cannot prove
/// that the append-only guard is absent from an ambiguous DROP statement.
fn statement_drops_append_only_guard_routine(statement: &str) -> bool {
    let Some(drop_keyword) = next_sql_token_span(statement, 0) else {
        return false;
    };
    let Some(function_keyword) = next_sql_token_span(statement, drop_keyword.1) else {
        return false;
    };
    if !token_span_eq(statement, drop_keyword, "DROP")
        || !token_span_eq(statement, function_keyword, "FUNCTION")
    {
        return false;
    }

    let mut cursor = function_keyword.1;
    let Some(mut first_target) = next_sql_token_span(statement, cursor) else {
        return true;
    };
    if token_span_eq(statement, first_target, "IF") {
        let Some(exists_keyword) = next_sql_token_span(statement, first_target.1) else {
            return true;
        };
        if !token_span_eq(statement, exists_keyword, "EXISTS") {
            return true;
        }
        cursor = exists_keyword.1;
        first_target = match next_sql_token_span(statement, cursor) {
            Some(target) => target,
            None => return true,
        };
    }
    cursor = first_target.0;

    let Some(targets) = statement.get(cursor..) else {
        return true;
    };
    let mut parenthesis_depth = 0usize;
    let mut target_start = 0usize;
    for (index, ch) in targets.char_indices() {
        match ch {
            '(' => parenthesis_depth += 1,
            ')' => {
                if parenthesis_depth == 0 {
                    return true;
                }
                parenthesis_depth -= 1;
            }
            ',' if parenthesis_depth == 0 => {
                let target = &targets[target_start..index];
                if next_sql_token_span(target, 0)
                    .is_some_and(|span| token_span_names_append_only_guard_routine(target, span))
                {
                    return true;
                }
                target_start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if parenthesis_depth != 0 {
        return true;
    }
    let final_target = &targets[target_start..];
    next_sql_token_span(final_target, 0)
        .is_some_and(|span| token_span_names_append_only_guard_routine(final_target, span))
}

/// Return whether one committed statement defines TEPP's append-only guard routine.
fn statement_defines_append_only_guard_routine(statement: &str) -> bool {
    let Some(create_keyword) = next_sql_token_span(statement, 0) else {
        return false;
    };
    let Some(or_keyword) = next_sql_token_span(statement, create_keyword.1) else {
        return false;
    };
    let Some(replace_keyword) = next_sql_token_span(statement, or_keyword.1) else {
        return false;
    };
    let Some(function_keyword) = next_sql_token_span(statement, replace_keyword.1) else {
        return false;
    };
    let Some(routine) = next_sql_token_span(statement, function_keyword.1) else {
        return false;
    };

    token_span_eq(statement, create_keyword, "CREATE")
        && token_span_eq(statement, or_keyword, "OR")
        && token_span_eq(statement, replace_keyword, "REPLACE")
        && token_span_eq(statement, function_keyword, "FUNCTION")
        && token_span_names_append_only_guard_routine(statement, routine)
}

/// Detect committed mutations that make historical append-only guard evidence stale.
///
/// PostgreSQL `DROP FUNCTION ... CASCADE` can remove dependent triggers, while a
/// later `CREATE OR REPLACE FUNCTION` can replace the routine body without
/// changing the function identity referenced by those triggers. Until TEPP owns
/// final routine-body and dependency state, the first canonical guard definition
/// is accepted but a later replacement or committed removal fails closed. Input
/// is already lexically normalized and transaction-projected, so rolled-back
/// mutations and marker text in comments/literals/dollar bodies are absent here.
fn contains_unsupported_append_only_guard_routine_mutation(sql: &str) -> bool {
    let mut seen_guard_definition = false;
    for statement in sql.split(';') {
        if statement_drops_append_only_guard_routine(statement) {
            return true;
        }
        if statement_defines_append_only_guard_routine(statement) {
            if seen_guard_definition {
                return true;
            }
            seen_guard_definition = true;
        }
    }
    false
}

/// Return whether one ALTER TABLE action begins with destructive `DROP`.
fn alter_table_action_starts_with_drop(action: &str) -> bool {
    next_sql_token_span(action, 0).is_some_and(|span| token_span_eq(action, span, "DROP"))
}

/// Return whether one ALTER TABLE action weakens ordinary trigger enforcement.
///
/// PostgreSQL keeps disabled triggers in catalog state but does not execute them.
/// `ENABLE REPLICA TRIGGER` is likewise insufficient for TEPP's ordinary
/// application path because it fires only when `session_replication_role` is
/// `replica`, not under the normal origin/local modes. Ordinary `ENABLE TRIGGER`
/// and `ENABLE ALWAYS TRIGGER` remain admissible.
fn alter_table_action_weakens_trigger_enforcement(action: &str) -> bool {
    let Some(first) = next_sql_token_span(action, 0) else {
        return false;
    };
    let Some(second) = next_sql_token_span(action, first.1) else {
        return false;
    };

    if token_span_eq(action, first, "DISABLE") && token_span_eq(action, second, "TRIGGER") {
        return true;
    }
    if !token_span_eq(action, first, "ENABLE") || !token_span_eq(action, second, "REPLICA") {
        return false;
    }
    next_sql_token_span(action, second.1)
        .is_some_and(|third| token_span_eq(action, third, "TRIGGER"))
}

/// Detect unsupported final-state actions in PostgreSQL's comma-separated ALTER TABLE action list.
///
/// Action commas are recognized only outside expression parentheses and
/// array/subscript brackets. That keeps commas inside CHECK/function expressions
/// or `ARRAY[...]` from manufacturing action boundaries. Destructive `DROP` and
/// trigger modes that disable normal application-path enforcement fail closed.
/// Unbalanced delimiters also fail closed because malformed structure cannot
/// prove the absence of a later unsupported action.
fn alter_table_actions_include_unsupported_final_state_mutation(actions: &str) -> bool {
    let mut parenthesis_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut action_start = 0usize;

    for (index, ch) in actions.char_indices() {
        match ch {
            '(' => parenthesis_depth += 1,
            ')' => {
                if parenthesis_depth == 0 {
                    return true;
                }
                parenthesis_depth -= 1;
            }
            '[' => bracket_depth += 1,
            ']' => {
                if bracket_depth == 0 {
                    return true;
                }
                bracket_depth -= 1;
            }
            ',' if parenthesis_depth == 0 && bracket_depth == 0 => {
                let action = &actions[action_start..index];
                if alter_table_action_starts_with_drop(action)
                    || alter_table_action_weakens_trigger_enforcement(action)
                {
                    return true;
                }
                action_start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    if parenthesis_depth != 0 || bracket_depth != 0 {
        return true;
    }
    let final_action = &actions[action_start..];
    alter_table_action_starts_with_drop(final_action)
        || alter_table_action_weakens_trigger_enforcement(final_action)
}

/// Return the committed ALTER TABLE action list after its target relation.
///
/// `None` means the statement is not an ALTER TABLE statement. `Err(())` means
/// an ALTER TABLE prefix was present but its target grammar was incomplete, so
/// callers that certify final-state safety must fail closed rather than treating
/// malformed SQL as an unrelated statement.
fn alter_table_action_list(statement: &str) -> Result<Option<&str>, ()> {
    let Some(first) = next_sql_token_span(statement, 0) else {
        return Ok(None);
    };
    let Some(second) = next_sql_token_span(statement, first.1) else {
        return Ok(None);
    };
    if !token_span_eq(statement, first, "ALTER") || !token_span_eq(statement, second, "TABLE") {
        return Ok(None);
    }

    let mut cursor = second.1;
    let Some(mut target) = next_sql_token_span(statement, cursor) else {
        return Err(());
    };
    if token_span_eq(statement, target, "IF") {
        let Some(exists) = next_sql_token_span(statement, target.1) else {
            return Err(());
        };
        if !token_span_eq(statement, exists, "EXISTS") {
            return Err(());
        }
        target = next_sql_token_span(statement, exists.1).ok_or(())?;
    }
    if token_span_eq(statement, target, "ONLY") {
        target = next_sql_token_span(statement, target.1).ok_or(())?;
    }

    cursor = target.1;
    if let Some(star) = next_sql_token_span(statement, cursor) {
        if statement.get(star.0..star.1) == Some("*") {
            cursor = star.1;
        }
    }

    let actions = statement.get(cursor..).ok_or(())?;
    if next_sql_token_span(actions, 0).is_none() {
        return Err(());
    }
    Ok(Some(actions))
}

/// Return whether one ALTER TABLE ADD action introduces a nonconforming column name.
///
/// PostgreSQL permits both `ADD [COLUMN] name ...` and table-constraint forms
/// such as `ADD CONSTRAINT`, `ADD CHECK`, and `ADD FOREIGN KEY`. Only the column
/// form is subject to the durable column-name contract here. The action has
/// already crossed the shared lexical authority, so quoted-identifier handling
/// remains owned by that authority rather than being reparsed locally.
fn alter_table_add_action_has_invalid_column_name(action: &str) -> bool {
    let Some(add) = next_sql_token_span(action, 0) else {
        return false;
    };
    if !token_span_eq(action, add, "ADD") {
        return false;
    }

    let Some(mut candidate) = next_sql_token_span(action, add.1) else {
        return true;
    };
    let explicit_column = token_span_eq(action, candidate, "COLUMN");
    if explicit_column {
        candidate = match next_sql_token_span(action, candidate.1) {
            Some(span) => span,
            None => return true,
        };
    }

    if token_span_eq(action, candidate, "IF") {
        let Some(not) = next_sql_token_span(action, candidate.1) else {
            return true;
        };
        let Some(exists) = next_sql_token_span(action, not.1) else {
            return true;
        };
        if !token_span_eq(action, not, "NOT") || !token_span_eq(action, exists, "EXISTS") {
            return true;
        }
        candidate = match next_sql_token_span(action, exists.1) {
            Some(span) => span,
            None => return true,
        };
    } else if !explicit_column
        && ["CONSTRAINT", "CHECK", "NOT", "UNIQUE", "PRIMARY", "EXCLUDE", "FOREIGN"]
            .iter()
            .any(|keyword| token_span_eq(action, candidate, keyword))
    {
        return false;
    }

    action
        .get(candidate.0..candidate.1)
        .is_none_or(|name| !is_multi_word_snake_case(name))
}

/// Detect invalid ADD-column names in PostgreSQL's top-level ALTER TABLE action list.
///
/// The delimiter rules mirror the destructive-action scan: commas nested in
/// expressions or array/subscript brackets stay inside one action. Structural
/// imbalance is already rejected by the final-state mutation boundary before
/// this naming check runs.
fn alter_table_actions_include_invalid_added_column_name(actions: &str) -> bool {
    let mut parenthesis_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut action_start = 0usize;

    for (index, ch) in actions.char_indices() {
        match ch {
            '(' => parenthesis_depth += 1,
            ')' => parenthesis_depth = parenthesis_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if parenthesis_depth == 0 && bracket_depth == 0 => {
                if alter_table_add_action_has_invalid_column_name(&actions[action_start..index]) {
                    return true;
                }
                action_start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    alter_table_add_action_has_invalid_column_name(&actions[action_start..])
}

/// Detect committed ALTER TABLE additions that bypass the durable column-name contract.
fn contains_invalid_alter_table_added_column_name(sql: &str) -> bool {
    sql.split(';').any(|statement| {
        alter_table_action_list(statement)
            .ok()
            .flatten()
            .is_some_and(alter_table_actions_include_invalid_added_column_name)
    })
}

/// Return whether one committed statement mutates table final state beyond the bounded model.
///
/// `DROP TABLE` removes the durable relation and `DROP TRIGGER` removes durable
/// trigger enforcement. Standalone PostgreSQL `RENAME` forms make historical
/// table/column identities stale. The ordinary `ALTER TABLE ... action [, ...]`
/// form can also contain destructive `DROP` actions such as `DROP COLUMN` or
/// `DROP CONSTRAINT`, plus trigger firing-state changes that can disable ordinary
/// application-path enforcement. Every top-level action is inspected so a safe
/// first action cannot hide a later unsupported mutation.
/// Target parsing is positional, which keeps a table literally named `rename`
/// or `drop` from being confused with an action after lexical normalization.
fn statement_has_unsupported_table_final_state_mutation(statement: &str) -> bool {
    let Some(first) = next_sql_token_span(statement, 0) else {
        return false;
    };
    let Some(second) = next_sql_token_span(statement, first.1) else {
        return false;
    };

    if token_span_eq(statement, first, "DROP")
        && (token_span_eq(statement, second, "TABLE")
            || token_span_eq(statement, second, "TRIGGER"))
    {
        return true;
    }

    let actions = match alter_table_action_list(statement) {
        Ok(Some(actions)) => actions,
        Ok(None) => return false,
        Err(()) => return true,
    };
    let Some(first_action) = next_sql_token_span(actions, 0) else {
        return true;
    };
    if token_span_eq(actions, first_action, "RENAME") {
        return true;
    }
    alter_table_actions_include_unsupported_final_state_mutation(actions)
}

/// Detect committed table removals or identity/destructive mutations not yet owned by final-table state.
///
/// The input has already crossed lexical normalization and transaction outcome,
/// so rolled-back mutations never reach this boundary. Until a first-class
/// table aggregate owns create/drop/rename/recreate plus column/constraint state,
/// accepting historical `CREATE TABLE` evidence after these mutations would be
/// fail-open and is therefore rejected explicitly.
fn contains_unsupported_table_final_state_mutation(sql: &str) -> bool {
    sql.split(';')
        .any(statement_has_unsupported_table_final_state_mutation)
}

/// Validate migration SQL after canonicalizing PostgreSQL table persistence modifiers.
///
/// `UNLOGGED`, `TEMP`/`TEMPORARY`, and PostgreSQL's compatibility
/// `GLOBAL`/`LOCAL TEMP[TEMPORARY]` spellings must traverse the same table-name
/// and table-body contracts as ordinary `CREATE TABLE`. The canonicalized copy
/// exists only for validation; executable migration SQL is never rewritten.
/// Runtime-role membership safety consumes only statements whose effects survive
/// explicit transaction outcome. Executor-relative grantor identity is projected
/// before that transaction filter so in-transaction `SET LOCAL ROLE` and session
/// authorization still identify the grantor that PostgreSQL recorded, while a
/// later rollback cannot donate false membership or revocation evidence. The
/// structural validator and facade-level RLS witnesses receive the same committed
/// final-state projection, so rolled-back DDL or policy composition cannot
/// satisfy naming, tenant, temporal, RLS, or governance contracts. RLS table
/// enablement is additionally folded in statement order so a committed trailing
/// `DISABLE` or `NO FORCE` cannot reuse stale positive evidence from earlier SQL.
/// Committed `ALTER POLICY` and `DROP POLICY` are temporarily rejected until
/// policy identity, clause replacement, and removal have their own final-state
/// authority. Committed `DROP TABLE` / `DROP TRIGGER`, standalone table/column
/// rename forms, destructive ALTER TABLE DROP actions, trigger modes that disable
/// normal application-path enforcement, append-only guard-routine removal, and
/// a second committed `CREATE OR REPLACE FUNCTION reject_append_only_mutation`
/// are likewise rejected until table, trigger, routine, column, constraint,
/// removal/recreation, dependent-object, and routine-body effects are represented
/// by first-class final-state aggregates. Committed ADD-column actions remain
/// supported only when each introduced durable column satisfies the same
/// multi-word `snake_case` authority as CREATE TABLE columns. Mutations removed
/// by the transaction projection never reach either bounded boundary.
///
/// # Errors
///
/// Returns the same naming, tenant, temporal, RLS, or structural contract
/// errors as the underlying migration validator, plus `MissingAppRuntimeRole`
/// when the application runtime has an unsafe or unprovable PostgreSQL
/// membership path or transaction outcome.
pub fn validate_migration_catalog(
    catalog: &MigrationCatalog,
) -> Result<(), MigrationContractError> {
    let Some(grantor_sql) =
        runtime_role_executor::project_executor_relative_grantors(catalog.up_sql())
    else {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    };
    let Some(committed_up) = project_committed_sql(catalog.up_sql()) else {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    };
    let Some(committed_down) = project_committed_sql(catalog.down_sql()) else {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    };
    let Some(committed_grantor_sql) = project_committed_sql(&grantor_sql) else {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    };

    if contains_unsupported_append_only_guard_routine_mutation(&committed_up)
        || contains_unsupported_table_final_state_mutation(&committed_up)
    {
        return Err(MigrationContractError::UnsupportedTableFinalStateMutation);
    }
    if contains_invalid_alter_table_added_column_name(&committed_up) {
        return Err(MigrationContractError::SingleWordObjectName);
    }
    if contains_unsupported_policy_mutation(&committed_up) {
        return Err(MigrationContractError::MissingRlsPolicy);
    }
    let committed_requires_runtime_role =
        super::validation::declares_row_level_security(&committed_up);
    if committed_requires_runtime_role
        && !rls_table_state::final_rls_table_states_are_safe(&committed_up)
    {
        return Err(MigrationContractError::MissingRlsEnable);
    }
    if committed_requires_runtime_role && !super::declares_tenant_session_guc(&committed_up) {
        return Err(MigrationContractError::MissingTenantSessionGuc);
    }
    if committed_requires_runtime_role && !super::tenant_policies_bind_session_guc(&committed_up) {
        return Err(MigrationContractError::MissingRlsPolicy);
    }

    if !runtime_role_membership::runtime_membership_is_rls_safe(&committed_up)
        || !runtime_role_grantor::runtime_membership_grantors_are_rls_safe(
            &committed_grantor_sql,
        )
    {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    }

    let canonical_up = canonicalize_table_persistence_modifiers(&committed_up);
    let canonical_catalog = MigrationCatalog::from_sql(&canonical_up, &committed_down);
    implementation::validate_migration_catalog(&canonical_catalog)
}

/// Return whether `ch` can continue a PostgreSQL unquoted identifier.
///
/// This mirrors the migration-core token boundary so `CREATE` embedded in an
/// identifier cannot start a synthetic declaration while scanning modifiers.
fn is_postgresql_identifier_continuation(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || !ch.is_ascii()
}

/// Match one ASCII keyword at `start` with PostgreSQL identifier boundaries.
fn bounded_keyword_end(sql: &str, start: usize, keyword: &str) -> Option<usize> {
    let end = start.checked_add(keyword.len())?;
    let candidate = sql.get(start..end)?;
    if !candidate.eq_ignore_ascii_case(keyword) {
        return None;
    }
    if sql[..start]
        .chars()
        .next_back()
        .is_some_and(is_postgresql_identifier_continuation)
    {
        return None;
    }
    if sql[end..]
        .chars()
        .next()
        .is_some_and(is_postgresql_identifier_continuation)
    {
        return None;
    }
    Some(end)
}

/// Consume at least one SQL whitespace character before matching `keyword`.
fn keyword_after_required_whitespace(sql: &str, from: usize, keyword: &str) -> Option<usize> {
    let rest = sql.get(from..)?;
    let mut consumed = 0usize;
    for ch in rest.chars() {
        if !ch.is_whitespace() {
            break;
        }
        consumed += ch.len_utf8();
    }
    if consumed == 0 {
        return None;
    }
    bounded_keyword_end(sql, from + consumed, keyword)
}

/// Return the byte immediately after `TABLE` for one supported modifier-bearing declaration.
///
/// PostgreSQL 18 accepts `UNLOGGED`, `TEMP`/`TEMPORARY`, and compatibility
/// `GLOBAL`/`LOCAL TEMP[TEMPORARY]` prefixes. Ordinary `CREATE TABLE` is left
/// untouched so this helper cannot broaden the legacy parser's authority.
fn modifier_table_declaration_end(sql: &str, create_start: usize) -> Option<usize> {
    let create_end = bounded_keyword_end(sql, create_start, "CREATE")?;

    let first_start = {
        let rest = sql.get(create_end..)?;
        let mut consumed = 0usize;
        for ch in rest.chars() {
            if !ch.is_whitespace() {
                break;
            }
            consumed += ch.len_utf8();
        }
        if consumed == 0 {
            return None;
        }
        create_end + consumed
    };

    if let Some(unlogged_end) = bounded_keyword_end(sql, first_start, "UNLOGGED") {
        return keyword_after_required_whitespace(sql, unlogged_end, "TABLE");
    }
    if let Some(temp_end) = bounded_keyword_end(sql, first_start, "TEMP") {
        return keyword_after_required_whitespace(sql, temp_end, "TABLE");
    }
    if let Some(temporary_end) = bounded_keyword_end(sql, first_start, "TEMPORARY") {
        return keyword_after_required_whitespace(sql, temporary_end, "TABLE");
    }

    let scope_end = bounded_keyword_end(sql, first_start, "GLOBAL")
        .or_else(|| bounded_keyword_end(sql, first_start, "LOCAL"))?;
    let temp_end = keyword_after_required_whitespace(sql, scope_end, "TEMP")
        .or_else(|| keyword_after_required_whitespace(sql, scope_end, "TEMPORARY"))?;
    keyword_after_required_whitespace(sql, temp_end, "TABLE")
}

/// Canonicalize supported PostgreSQL table persistence modifiers for validation only.
///
/// The scanner changes only bounded declaration prefixes and copies every other
/// byte verbatim. This preserves declared table spelling, `IF NOT EXISTS`, table
/// bodies, policy evidence, and statement locality while making all supported
/// table forms visible to the one existing `CREATE TABLE` structural authority.
fn canonicalize_table_persistence_modifiers(sql: &str) -> String {
    let bytes = sql.as_bytes();
    let mut output = String::with_capacity(sql.len());
    let mut copied_through = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        if matches!(bytes[index], b'C' | b'c') {
            if let Some(declaration_end) = modifier_table_declaration_end(sql, index) {
                output.push_str(&sql[copied_through..index]);
                output.push_str("CREATE TABLE");
                copied_through = declaration_end;
                index = declaration_end;
                continue;
            }
        }
        index += 1;
    }

    if copied_through == 0 {
        return sql.to_owned();
    }
    output.push_str(&sql[copied_through..]);
    output
}

#[cfg(test)]
mod tests {
    use super::{
        canonicalize_table_persistence_modifiers,
        contains_invalid_alter_table_added_column_name,
        contains_unsupported_append_only_guard_routine_mutation,
        contains_unsupported_policy_mutation, contains_unsupported_table_final_state_mutation,
        project_committed_sql,
    };

    #[test]
    fn table_modifier_canonicalization_preserves_the_declared_name_and_body() {
        let sql = "CREATE\nGLOBAL\tTEMPORARY TABLE IF NOT EXISTS derived_cache (derived_cache_id uuid);";
        assert_eq!(
            canonicalize_table_persistence_modifiers(sql),
            "CREATE TABLE IF NOT EXISTS derived_cache (derived_cache_id uuid);"
        );
    }

    #[test]
    fn ordinary_and_identifier_attached_create_tokens_are_unchanged() {
        for sql in [
            "CREATE TABLE tenant_record (tenant_record_id uuid);",
            "prefixCREATE UNLOGGED TABLE derived_cache (derived_cache_id uuid);",
            "CREATE_UNLOGGED TABLE derived_cache (derived_cache_id uuid);",
        ] {
            assert_eq!(canonicalize_table_persistence_modifiers(sql), sql);
        }
    }

    #[test]
    fn rolled_back_structural_statement_is_absent_from_committed_projection() {
        let projected = project_committed_sql(
            "CREATE TABLE durable_record (durable_record_id uuid); BEGIN; CREATE TABLE rolled_back_record (rolled_back_record_id uuid); ROLLBACK;",
        )
        .expect("simple rollback outcome must project");
        assert!(projected.contains("CREATE TABLE durable_record"));
        assert!(!projected.contains("rolled_back_record"));
    }

    #[test]
    fn append_only_guard_routine_final_state_detection_is_target_bounded() {
        let canonical = "CREATE OR REPLACE FUNCTION reject_append_only_mutation() RETURNS trigger LANGUAGE plpgsql AS  BEGIN RETURN NULL END  ;";
        assert!(!contains_unsupported_append_only_guard_routine_mutation(canonical));
        assert!(contains_unsupported_append_only_guard_routine_mutation(&format!(
            "{canonical} CREATE OR REPLACE FUNCTION reject_append_only_mutation() RETURNS trigger LANGUAGE plpgsql AS  BEGIN RETURN NULL END  ;"
        )));
        assert!(contains_unsupported_append_only_guard_routine_mutation(&format!(
            "{canonical} DROP FUNCTION reject_append_only_mutation() CASCADE ;"
        )));
        assert!(contains_unsupported_append_only_guard_routine_mutation(&format!(
            "{canonical} DROP FUNCTION other_guard(), public.reject_append_only_mutation() CASCADE ;"
        )));
        assert!(!contains_unsupported_append_only_guard_routine_mutation(&format!(
            "{canonical} DROP FUNCTION reject_append_only_mutation_shadow() CASCADE ;"
        )));
    }

    #[test]
    fn table_final_state_mutation_detection_is_statement_token_and_action_bounded() {
        for sql in [
            "DROP\nTABLE tenant_record ;",
            "DROP TABLE IF EXISTS tenant_record CASCADE ;",
            "DROP TRIGGER source_artifact_reject_mutation ON source_artifact ;",
            "DROP TRIGGER IF EXISTS source_artifact_reject_mutation ON source_artifact RESTRICT ;",
            "ALTER TABLE tenant_record RENAME TO tenant_record_archive ;",
            "ALTER TABLE IF EXISTS ONLY tenant_record RENAME COLUMN tenant_record_id TO tenant_key ;",
            "ALTER TABLE tenant_record DROP COLUMN tenant_record_id CASCADE ;",
            "ALTER TABLE tenant_record ADD COLUMN auxiliary_flag boolean, DROP COLUMN tenant_record_id CASCADE ;",
            "ALTER TABLE source_artifact DISABLE TRIGGER source_artifact_reject_mutation ;",
            "ALTER TABLE source_artifact DISABLE TRIGGER USER ;",
            "ALTER TABLE source_artifact ENABLE REPLICA TRIGGER source_artifact_reject_mutation ;",
            "ALTER TABLE source_artifact ADD COLUMN auxiliary_flag boolean, DISABLE TRIGGER source_artifact_reject_mutation ;",
        ] {
            assert!(contains_unsupported_table_final_state_mutation(sql));
        }
        for sql in [
            "SELECT drop_table_marker ; CREATE TABLE tenant_record ( tenant_record_id uuid ) ;",
            "ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY ;",
            "ALTER TABLE rename ENABLE ROW LEVEL SECURITY ;",
            "ALTER TABLE tenant_record ADD COLUMN drop_flag boolean ;",
            "ALTER TABLE tenant_record ADD CONSTRAINT tenant_record_shape CHECK (some_func(a, drop_flag)) ;",
            "ALTER TABLE tenant_record ADD CONSTRAINT tenant_record_array_shape CHECK (some_func(ARRAY[a, drop_flag])) ;",
            "ALTER TABLE source_artifact ENABLE TRIGGER source_artifact_reject_mutation ;",
            "ALTER TABLE source_artifact ENABLE ALWAYS TRIGGER source_artifact_reject_mutation ;",
        ] {
            assert!(!contains_unsupported_table_final_state_mutation(sql));
        }
    }

    #[test]
    fn alter_table_add_column_naming_detection_is_action_bounded() {
        for sql in [
            "ALTER TABLE tenant_record ADD COLUMN flag boolean ;",
            "ALTER TABLE tenant_record ADD COLUMN IF NOT EXISTS flag boolean ;",
            "ALTER TABLE tenant_record ADD flag boolean ;",
            "ALTER TABLE tenant_record ADD COLUMN auxiliary_flag boolean, ADD COLUMN flag boolean ;",
        ] {
            assert!(contains_invalid_alter_table_added_column_name(sql));
        }
        for sql in [
            "ALTER TABLE tenant_record ADD COLUMN auxiliary_flag boolean ;",
            "ALTER TABLE tenant_record ADD CONSTRAINT tenant_record_shape CHECK (some_func(a, b)) ;",
            "ALTER TABLE tenant_record ADD CHECK (tenant_record_id IS NOT NULL) ;",
            "ALTER TABLE tenant_record ADD NOT NULL tenant_record_id ;",
            "ALTER TABLE tenant_record ADD UNIQUE (tenant_record_id) ;",
            "ALTER TABLE tenant_record ADD PRIMARY KEY (tenant_record_id) ;",
            "ALTER TABLE tenant_record ADD FOREIGN KEY (tenant_record_id) REFERENCES tenant_record (tenant_record_id) ;",
        ] {
            assert!(!contains_invalid_alter_table_added_column_name(sql));
        }
    }

    #[test]
    fn policy_mutation_detection_is_statement_and_token_bounded() {
        assert!(contains_unsupported_policy_mutation(
            "ALTER\nPOLICY tenant_isolation ON tenant_record USING ( true ) ;"
        ));
        assert!(contains_unsupported_policy_mutation(
            "DROP\tPOLICY tenant_isolation ON tenant_record ;"
        ));
        assert!(!contains_unsupported_policy_mutation(
            "SELECT alter_policy_marker, drop_policy_marker ; CREATE POLICY tenant_isolation ON tenant_record USING ( true ) ;"
        ));
    }
}
