//! Fail-closed PostgreSQL role-membership boundary for the application runtime.
//!
//! Direct role attributes and lifecycle remain owned by `migration_validation`.
//! This module owns the complementary membership invariant: an RLS-protected
//! application runtime must not be able to become another role with `SET ROLE`
//! or hold `ADMIN` on that role, which would let it grant the role back to
//! itself with `SET TRUE`.

use std::collections::BTreeMap;

const RUNTIME_ROLE: &str = "tepp_app_runtime";
const CREATE_IN_ROLE_SENTINEL: &str = "__create_in_role_membership__";
const MALFORMED_GRANTEE_SENTINEL: &str = "__malformed_membership_grantee_list__";

/// Security-relevant options for one runtime membership edge.
///
/// `SET` is the direct `SET ROLE` capability. `ADMIN` is equally security
/// relevant because PostgreSQL allows an ADMIN member to grant the role back
/// to itself with a different SET value. A membership is therefore safe for the
/// RLS runtime only when both options are false.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MembershipSecurityState {
    set_enabled: bool,
    admin_enabled: bool,
}

impl MembershipSecurityState {
    /// PostgreSQL defaults for a newly created role membership.
    const fn new() -> Self {
        Self {
            set_enabled: true,
            admin_enabled: false,
        }
    }

    /// Return whether the runtime can reach or manufacture a SET ROLE path.
    const fn can_escape_runtime_identity(self) -> bool {
        self.set_enabled || self.admin_enabled
    }
}

/// Explicit security-relevant options supplied by one membership GRANT.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ExplicitMembershipOptions {
    set_enabled: Option<bool>,
    admin_enabled: Option<bool>,
}

/// Return whether the normalized migration's final runtime memberships disable SET ROLE escalation.
///
/// Object-privilege grants/revokes fall outside the bounded membership grammar
/// because their privilege/object tokens do not form a comma-separated role
/// list before `TO`/`FROM`. New memberships default to `SET TRUE, ADMIN FALSE`;
/// later GRANTs retain omitted options. Both SET and ADMIN must be false in the
/// final state: PostgreSQL documents that ADMIN can be used to grant the role
/// back to oneself with SET enabled. `CREATE ROLE ... IN ROLE ...` and its
/// deprecated `IN GROUP` alias remain conservatively SET-capable.
pub(super) fn runtime_membership_is_rls_safe(sql: &str) -> bool {
    let tokenized = sql.replace(';', " ; ").replace(',', " , ");
    let tokens = tokenized.split_whitespace().collect::<Vec<_>>();
    let mut memberships = BTreeMap::<String, MembershipSecurityState>::new();
    let mut index = 0usize;

    while index < tokens.len() {
        let end = statement_end(&tokens, index);
        let statement = &tokens[index..end];
        if statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("GRANT"))
        {
            apply_runtime_membership_grant(statement, &mut memberships);
        } else if statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("REVOKE"))
        {
            apply_runtime_membership_revoke(statement, &mut memberships);
        } else if create_runtime_role_in_role(statement) {
            memberships.insert(
                CREATE_IN_ROLE_SENTINEL.to_owned(),
                MembershipSecurityState::new(),
            );
        }
        index = end.saturating_add(1);
    }

    memberships
        .values()
        .all(|state| !state.can_escape_runtime_identity())
}

/// Return the exclusive end of the semicolon-delimited statement containing `start`.
fn statement_end(tokens: &[&str], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| *token == ";")
        .map_or(tokens.len(), |relative| start + relative)
}

/// Locate `TO`/`FROM` only after a complete comma-separated membership role list.
///
/// The lexical boundary intentionally projects identity-equivalent lowercase
/// quoted identifiers onto bare tokens, so a role literally named `"to"` or
/// `"from"` is indistinguishable by spelling alone. Position resolves that
/// ambiguity: while a role is expected the token is a role name; only after a
/// complete role and before another comma can the delimiter keyword terminate
/// the granted-role list. Object-privilege statements naturally return `None`
/// because `ON`/object syntax interrupts this membership list grammar.
fn membership_delimiter(
    statement: &[&str],
    roles_start: usize,
    delimiter_keyword: &str,
) -> Option<usize> {
    let mut index = roles_start;
    let mut expects_role = true;
    let mut saw_role = false;

    while let Some(token) = statement.get(index) {
        if expects_role {
            if *token == "," {
                return None;
            }
            saw_role = true;
            expects_role = false;
        } else if *token == "," {
            expects_role = true;
        } else if token.eq_ignore_ascii_case(delimiter_keyword) {
            return saw_role.then_some(index);
        } else {
            return None;
        }
        index += 1;
    }
    None
}

/// Parse the comma-separated membership grantee list after `TO`/`FROM`.
///
/// Returns whether the runtime is one of the grantees plus the first token after
/// the role list. The parser uses list position rather than keyword spelling so
/// valid quoted grantees such as `"with"`, `"granted"`, or `"cascade"` cannot
/// impersonate trailing clauses after lexical projection. Empty, leading-comma,
/// and trailing-comma lists return `None` so the membership boundary fails closed.
fn runtime_grantee_list(statement: &[&str], delimiter: usize) -> Option<(bool, usize)> {
    let mut index = delimiter + 1;
    let mut expects_role = true;
    let mut saw_role = false;
    let mut runtime_is_grantee = false;

    while let Some(token) = statement.get(index) {
        if expects_role {
            if *token == "," {
                return None;
            }
            saw_role = true;
            runtime_is_grantee |= token.eq_ignore_ascii_case(RUNTIME_ROLE);
            expects_role = false;
            index += 1;
            continue;
        }
        if *token == "," {
            expects_role = true;
            index += 1;
            continue;
        }
        break;
    }

    if !saw_role || expects_role {
        None
    } else {
        Some((runtime_is_grantee, index))
    }
}

/// Apply one PostgreSQL role-membership GRANT that targets the application runtime.
///
/// The granted-role and grantee lists are parsed positionally before optional
/// clauses are inspected. New memberships use PostgreSQL's SET-true/ADMIN-false
/// defaults; later GRANTs update only explicitly supplied security options.
fn apply_runtime_membership_grant(
    statement: &[&str],
    memberships: &mut BTreeMap<String, MembershipSecurityState>,
) {
    let Some(to_index) = membership_delimiter(statement, 1, "TO") else {
        return;
    };
    let Some((targets_runtime, trailing_start)) = runtime_grantee_list(statement, to_index) else {
        memberships.insert(
            MALFORMED_GRANTEE_SENTINEL.to_owned(),
            MembershipSecurityState::new(),
        );
        return;
    };
    if !targets_runtime {
        return;
    }

    let options = explicit_membership_options(statement, trailing_start);
    for role in membership_role_names(&statement[1..to_index]) {
        match memberships.get_mut(&role) {
            Some(state) => {
                if let Some(set_enabled) = options.set_enabled {
                    state.set_enabled = set_enabled;
                }
                if let Some(admin_enabled) = options.admin_enabled {
                    state.admin_enabled = admin_enabled;
                }
            }
            None => {
                let mut state = MembershipSecurityState::new();
                if let Some(set_enabled) = options.set_enabled {
                    state.set_enabled = set_enabled;
                }
                if let Some(admin_enabled) = options.admin_enabled {
                    state.admin_enabled = admin_enabled;
                }
                memberships.insert(role, state);
            }
        }
    }
}

/// Security-relevant option targeted by a membership-option REVOKE.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RevokedMembershipOption {
    Set,
    Admin,
}

/// Apply one PostgreSQL role-membership REVOKE that targets the application runtime.
///
/// Plain membership REVOKE removes the edge. `REVOKE SET OPTION FOR` and
/// `REVOKE ADMIN OPTION FOR` mutate only existing memberships; neither creates a
/// phantom safe state. INHERIT-option revocation is outside the SET/ADMIN escape
/// invariant and leaves this map unchanged. Object privilege revokes stay
/// outside the state map because they do not match the bounded role-list grammar.
fn apply_runtime_membership_revoke(
    statement: &[&str],
    memberships: &mut BTreeMap<String, MembershipSecurityState>,
) {
    let (roles_start, revoked_option) = if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("SET"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("OPTION"))
        && statement
            .get(3)
            .is_some_and(|token| token.eq_ignore_ascii_case("FOR"))
    {
        (4usize, Some(RevokedMembershipOption::Set))
    } else if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("ADMIN"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("OPTION"))
        && statement
            .get(3)
            .is_some_and(|token| token.eq_ignore_ascii_case("FOR"))
    {
        (4usize, Some(RevokedMembershipOption::Admin))
    } else if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("INHERIT"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("OPTION"))
        && statement
            .get(3)
            .is_some_and(|token| token.eq_ignore_ascii_case("FOR"))
    {
        return;
    } else {
        (1usize, None)
    };

    let Some(from_index) = membership_delimiter(statement, roles_start, "FROM") else {
        return;
    };
    let Some((targets_runtime, _trailing_start)) = runtime_grantee_list(statement, from_index) else {
        memberships.insert(
            MALFORMED_GRANTEE_SENTINEL.to_owned(),
            MembershipSecurityState::new(),
        );
        return;
    };
    if !targets_runtime {
        return;
    }

    for role in membership_role_names(&statement[roles_start..from_index]) {
        match revoked_option {
            Some(RevokedMembershipOption::Set) => {
                if let Some(state) = memberships.get_mut(&role) {
                    state.set_enabled = false;
                }
            }
            Some(RevokedMembershipOption::Admin) => {
                if let Some(state) = memberships.get_mut(&role) {
                    state.admin_enabled = false;
                }
            }
            None => {
                memberships.remove(&role);
            }
        }
    }
}

/// Return normalized role names from a validated comma-separated membership role list.
///
/// PostgreSQL's role-membership GRANT form does not allow `GROUP` as a noise
/// word in the granted-role specification. A normalized `group` token at a role
/// position can therefore represent a quoted role named `"group"` and must be
/// preserved rather than discarded.
fn membership_role_names(tokens: &[&str]) -> Vec<String> {
    tokens
        .iter()
        .filter(|token| **token != ",")
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

/// Return explicitly supplied SET and ADMIN membership options.
///
/// `trailing_start` is the first token after the complete grantee list, so role
/// names projected from quoted keywords cannot impersonate the `WITH` clause.
/// PostgreSQL accepts `OPTION` as the true spelling. Malformed or missing values
/// after a recognized security option map to true so the boundary fails closed.
fn explicit_membership_options(
    statement: &[&str],
    trailing_start: usize,
) -> ExplicitMembershipOptions {
    let Some(with_index) = statement[trailing_start..]
        .iter()
        .position(|token| token.eq_ignore_ascii_case("WITH"))
        .map(|relative| trailing_start + relative)
    else {
        return ExplicitMembershipOptions::default();
    };

    let mut options = ExplicitMembershipOptions::default();
    let mut index = with_index + 1;
    while index < statement.len() {
        if statement[index].eq_ignore_ascii_case("GRANTED") {
            break;
        }
        let value = statement
            .get(index + 1)
            .map(|token| !token.eq_ignore_ascii_case("FALSE"))
            .unwrap_or(true);
        if statement[index].eq_ignore_ascii_case("SET") {
            options.set_enabled = Some(value);
            index += 2;
            continue;
        }
        if statement[index].eq_ignore_ascii_case("ADMIN") {
            options.admin_enabled = Some(value);
            index += 2;
            continue;
        }
        index += 1;
    }
    options
}

/// Return whether canonicalized role creation adds the runtime to another role.
///
/// `migration_validation` maps CREATE ROLE/USER/GROUP to CREATE TYPE for the
/// shared object-name parser while leaving role attributes in place. PostgreSQL
/// creates both `IN ROLE` and deprecated `IN GROUP` memberships with SET
/// enabled, so either creation shortcut violates the RLS contract. `ROLE` and
/// `ADMIN` clauses point in the opposite membership direction and are not
/// treated as runtime escape paths here.
fn create_runtime_role_in_role(statement: &[&str]) -> bool {
    if statement.len() < 3
        || !statement[0].eq_ignore_ascii_case("CREATE")
        || !statement[1].eq_ignore_ascii_case("TYPE")
        || !statement[2].eq_ignore_ascii_case(RUNTIME_ROLE)
    {
        return false;
    }
    statement[3..].windows(2).any(|window| {
        window[0].eq_ignore_ascii_case("IN")
            && (window[1].eq_ignore_ascii_case("ROLE")
                || window[1].eq_ignore_ascii_case("GROUP"))
    })
}

#[cfg(test)]
mod tests {
    use super::runtime_membership_is_rls_safe;

    #[test]
    fn object_grants_and_inverse_membership_do_not_give_runtime_set_role() {
        for sql in [
            "GRANT SELECT ON TABLE tenant_record TO tepp_app_runtime ;",
            "GRANT SET ON PARAMETER work_mem TO tepp_app_runtime ;",
            "GRANT tepp_app_runtime TO CURRENT_USER ;",
        ] {
            assert!(runtime_membership_is_rls_safe(sql), "{sql}");
        }
    }

    #[test]
    fn set_or_admin_capable_runtime_memberships_fail_closed() {
        for sql in [
            "GRANT reporting_operator TO tepp_app_runtime ;",
            "GRANT reporting_operator TO tepp_app_runtime WITH SET TRUE ;",
            "GRANT reporting_operator TO tepp_app_runtime WITH SET OPTION ;",
            "GRANT reporting_operator TO tepp_app_runtime WITH SET FALSE , ADMIN TRUE ;",
            "GRANT reporting_operator TO tepp_app_runtime WITH ADMIN TRUE , SET FALSE ;",
            "GRANT reporting_operator TO tepp_app_runtime WITH SET FALSE ; GRANT reporting_operator TO tepp_app_runtime WITH ADMIN TRUE ;",
            "GRANT on TO tepp_app_runtime ;",
            "GRANT reporting_operator , on TO tepp_app_runtime ;",
            "GRANT to TO tepp_app_runtime ;",
            "GRANT group TO tepp_app_runtime ;",
            "GRANT reporting_operator TO with , tepp_app_runtime ;",
            "GRANT reporting_operator TO granted , tepp_app_runtime ;",
            "CREATE TYPE tepp_app_runtime NOSUPERUSER NOBYPASSRLS IN ROLE reporting_operator ;",
            "CREATE TYPE tepp_app_runtime NOSUPERUSER NOBYPASSRLS IN GROUP reporting_operator ;",
            "REVOKE SET OPTION FOR reporting_operator FROM tepp_app_runtime ; GRANT reporting_operator TO tepp_app_runtime ;",
        ] {
            assert!(!runtime_membership_is_rls_safe(sql), "{sql}");
        }
    }

    #[test]
    fn explicit_security_option_repair_or_later_revoke_restores_membership_safety() {
        for sql in [
            "GRANT reporting_operator TO tepp_app_runtime WITH INHERIT TRUE , SET FALSE ;",
            "GRANT reporting_operator TO with , tepp_app_runtime WITH SET FALSE ;",
            "GRANT reporting_operator TO tepp_app_runtime ; REVOKE SET OPTION FOR reporting_operator FROM tepp_app_runtime ;",
            "GRANT reporting_operator TO tepp_app_runtime ; REVOKE reporting_operator FROM tepp_app_runtime ;",
            "GRANT reporting_operator TO tepp_app_runtime ; REVOKE reporting_operator FROM cascade , tepp_app_runtime ;",
            "GRANT reporting_operator TO tepp_app_runtime ; GRANT reporting_operator TO tepp_app_runtime WITH SET FALSE ;",
            "GRANT reporting_operator TO tepp_app_runtime WITH SET FALSE , ADMIN TRUE ; REVOKE ADMIN OPTION FOR reporting_operator FROM tepp_app_runtime ;",
            "GRANT reporting_operator TO tepp_app_runtime WITH SET FALSE , ADMIN TRUE ; GRANT reporting_operator TO tepp_app_runtime WITH ADMIN FALSE ;",
        ] {
            assert!(runtime_membership_is_rls_safe(sql), "{sql}");
        }
    }
}
