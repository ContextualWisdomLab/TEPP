//! Grantor-aware safety evidence for PostgreSQL runtime-role memberships.
//!
//! `pg_auth_members` records one row per role/member/grantor relationship. The
//! aggregate membership validator intentionally models the effective runtime
//! edge, while this companion boundary preserves explicit `GRANTED BY`
//! provenance so revoking one grantor cannot erase a still-dangerous grant from
//! another grantor. Statements without explicit grantor provenance never erase
//! explicit rows here; the bounded validator cannot prove which recorded grant
//! an implicit executor would revoke.

use std::collections::BTreeMap;

const RUNTIME_ROLE: &str = "tepp_app_runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GrantorMembershipState {
    set_enabled: bool,
    admin_enabled: bool,
    inherit_enabled: bool,
}

impl GrantorMembershipState {
    /// Conservative PostgreSQL defaults for a newly observed membership row.
    ///
    /// SET defaults true and ADMIN false. INHERIT on a new membership depends
    /// on the member role's inheritance attribute; this bounded authority does
    /// not independently prove runtime NOINHERIT, so omitted INHERIT remains
    /// unsafe until explicit false evidence appears.
    const fn new() -> Self {
        Self {
            set_enabled: true,
            admin_enabled: false,
            inherit_enabled: true,
        }
    }

    const fn can_escape_runtime_identity(self) -> bool {
        self.set_enabled || self.admin_enabled || self.inherit_enabled
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ExplicitMembershipOptions {
    set_enabled: Option<bool>,
    admin_enabled: Option<bool>,
    inherit_enabled: Option<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RevokedMembershipOption {
    Set,
    Admin,
    Inherit,
}

/// Return whether every explicitly grantor-attributed runtime membership is safe.
///
/// PostgreSQL can retain more than one membership row for the same granted role
/// and member when grantors differ. Each explicit grantor path is therefore
/// tracked independently. An implicit REVOKE is deliberately not allowed to
/// delete explicit rows because executor/grantor selection is outside this
/// static migration boundary; that uncertainty must fail closed rather than
/// donate false RLS-safety evidence.
pub(super) fn runtime_membership_grantors_are_rls_safe(sql: &str) -> bool {
    let tokenized = sql.replace(';', " ; ").replace(',', " , ");
    let tokens = tokenized.split_whitespace().collect::<Vec<_>>();
    let mut grants = BTreeMap::<(String, String), GrantorMembershipState>::new();
    let mut index = 0usize;

    while index < tokens.len() {
        let end = statement_end(&tokens, index);
        let statement = &tokens[index..end];
        if statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("GRANT"))
        {
            apply_explicit_grant(statement, &mut grants);
        } else if statement
            .first()
            .is_some_and(|token| token.eq_ignore_ascii_case("REVOKE"))
        {
            apply_explicit_revoke(statement, &mut grants);
        }
        index = end.saturating_add(1);
    }

    grants
        .values()
        .all(|state| !state.can_escape_runtime_identity())
}

fn statement_end(tokens: &[&str], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| *token == ";")
        .map_or(tokens.len(), |relative| start + relative)
}

/// Locate the membership `TO`/`FROM` delimiter after a complete role list.
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

/// Parse the complete grantee list and return whether it contains the runtime.
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

fn membership_role_names(tokens: &[&str]) -> Vec<String> {
    tokens
        .iter()
        .filter(|token| **token != ",")
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

/// Return the explicit grantor following a trailing `GRANTED BY` clause.
///
/// The scan starts only after the complete grantee list, so quoted grantee names
/// projected to keyword-looking spellings cannot impersonate this clause.
fn explicit_grantor(statement: &[&str], trailing_start: usize) -> Option<String> {
    statement[trailing_start..]
        .windows(3)
        .find(|window| {
            window[0].eq_ignore_ascii_case("GRANTED")
                && window[1].eq_ignore_ascii_case("BY")
        })
        .map(|window| window[2].to_ascii_lowercase())
}

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
        if statement[index].eq_ignore_ascii_case("INHERIT") {
            options.inherit_enabled = Some(value);
            index += 2;
            continue;
        }
        index += 1;
    }
    options
}

fn apply_explicit_grant(
    statement: &[&str],
    grants: &mut BTreeMap<(String, String), GrantorMembershipState>,
) {
    let Some(to_index) = membership_delimiter(statement, 1, "TO") else {
        return;
    };
    let Some((targets_runtime, trailing_start)) = runtime_grantee_list(statement, to_index) else {
        return;
    };
    if !targets_runtime {
        return;
    }
    let Some(grantor) = explicit_grantor(statement, trailing_start) else {
        return;
    };
    let options = explicit_membership_options(statement, trailing_start);

    for role in membership_role_names(&statement[1..to_index]) {
        let state = grants
            .entry((role, grantor.clone()))
            .or_insert_with(GrantorMembershipState::new);
        if let Some(set_enabled) = options.set_enabled {
            state.set_enabled = set_enabled;
        }
        if let Some(admin_enabled) = options.admin_enabled {
            state.admin_enabled = admin_enabled;
        }
        if let Some(inherit_enabled) = options.inherit_enabled {
            state.inherit_enabled = inherit_enabled;
        }
    }
}

fn apply_explicit_revoke(
    statement: &[&str],
    grants: &mut BTreeMap<(String, String), GrantorMembershipState>,
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
        (4usize, Some(RevokedMembershipOption::Inherit))
    } else {
        (1usize, None)
    };

    let Some(from_index) = membership_delimiter(statement, roles_start, "FROM") else {
        return;
    };
    let Some((targets_runtime, trailing_start)) = runtime_grantee_list(statement, from_index) else {
        return;
    };
    if !targets_runtime {
        return;
    }
    let Some(grantor) = explicit_grantor(statement, trailing_start) else {
        // The static boundary cannot prove which explicit grantor row an
        // executor-relative REVOKE would select, so explicit rows stay intact.
        return;
    };

    for role in membership_role_names(&statement[roles_start..from_index]) {
        let key = (role, grantor.clone());
        match revoked_option {
            Some(RevokedMembershipOption::Set) => {
                if let Some(state) = grants.get_mut(&key) {
                    state.set_enabled = false;
                }
            }
            Some(RevokedMembershipOption::Admin) => {
                if let Some(state) = grants.get_mut(&key) {
                    state.admin_enabled = false;
                }
            }
            Some(RevokedMembershipOption::Inherit) => {
                if let Some(state) = grants.get_mut(&key) {
                    state.inherit_enabled = false;
                }
            }
            None => {
                grants.remove(&key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::runtime_membership_grantors_are_rls_safe;

    #[test]
    fn distinct_grantors_remain_independent_until_each_path_is_safe_or_revoked() {
        let unsafe_path_remains = "GRANT reporting_owner TO tepp_app_runtime WITH INHERIT FALSE , SET TRUE , ADMIN FALSE GRANTED BY grantor_a ; GRANT reporting_owner TO tepp_app_runtime WITH INHERIT FALSE , SET FALSE , ADMIN FALSE GRANTED BY grantor_b ; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY grantor_b ;";
        assert!(!runtime_membership_grantors_are_rls_safe(unsafe_path_remains));

        let both_paths_removed = "GRANT reporting_owner TO tepp_app_runtime WITH INHERIT FALSE , SET TRUE , ADMIN FALSE GRANTED BY grantor_a ; GRANT reporting_owner TO tepp_app_runtime WITH INHERIT FALSE , SET FALSE , ADMIN FALSE GRANTED BY grantor_b ; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY grantor_b ; REVOKE reporting_owner FROM tepp_app_runtime GRANTED BY grantor_a ;";
        assert!(runtime_membership_grantors_are_rls_safe(both_paths_removed));
    }
}
