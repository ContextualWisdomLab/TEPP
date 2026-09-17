//! Execution-state projection for PostgreSQL pseudo-grantor role specifications.
//!
//! The shared lexical authority has already removed comments/literals from
//! structural consideration before this boundary runs. This module therefore
//! does not lex SQL again: it tracks only normalized statement tokens that
//! change the effective role and rewrites executor-relative `GRANTED BY`
//! pseudo-targets to validation-only provenance identities.

const EXECUTOR_GRANTOR_PREFIX: &str = "__tepp_executor_grantor_";
const INVALID_QUOTED_IDENTIFIER: &str = "INVALID_QUOTED_IDENTIFIER";

/// Effective PostgreSQL role identity visible to `CURRENT_USER` / `CURRENT_ROLE`.
///
/// Initial and session identities remain separate because PostgreSQL can start
/// a connection with a role setting distinct from `SESSION_USER`. Named roles
/// are exact normalized identities. Unknown targets are scoped to the `SET ROLE`
/// statement that introduced them so later statements cannot accidentally
/// alias them to a different executor role.
#[derive(Clone, Debug, Eq, PartialEq)]
enum EffectiveRoleProjection {
    InitialCurrentUser,
    SessionUser,
    Named(String),
    Unknown(usize),
}

impl EffectiveRoleProjection {
    /// Produce a validation-only token that cannot collide with valid unquoted SQL identifiers.
    ///
    /// Known role names remain their normalized PostgreSQL identity so a later
    /// named `GRANTED BY` can address the same membership row. Unknown and
    /// connection-relative identities use an `@` terminator, which PostgreSQL
    /// cannot emit as part of an unquoted identifier; quoted names already use
    /// the shared hex sentinel and therefore cannot alias these tokens.
    fn provenance_token(&self) -> String {
        match self {
            Self::InitialCurrentUser => {
                format!("{EXECUTOR_GRANTOR_PREFIX}initial_current_user@")
            }
            Self::SessionUser => format!("{EXECUTOR_GRANTOR_PREFIX}session_user@"),
            Self::Named(role_name) => role_name.clone(),
            Self::Unknown(statement_index) => {
                format!("{EXECUTOR_GRANTOR_PREFIX}unknown_{statement_index}@")
            }
        }
    }
}

/// Project executor-relative grantors onto stable validation-only identities.
///
/// PostgreSQL records the role denoted by `GRANTED BY`, not the literal text of
/// `CURRENT_USER` or `CURRENT_ROLE`. `SET ROLE` can therefore make identical
/// pseudo-target spellings refer to different `pg_auth_members.grantor` rows in
/// one migration. `SESSION_USER` is connection-stable and intentionally kept
/// separate from the mutable effective role. Unknown quoted/string role targets
/// receive a statement-local opaque identity instead of donating false revoke
/// evidence. The returned SQL is consumed only by the grantor provenance
/// validator; executable migration SQL is unchanged.
pub(super) fn project_executor_relative_grantors(sql: &str) -> Option<String> {
    if sql.contains(EXECUTOR_GRANTOR_PREFIX) {
        return None;
    }

    let tokenized = sql.replace(';', " ; ").replace(',', " , ");
    let tokens = tokenized.split_whitespace().collect::<Vec<_>>();
    let mut output = Vec::<String>::with_capacity(tokens.len());
    let mut current_role = EffectiveRoleProjection::InitialCurrentUser;
    let mut statement_index = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        let end = statement_end(&tokens, index);
        let mut statement = tokens[index..end]
            .iter()
            .map(|token| (*token).to_owned())
            .collect::<Vec<_>>();

        update_effective_role(&statement, statement_index, &mut current_role);
        rewrite_granted_by_pseudo_target(&mut statement, &current_role);
        output.extend(statement);
        if end < tokens.len() {
            output.push(";".to_owned());
        }

        index = end.saturating_add(1);
        statement_index = statement_index.saturating_add(1);
    }

    Some(output.join(" "))
}

/// Find the end of one already-normalized semicolon-delimited statement.
fn statement_end(tokens: &[&str], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| *token == ";")
        .map_or(tokens.len(), |relative| start + relative)
}

/// Update the effective current-role projection for one normalized statement.
fn update_effective_role(
    statement: &[String],
    statement_index: usize,
    current_role: &mut EffectiveRoleProjection,
) {
    if statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("RESET"))
        && statement
            .get(1)
            .is_some_and(|token| token.eq_ignore_ascii_case("ROLE"))
    {
        *current_role = EffectiveRoleProjection::InitialCurrentUser;
        return;
    }

    if !statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("SET"))
    {
        return;
    }

    let role_target_index = if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("ROLE"))
    {
        Some(2usize)
    } else if statement.get(1).is_some_and(|token| {
        token.eq_ignore_ascii_case("SESSION") || token.eq_ignore_ascii_case("LOCAL")
    }) && statement
        .get(2)
        .is_some_and(|token| token.eq_ignore_ascii_case("ROLE"))
    {
        Some(3usize)
    } else {
        None
    };

    let Some(target) = role_target_index.and_then(|target_index| statement.get(target_index)) else {
        return;
    };

    if target.eq_ignore_ascii_case("NONE") {
        *current_role = EffectiveRoleProjection::SessionUser;
    } else if role_target_is_statically_named(target) {
        *current_role = EffectiveRoleProjection::Named(target.to_ascii_lowercase());
    } else {
        *current_role = EffectiveRoleProjection::Unknown(statement_index);
    }
}

/// Return whether the normalized SET ROLE target still carries a plain role identity.
fn role_target_is_statically_named(target: &str) -> bool {
    target != INVALID_QUOTED_IDENTIFIER
        && !target.starts_with('\'')
        && !target.contains('@')
        && !target.starts_with("__tepp_quoted_grantor_identity_")
}

/// Replace pseudo-target spellings only in an explicit trailing `GRANTED BY` slot.
fn rewrite_granted_by_pseudo_target(
    statement: &mut [String],
    current_role: &EffectiveRoleProjection,
) {
    let mut index = 0usize;
    while index + 2 < statement.len() {
        if statement[index].eq_ignore_ascii_case("GRANTED")
            && statement[index + 1].eq_ignore_ascii_case("BY")
        {
            let replacement = if statement[index + 2].eq_ignore_ascii_case("CURRENT_USER")
                || statement[index + 2].eq_ignore_ascii_case("CURRENT_ROLE")
            {
                Some(current_role.provenance_token())
            } else if statement[index + 2].eq_ignore_ascii_case("SESSION_USER") {
                Some(EffectiveRoleProjection::SessionUser.provenance_token())
            } else {
                None
            };
            if let Some(replacement) = replacement {
                statement[index + 2] = replacement;
            }
            return;
        }
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::project_executor_relative_grantors;

    #[test]
    fn current_user_follows_set_role_while_session_user_stays_stable() {
        let projected = project_executor_relative_grantors(
            "SET ROLE grantor_a; GRANT reporting_owner TO tepp_app_runtime GRANTED BY CURRENT_USER; SET ROLE grantor_b; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_ROLE; SET ROLE NONE; GRANT reporting_owner TO tepp_app_runtime GRANTED BY SESSION_USER;",
        )
        .expect("normalized SQL must project");

        assert!(projected.contains("GRANTED BY grantor_a"));
        assert!(projected.contains("GRANTED BY grantor_b"));
        assert!(projected.contains("GRANTED BY __tepp_executor_grantor_session_user@"));
    }

    #[test]
    fn reserved_projection_prefix_fails_closed() {
        assert!(
            project_executor_relative_grantors(
                "GRANT reporting_owner TO tepp_app_runtime GRANTED BY __tepp_executor_grantor_session_user@;"
            )
            .is_none()
        );
    }
}
