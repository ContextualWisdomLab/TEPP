//! Execution-state projection for PostgreSQL pseudo-grantor role specifications.
//!
//! The shared lexical authority has already removed comments/literals from
//! structural consideration before this boundary runs. This module therefore
//! does not lex SQL again: it tracks normalized current/session authorization,
//! transaction-local overrides, and rewrites executor-relative `GRANTED BY`
//! pseudo-targets to validation-only provenance identities.

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

/// Session-level `role` setting used to derive PostgreSQL `CURRENT_USER`.
///
/// `SET ROLE NONE` follows the current session user rather than freezing its
/// identity at the time of the command. `RESET ROLE` is kept as a separate
/// connection-default state because the startup `role` setting is outside this
/// bounded migration validator.
#[derive(Clone, Debug, Eq, PartialEq)]
enum CurrentRoleSetting {
    ConnectionDefault,
    FollowSessionUser,
    Explicit(EffectiveRoleProjection),
}

impl CurrentRoleSetting {
    /// Resolve the effective current-user identity against one session-user projection.
    fn resolve(&self, session_role: &EffectiveRoleProjection) -> EffectiveRoleProjection {
        match self {
            Self::ConnectionDefault => EffectiveRoleProjection::InitialCurrentUser,
            Self::FollowSessionUser => session_role.clone(),
            Self::Explicit(role) => role.clone(),
        }
    }
}

/// Session-persistent executor settings captured at transaction entry.
///
/// Ordinary `SET` changes inside a transaction survive COMMIT but disappear on
/// ROLLBACK. The snapshot therefore covers only session-persistent settings;
/// `SET LOCAL` overlays are discarded at every transaction end.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SessionSettingsSnapshot {
    session_role: EffectiveRoleProjection,
    current_setting: CurrentRoleSetting,
}

/// Current/session authorization state used for grantor provenance projection.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ExecutorRoleState {
    session_role: EffectiveRoleProjection,
    current_setting: CurrentRoleSetting,
    local_session_role: Option<EffectiveRoleProjection>,
    local_current_setting: Option<CurrentRoleSetting>,
    transaction_baseline: Option<SessionSettingsSnapshot>,
    savepoint_uncertain: bool,
}

impl ExecutorRoleState {
    /// Start with separate opaque identities for startup current role and authenticated user.
    fn initial() -> Self {
        Self {
            session_role: EffectiveRoleProjection::AuthenticatedUser,
            current_setting: CurrentRoleSetting::ConnectionDefault,
            local_session_role: None,
            local_current_setting: None,
            transaction_baseline: None,
            savepoint_uncertain: false,
        }
    }

    /// Return the session-user identity visible to the current statement.
    fn active_session_role(&self) -> EffectiveRoleProjection {
        self.local_session_role
            .clone()
            .unwrap_or_else(|| self.session_role.clone())
    }

    /// Return the current-user identity after applying a transaction-local role overlay.
    fn active_current_role(&self) -> EffectiveRoleProjection {
        let session_role = self.active_session_role();
        self.local_current_setting
            .as_ref()
            .unwrap_or(&self.current_setting)
            .resolve(&session_role)
    }

    /// Enter an explicit transaction without replacing an existing transaction baseline.
    fn begin_transaction(&mut self) {
        if self.transaction_baseline.is_none() {
            self.transaction_baseline = Some(SessionSettingsSnapshot {
                session_role: self.session_role.clone(),
                current_setting: self.current_setting.clone(),
            });
            self.local_session_role = None;
            self.local_current_setting = None;
            self.savepoint_uncertain = false;
        }
    }

    /// Commit session settings while discarding transaction-local authorization overlays.
    fn commit_transaction(&mut self, and_chain: bool) {
        self.local_session_role = None;
        self.local_current_setting = None;
        self.transaction_baseline = None;
        self.savepoint_uncertain = false;
        if and_chain {
            self.begin_transaction();
        }
    }

    /// Restore transaction-entry session settings and discard local authorization overlays.
    fn rollback_transaction(&mut self, and_chain: bool) {
        if let Some(snapshot) = self.transaction_baseline.take() {
            self.session_role = snapshot.session_role;
            self.current_setting = snapshot.current_setting;
        }
        self.local_session_role = None;
        self.local_current_setting = None;
        self.savepoint_uncertain = false;
        if and_chain {
            self.begin_transaction();
        }
    }

    /// Apply a session-authorization target using PostgreSQL SESSION/LOCAL scope.
    fn set_session_authorization(
        &mut self,
        projection: EffectiveRoleProjection,
        local: bool,
    ) {
        if local {
            if self.transaction_baseline.is_some() {
                self.local_session_role = Some(projection);
                self.local_current_setting = Some(CurrentRoleSetting::FollowSessionUser);
            }
            return;
        }

        self.session_role = projection;
        self.current_setting = CurrentRoleSetting::FollowSessionUser;
        self.local_session_role = None;
        self.local_current_setting = None;
    }

    /// Apply one `SET ROLE` target while preserving PostgreSQL LOCAL transaction scope.
    fn set_role(&mut self, setting: CurrentRoleSetting, local: bool) {
        if local {
            if self.transaction_baseline.is_some() {
                self.local_current_setting = Some(setting);
            }
            return;
        }

        self.current_setting = setting;
        self.local_current_setting = None;
    }
}

/// Project executor-relative grantors onto stable validation-only identities.
///
/// PostgreSQL records the role denoted by `GRANTED BY`, not the literal text of
/// `CURRENT_USER`, `CURRENT_ROLE`, or `SESSION_USER`. `SET ROLE` can change the
/// effective current role; `SET SESSION AUTHORIZATION` can change both session
/// and current identities; `SET LOCAL` overlays disappear at transaction end.
/// Savepoint control is deliberately not modeled as a partial transaction stack:
/// once encountered, pseudo-target uses receive statement-local opaque identities
/// until transaction end so an uncertain rollback path cannot donate false revoke
/// evidence. Executable migration SQL is unchanged.
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
        rewrite_granted_by_pseudo_target(&mut statement, statement_index, &state);
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

/// Update PostgreSQL executor state for one normalized top-level statement.
fn update_executor_role_state(
    statement: &[String],
    statement_index: usize,
    state: &mut ExecutorRoleState,
) {
    if is_transaction_start(statement) {
        state.begin_transaction();
        return;
    }

    if is_savepoint_control(statement) {
        if state.transaction_baseline.is_some() {
            state.savepoint_uncertain = true;
        }
        return;
    }

    if let Some((commit, and_chain)) = transaction_end(statement) {
        if commit {
            state.commit_transaction(and_chain);
        } else {
            state.rollback_transaction(and_chain);
        }
        return;
    }

    if is_reset_session_authorization(statement) {
        state.set_session_authorization(EffectiveRoleProjection::AuthenticatedUser, false);
        return;
    }

    if let Some((target_index, local)) = session_authorization_target(statement) {
        let Some(target) = statement.get(target_index) else {
            return;
        };
        let projection = if target.eq_ignore_ascii_case("DEFAULT") {
            EffectiveRoleProjection::AuthenticatedUser
        } else {
            target_role_projection(target, statement_index)
        };
        state.set_session_authorization(projection, local);
        return;
    }

    if statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("RESET"))
        && statement
            .get(1)
            .is_some_and(|token| token.eq_ignore_ascii_case("ROLE"))
    {
        state.set_role(CurrentRoleSetting::ConnectionDefault, false);
        return;
    }

    let Some((target_index, local)) = role_target(statement) else {
        return;
    };
    let Some(target) = statement.get(target_index) else {
        return;
    };
    let setting = if target.eq_ignore_ascii_case("NONE") {
        CurrentRoleSetting::FollowSessionUser
    } else {
        CurrentRoleSetting::Explicit(target_role_projection(target, statement_index))
    };
    state.set_role(setting, local);
}

/// Return whether the statement starts an explicit PostgreSQL transaction block.
fn is_transaction_start(statement: &[String]) -> bool {
    statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("BEGIN"))
        || (statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("START"))
            && statement
                .get(1)
                .is_some_and(|token| token.eq_ignore_ascii_case("TRANSACTION")))
}

/// Mark savepoint-sensitive state as uncertain rather than pretending to model a stack.
fn is_savepoint_control(statement: &[String]) -> bool {
    statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("SAVEPOINT"))
        || statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("RELEASE"))
        || (statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("ROLLBACK"))
            && statement
                .get(1)
                .is_some_and(|token| token.eq_ignore_ascii_case("TO")))
}

/// Return transaction-end kind and whether PostgreSQL immediately chains a new transaction.
fn transaction_end(statement: &[String]) -> Option<(bool, bool)> {
    let first = statement.first()?;
    let commit = if first.eq_ignore_ascii_case("COMMIT") {
        if statement
            .get(1)
            .is_some_and(|token| token.eq_ignore_ascii_case("PREPARED"))
        {
            return None;
        }
        true
    } else if first.eq_ignore_ascii_case("END") {
        true
    } else if first.eq_ignore_ascii_case("ROLLBACK") {
        if statement.get(1).is_some_and(|token| {
            token.eq_ignore_ascii_case("TO") || token.eq_ignore_ascii_case("PREPARED")
        }) {
            return None;
        }
        false
    } else {
        return None;
    };

    let and_chain = statement
        .windows(2)
        .any(|pair| pair[0].eq_ignore_ascii_case("AND") && pair[1].eq_ignore_ascii_case("CHAIN"))
        || statement.windows(3).any(|triple| {
            triple[0].eq_ignore_ascii_case("AND")
                && triple[1].eq_ignore_ascii_case("NO")
                && triple[2].eq_ignore_ascii_case("CHAIN")
        });
    Some((commit, and_chain))
}

/// Return target index and LOCAL scope for PostgreSQL session-authorization syntax.
fn session_authorization_target(statement: &[String]) -> Option<(usize, bool)> {
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
        return Some((3, false));
    }

    if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("LOCAL"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("SESSION"))
        && statement
            .get(3)
            .is_some_and(|token| token.eq_ignore_ascii_case("AUTHORIZATION"))
    {
        return Some((4, true));
    }

    if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("SESSION"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("SESSION"))
        && statement
            .get(3)
            .is_some_and(|token| token.eq_ignore_ascii_case("AUTHORIZATION"))
    {
        return Some((4, false));
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

/// Return target index and LOCAL scope for PostgreSQL `SET [SESSION|LOCAL] ROLE`.
fn role_target(statement: &[String]) -> Option<(usize, bool)> {
    if !statement
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("SET"))
    {
        return None;
    }

    if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("ROLE"))
    {
        return Some((2, false));
    }
    if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("LOCAL"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("ROLE"))
    {
        return Some((3, true));
    }
    if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("SESSION"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("ROLE"))
    {
        return Some((3, false));
    }
    None
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
///
/// Savepoint-sensitive state receives a statement-local identity because this
/// bounded authority intentionally does not guess which earlier SET operation a
/// later `ROLLBACK TO` preserved. That can reject an otherwise safe migration,
/// but it cannot turn uncertain provenance into false revocation evidence.
fn rewrite_granted_by_pseudo_target(
    statement: &mut [String],
    statement_index: usize,
    state: &ExecutorRoleState,
) {
    let mut index = 0usize;
    while index + 2 < statement.len() {
        if statement[index].eq_ignore_ascii_case("GRANTED")
            && statement[index + 1].eq_ignore_ascii_case("BY")
        {
            let replacement = if statement[index + 2].eq_ignore_ascii_case("CURRENT_USER")
                || statement[index + 2].eq_ignore_ascii_case("CURRENT_ROLE")
            {
                if state.savepoint_uncertain {
                    Some(EffectiveRoleProjection::Unknown(statement_index).provenance_token())
                } else {
                    Some(state.active_current_role().provenance_token())
                }
            } else if statement[index + 2].eq_ignore_ascii_case("SESSION_USER") {
                if state.savepoint_uncertain {
                    Some(EffectiveRoleProjection::Unknown(statement_index).provenance_token())
                } else {
                    Some(state.active_session_role().provenance_token())
                }
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
    fn local_role_is_discarded_at_commit() {
        let projected = project_executor_relative_grantors(
            "SET ROLE grantor_a; BEGIN; SET LOCAL ROLE grantor_b; GRANT reporting_owner TO tepp_app_runtime GRANTED BY CURRENT_USER; COMMIT; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_USER;",
        )
        .expect("normalized SQL must project");

        assert!(projected.contains("GRANTED BY grantor_b"));
        assert!(projected.contains("GRANTED BY grantor_a"));
    }

    #[test]
    fn local_session_authorization_is_discarded_at_rollback() {
        let projected = project_executor_relative_grantors(
            "SET SESSION AUTHORIZATION grantor_a; BEGIN; SET LOCAL SESSION AUTHORIZATION grantor_b; GRANT reporting_owner TO tepp_app_runtime GRANTED BY SESSION_USER; ROLLBACK; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY SESSION_USER;",
        )
        .expect("normalized SQL must project");

        assert!(projected.contains("GRANTED BY grantor_b"));
        assert!(projected.contains("GRANTED BY grantor_a"));
    }

    #[test]
    fn regular_transaction_setting_rolls_back_to_entry_state() {
        let projected = project_executor_relative_grantors(
            "SET ROLE grantor_a; BEGIN; SET ROLE grantor_b; ROLLBACK; GRANT reporting_owner TO tepp_app_runtime GRANTED BY CURRENT_USER;",
        )
        .expect("normalized SQL must project");

        assert!(projected.contains("GRANTED BY grantor_a"));
    }

    #[test]
    fn savepoint_control_uses_statement_local_opaque_grantors_until_transaction_end() {
        let projected = project_executor_relative_grantors(
            "BEGIN; SAVEPOINT before_role; SET ROLE grantor_a; GRANT reporting_owner TO tepp_app_runtime GRANTED BY CURRENT_USER; ROLLBACK TO before_role; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY CURRENT_USER; COMMIT;",
        )
        .expect("normalized SQL must project");

        assert!(projected.contains("GRANTED BY __tepp_executor_grantor_unknown_3@"));
        assert!(projected.contains("GRANTED BY __tepp_executor_grantor_unknown_5@"));
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
