//! Execution-state projection for PostgreSQL pseudo-grantor role specifications.
//!
//! The shared lexical authority has already removed comments/literals from
//! structural consideration before this boundary runs. This module therefore
//! does not lex SQL again: it tracks only normalized statement tokens that
//! change current/session authorization and rewrites executor-relative
//! `GRANTED BY` pseudo-targets to validation-only provenance identities.

const EXECUTOR_GRANTOR_PREFIX: &str = "__tepp_executor_grantor_";
const INVALID_QUOTED_IDENTIFIER: &str = "INVALID_QUOTED_IDENTIFIER";

/// PostgreSQL role identity used by one executor-authority slot.
///
/// The connection-time current-role setting and originally authenticated user
/// are distinct because `RESET ROLE` and `RESET SESSION AUTHORIZATION` restore
/// different PostgreSQL concepts. Named roles are exact normalized identities.
/// Unknown targets are scoped to the statement that introduced them so later
/// independently unknown authorization changes cannot alias each other.
#[derive(Clone, Debug, Eq, PartialEq)]
enum EffectiveRoleProjection {
    InitialCurrentUser,
    AuthenticatedUser,
    Named(String),
    Unknown(usize),
}

impl EffectiveRoleProjection {
    /// Produce a validation-only token that cannot collide with valid unquoted SQL identifiers.
    ///
    /// Known role names remain their normalized PostgreSQL identity so a later
    /// named `GRANTED BY` can address the same membership row. Connection- and
    /// uncertainty-relative identities use an `@` terminator, which PostgreSQL
    /// cannot emit as part of an unquoted identifier; quoted names already use
    /// the shared hex sentinel and therefore cannot alias these tokens.
    fn provenance_token(&self) -> String {
        match self {
            Self::InitialCurrentUser => {
                format!("{EXECUTOR_GRANTOR_PREFIX}initial_current_user@")
            }
            Self::AuthenticatedUser => {
                format!("{EXECUTOR_GRANTOR_PREFIX}session_user@")
            }
            Self::Named(role_name) => role_name.clone(),
            Self::Unknown(statement_index) => {
                format!("{EXECUTOR_GRANTOR_PREFIX}unknown_{statement_index}@")
            }
        }
    }
}

/// Current and session user projections for PostgreSQL executor-relative grantors.
///
/// PostgreSQL allows `SET ROLE` to change only the current user, while
/// `SET SESSION AUTHORIZATION` changes both session and current users. Keeping
/// both slots prevents `GRANTED BY SESSION_USER` from collapsing rows across a
/// session-authorization change and lets `SET ROLE NONE` restore the then-current
/// session user rather than a fixed process-wide sentinel.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ExecutorRoleState {
    current_role: EffectiveRoleProjection,
    session_role: EffectiveRoleProjection,
}

impl ExecutorRoleState {
    /// Start with distinct opaque identities for connection-time current role and authenticated user.
    ///
    /// They are intentionally not assumed equal because PostgreSQL can have a
    /// connection-time `role` setting that `RESET ROLE` restores independently
    /// from the authenticated/session user restored by session-authorization reset.
    const fn initial() -> Self {
        Self {
            current_role: EffectiveRoleProjection::InitialCurrentUser,
            session_role: EffectiveRoleProjection::AuthenticatedUser,
        }
    }
}

/// Project executor-relative grantors onto stable validation-only identities.
///
/// PostgreSQL records the role denoted by `GRANTED BY`, not the literal text of
/// `CURRENT_USER`, `CURRENT_ROLE`, or `SESSION_USER`. `SET ROLE` can change the
/// effective current role; `SET SESSION AUTHORIZATION` can change both session
/// and current identities. Identical pseudo-target spellings can therefore
/// refer to different `pg_auth_members.grantor` rows in one migration. Unknown
/// quoted/string authorization targets receive statement-scoped opaque identity
/// instead of donating false revoke evidence. The returned SQL is consumed only
/// by the grantor provenance validator; executable migration SQL is unchanged.
pub(super) fn project_executor_relative_grantors(sql: &str) -> Option<String> {
    if sql.contains(EXECUTOR_GRANTOR_PREFIX) {
        return None;
    }

    let tokenized = sql.replace(';', " ; ").replace(',', " , ");
    let tokens = tokenized.split_whitespace().collect::<Vec<_>>();
    let mut output = Vec::<String>::with_capacity(tokens.len());
    let mut state = ExecutorRoleState::initial();
    let mut statement_index = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        let end = statement_end(&tokens, index);
        let mut statement = tokens[index..end]
            .iter()
            .map(|token| (*token).to_owned())
            .collect::<Vec<_>>();

        update_executor_role_state(&statement, statement_index, &mut state);
        rewrite_granted_by_pseudo_target(&mut statement, &state);
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

/// Update PostgreSQL current/session authorization state for one normalized statement.
///
/// `SET SESSION AUTHORIZATION` is evaluated before the narrower `SET ROLE`
/// grammar because it owns both identity slots. `RESET SESSION AUTHORIZATION`
/// and `... DEFAULT` restore the originally authenticated identity. `RESET ROLE`
/// intentionally returns to the opaque connection-time current-role setting,
/// whereas `SET ROLE NONE` copies the current session identity.
fn update_executor_role_state(
    statement: &[String],
    statement_index: usize,
    state: &mut ExecutorRoleState,
) {
    if is_reset_session_authorization(statement) {
        let authenticated = EffectiveRoleProjection::AuthenticatedUser;
        state.session_role = authenticated.clone();
        state.current_role = authenticated;
        return;
    }

    if let Some(target_index) = session_authorization_target_index(statement) {
        let Some(target) = statement.get(target_index) else {
            return;
        };
        let projection = if target.eq_ignore_ascii_case("DEFAULT") {
            EffectiveRoleProjection::AuthenticatedUser
        } else {
            target_role_projection(target, statement_index)
        };
        state.session_role = projection.clone();
        state.current_role = projection;
        return;
    }

    if statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("RESET"))
        && statement
            .get(1)
            .is_some_and(|token| token.eq_ignore_ascii_case("ROLE"))
    {
        state.current_role = EffectiveRoleProjection::InitialCurrentUser;
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
        state.current_role = state.session_role.clone();
    } else {
        state.current_role = target_role_projection(target, statement_index);
    }
}

/// Return the target index for PostgreSQL `SET [SESSION|LOCAL] SESSION AUTHORIZATION`.
fn session_authorization_target_index(statement: &[String]) -> Option<usize> {
    if !statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("SET"))
    {
        return None;
    }

    if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("SESSION"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("AUTHORIZATION"))
    {
        return Some(3);
    }

    if statement.get(1).is_some_and(|token| {
        token.eq_ignore_ascii_case("SESSION") || token.eq_ignore_ascii_case("LOCAL")
    }) && statement
        .get(2)
        .is_some_and(|token| token.eq_ignore_ascii_case("SESSION"))
        && statement
            .get(3)
            .is_some_and(|token| token.eq_ignore_ascii_case("AUTHORIZATION"))
    {
        return Some(4);
    }

    None
}

/// Recognize PostgreSQL `RESET SESSION AUTHORIZATION` without broad RESET parsing.
fn is_reset_session_authorization(statement: &[String]) -> bool {
    statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("RESET"))
        && statement
            .get(1)
            .is_some_and(|token| token.eq_ignore_ascii_case("SESSION"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("AUTHORIZATION"))
}

/// Project one normalized authorization target without interpreting raw SQL again.
fn target_role_projection(target: &str, statement_index: usize) -> EffectiveRoleProjection {
    if role_target_is_statically_named(target) {
        EffectiveRoleProjection::Named(target.to_ascii_lowercase())
    } else {
        EffectiveRoleProjection::Unknown(statement_index)
    }
}

/// Return whether the normalized role target still carries a plain role identity.
fn role_target_is_statically_named(target: &str) -> bool {
    target != INVALID_QUOTED_IDENTIFIER
        && !target.starts_with('\'')
        && !target.contains('@')
        && !target.starts_with("__tepp_quoted_grantor_identity_")
}

/// Replace pseudo-target spellings only in an explicit trailing `GRANTED BY` slot.
fn rewrite_granted_by_pseudo_target(statement: &mut [String], state: &ExecutorRoleState) {
    let mut index = 0usize;
    while index + 2 < statement.len() {
        if statement[index].eq_ignore_ascii_case("GRANTED")
            && statement[index + 1].eq_ignore_ascii_case("BY")
        {
            let replacement = if statement[index + 2].eq_ignore_ascii_case("CURRENT_USER")
                || statement[index + 2].eq_ignore_ascii_case("CURRENT_ROLE")
            {
                Some(state.current_role.provenance_token())
            } else if statement[index + 2].eq_ignore_ascii_case("SESSION_USER") {
                Some(state.session_role.provenance_token())
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
    fn current_user_follows_set_role_while_session_user_stays_authenticated() {
        let projected = project_executor_relative_grantors(
            "SET ROLE grantor_a; GRANT reporting_owner TO tepp_app_runtime GRANTED BY CURRENT_USER; SET ROLE grantor_b; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_ROLE; SET ROLE NONE; GRANT reporting_owner TO tepp_app_runtime GRANTED BY SESSION_USER;",
        )
        .expect("normalized SQL must project");

        assert!(projected.contains("GRANTED BY grantor_a"));
        assert!(projected.contains("GRANTED BY grantor_b"));
        assert!(projected.contains("GRANTED BY __tepp_executor_grantor_session_user@"));
    }

    #[test]
    fn session_authorization_changes_session_and_current_grantor_identity() {
        let projected = project_executor_relative_grantors(
            "SET SESSION AUTHORIZATION grantor_a; GRANT reporting_owner TO tepp_app_runtime GRANTED BY SESSION_USER; GRANT reporting_owner TO tepp_app_runtime GRANTED BY CURRENT_USER; SET SESSION AUTHORIZATION grantor_b; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY SESSION_USER; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_ROLE;",
        )
        .expect("normalized SQL must project");

        assert_eq!(projected.matches("GRANTED BY grantor_a").count(), 2);
        assert_eq!(projected.matches("GRANTED BY grantor_b").count(), 2);
    }

    #[test]
    fn set_role_none_uses_the_current_session_authorization() {
        let projected = project_executor_relative_grantors(
            "SET SESSION AUTHORIZATION grantor_a; SET ROLE grantor_b; SET ROLE NONE; GRANT reporting_owner TO tepp_app_runtime GRANTED BY CURRENT_USER;",
        )
        .expect("normalized SQL must project");

        assert!(projected.contains("GRANTED BY grantor_a"));
    }

    #[test]
    fn reset_session_authorization_restores_authenticated_identity() {
        let projected = project_executor_relative_grantors(
            "SET SESSION AUTHORIZATION grantor_a; RESET SESSION AUTHORIZATION; GRANT reporting_owner TO tepp_app_runtime GRANTED BY SESSION_USER; GRANT reporting_owner TO tepp_app_runtime GRANTED BY CURRENT_USER;",
        )
        .expect("normalized SQL must project");

        assert_eq!(
            projected
                .matches("GRANTED BY __tepp_executor_grantor_session_user@")
                .count(),
            2
        );
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
