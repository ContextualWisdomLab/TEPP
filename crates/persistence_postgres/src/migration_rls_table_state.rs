//! Final PostgreSQL row-level-security table-state projection.
//!
//! The caller supplies SQL after lexical normalization and committed-statement
//! projection. This module owns only the order-sensitive `ALTER TABLE` state
//! needed to prevent historical `ENABLE`/`FORCE` statements from certifying a
//! table whose durable state was later changed to `DISABLE` or `NO FORCE`.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RlsTableState {
    enabled: bool,
    forced: bool,
}

/// Require every table touched by an RLS table-state action to finish enabled and forced.
///
/// Structural validation separately proves that every created table in an RLS
/// migration has the required actions. This projection adds the missing temporal
/// property: later committed actions override earlier ones. Unsupported target
/// grammar on a statement that contains an RLS state action fails closed rather
/// than donating ambiguous final-state evidence.
#[must_use]
pub(super) fn final_rls_table_states_are_safe(sql: &str) -> bool {
    let tokenizable = sql.replace(';', " ; ").replace(',', " , ");
    let tokens = tokenizable.split_whitespace().collect::<Vec<_>>();
    let mut states = BTreeMap::<String, RlsTableState>::new();
    let mut index = 0usize;

    while index < tokens.len() {
        let end = statement_end(&tokens, index);
        let statement = &tokens[index..end];
        index = end.saturating_add(1);

        if statement.is_empty() || !is_alter_table(statement) {
            continue;
        }
        let actions = rls_actions(statement);
        if actions.is_empty() {
            continue;
        }
        let Some(table) = direct_table_target(statement) else {
            return false;
        };
        let state = states.entry(table.to_ascii_lowercase()).or_default();
        for action in actions {
            match action {
                RlsTableAction::Enable => state.enabled = true,
                RlsTableAction::Disable => state.enabled = false,
                RlsTableAction::Force => state.forced = true,
                RlsTableAction::NoForce => state.forced = false,
            }
        }
    }

    states
        .values()
        .all(|state| state.enabled && state.forced)
}

/// Find one semicolon-delimited normalized statement boundary.
fn statement_end(tokens: &[&str], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| *token == ";")
        .map_or(tokens.len(), |relative| start + relative)
}

/// Return whether a normalized statement begins with direct `ALTER TABLE` syntax.
fn is_alter_table(statement: &[&str]) -> bool {
    statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("ALTER"))
        && statement
            .get(1)
            .is_some_and(|token| token.eq_ignore_ascii_case("TABLE"))
}

/// Extract the direct unqualified table target owned by the existing structural validator.
///
/// `ONLY`, `IF EXISTS`, qualification, and quoted-identity sentinels remain
/// outside this bounded grammar. If such a statement carries an RLS state action,
/// the caller fails closed rather than guessing which durable relation changed.
fn direct_table_target<'a>(statement: &'a [&'a str]) -> Option<&'a str> {
    let table = *statement.get(2)?;
    if table.eq_ignore_ascii_case("ONLY")
        || table.eq_ignore_ascii_case("IF")
        || table.contains('.')
        || table == ","
        || !table
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || !ch.is_ascii())
    {
        return None;
    }
    Some(table)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RlsTableAction {
    Enable,
    Disable,
    Force,
    NoForce,
}

/// Parse order-sensitive RLS state actions from one normalized `ALTER TABLE` statement.
fn rls_actions(statement: &[&str]) -> Vec<RlsTableAction> {
    let mut actions = Vec::new();
    let mut index = 3usize;
    while index < statement.len() {
        if keyword_sequence(statement, index, &["ENABLE", "ROW", "LEVEL", "SECURITY"]) {
            actions.push(RlsTableAction::Enable);
            index += 4;
            continue;
        }
        if keyword_sequence(statement, index, &["DISABLE", "ROW", "LEVEL", "SECURITY"]) {
            actions.push(RlsTableAction::Disable);
            index += 4;
            continue;
        }
        if keyword_sequence(statement, index, &["NO", "FORCE", "ROW", "LEVEL", "SECURITY"]) {
            actions.push(RlsTableAction::NoForce);
            index += 5;
            continue;
        }
        if keyword_sequence(statement, index, &["FORCE", "ROW", "LEVEL", "SECURITY"]) {
            actions.push(RlsTableAction::Force);
            index += 4;
            continue;
        }
        index += 1;
    }
    actions
}

/// Match one case-insensitive SQL keyword sequence without reinterpreting identifiers.
fn keyword_sequence(statement: &[&str], start: usize, expected: &[&str]) -> bool {
    statement
        .get(start..start.saturating_add(expected.len()))
        .is_some_and(|actual| {
            actual
                .iter()
                .zip(expected)
                .all(|(token, keyword)| token.eq_ignore_ascii_case(keyword))
        })
}

#[cfg(test)]
mod tests {
    use super::final_rls_table_states_are_safe;

    #[test]
    fn trailing_disable_or_no_force_overrides_historical_positive_state() {
        for sql in [
            "ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY; ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY; ALTER TABLE tenant_record DISABLE ROW LEVEL SECURITY;",
            "ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY; ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY; ALTER TABLE tenant_record NO FORCE ROW LEVEL SECURITY;",
        ] {
            assert!(!final_rls_table_states_are_safe(sql));
        }
    }

    #[test]
    fn later_enable_and_force_restore_final_state() {
        assert!(final_rls_table_states_are_safe(
            "ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY; ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY; ALTER TABLE tenant_record DISABLE ROW LEVEL SECURITY; ALTER TABLE tenant_record NO FORCE ROW LEVEL SECURITY; ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY; ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY;"
        ));
    }

    #[test]
    fn sibling_table_state_does_not_overwrite_another_table() {
        assert!(!final_rls_table_states_are_safe(
            "ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY; ALTER TABLE tenant_record FORCE ROW LEVEL SECURITY; ALTER TABLE tenant_record_shadow ENABLE ROW LEVEL SECURITY; ALTER TABLE tenant_record_shadow FORCE ROW LEVEL SECURITY; ALTER TABLE tenant_record DISABLE ROW LEVEL SECURITY;"
        ));
    }
}
