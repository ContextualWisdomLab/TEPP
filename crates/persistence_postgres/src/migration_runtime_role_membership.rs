//! Fail-closed PostgreSQL role-membership boundary for the application runtime.
//!
//! Direct role attributes and lifecycle remain owned by `migration_validation`.
//! This module owns the complementary membership invariant: an RLS-protected
//! application runtime must not be able to become another role with `SET ROLE`.
//! PostgreSQL makes `SET` membership transitive and enables it by default, so a
//! runtime with any SET-capable membership can escape the direct
//! `NOSUPERUSER NOBYPASSRLS` proof if the target role is or later becomes
//! privileged.

use std::collections::BTreeMap;

const RUNTIME_ROLE: &str = "tepp_app_runtime";
const CREATE_IN_ROLE_SENTINEL: &str = "__create_in_role_membership__";

/// Return whether the normalized migration's final runtime memberships disable SET ROLE.
///
/// Object-privilege grants/revokes are excluded by the structural `ON` token.
/// For role membership, PostgreSQL defaults SET to true on creation but retains
/// the current option when a later GRANT omits SET; the small state map mirrors
/// that ordering so an explicit later `WITH SET FALSE` can restore safety.
/// `CREATE ROLE ... IN ROLE ...` is represented as `CREATE TYPE` by the existing
/// structural alias canonicalizer and is conservatively SET-capable.
pub(super) fn runtime_membership_is_rls_safe(sql: &str) -> bool {
    let tokenized = sql.replace(';', " ; ").replace(',', " , ");
    let tokens = tokenized.split_whitespace().collect::<Vec<_>>();
    let mut memberships = BTreeMap::<String, bool>::new();
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
            memberships.insert(CREATE_IN_ROLE_SENTINEL.to_owned(), true);
        }
        index = end.saturating_add(1);
    }

    memberships.values().all(|set_enabled| !set_enabled)
}

/// Return the exclusive end of the semicolon-delimited statement containing `start`.
fn statement_end(tokens: &[&str], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| *token == ";")
        .map_or(tokens.len(), |relative| start + relative)
}

/// Apply one PostgreSQL role-membership GRANT that targets the application runtime.
///
/// A structural `ON` before `TO` identifies object privileges and leaves role
/// membership state untouched. New memberships default SET to true; on an
/// existing membership, an omitted SET option retains the previous value.
fn apply_runtime_membership_grant(statement: &[&str], memberships: &mut BTreeMap<String, bool>) {
    let Some(to_index) = statement
        .iter()
        .position(|token| token.eq_ignore_ascii_case("TO"))
    else {
        return;
    };
    if statement[..to_index]
        .iter()
        .any(|token| token.eq_ignore_ascii_case("ON"))
    {
        return;
    }
    if !runtime_is_grantee(statement, to_index, &["WITH", "GRANTED"]) {
        return;
    }

    let set_option = explicit_set_option(statement);
    for role in membership_role_names(&statement[1..to_index]) {
        match set_option {
            Some(set_enabled) => {
                memberships.insert(role, set_enabled);
            }
            None => {
                memberships.entry(role).or_insert(true);
            }
        }
    }
}

/// Apply one PostgreSQL role-membership REVOKE that targets the application runtime.
///
/// Plain membership REVOKE removes the edge. `REVOKE SET OPTION FOR` preserves
/// membership but disables SET ROLE. ADMIN/INHERIT option revocation does not
/// change SET state. Object privilege revokes contain `ON` and are ignored.
fn apply_runtime_membership_revoke(statement: &[&str], memberships: &mut BTreeMap<String, bool>) {
    let Some(from_index) = statement
        .iter()
        .position(|token| token.eq_ignore_ascii_case("FROM"))
    else {
        return;
    };
    if statement[..from_index]
        .iter()
        .any(|token| token.eq_ignore_ascii_case("ON"))
    {
        return;
    }
    if !runtime_is_grantee(statement, from_index, &["GRANTED", "CASCADE", "RESTRICT"]) {
        return;
    }

    let (roles_start, revoke_set_only) = if statement
        .get(1)
        .is_some_and(|token| token.eq_ignore_ascii_case("SET"))
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("OPTION"))
        && statement
            .get(3)
            .is_some_and(|token| token.eq_ignore_ascii_case("FOR"))
    {
        (4usize, true)
    } else if statement
        .get(1)
        .is_some_and(|token| {
            token.eq_ignore_ascii_case("ADMIN") || token.eq_ignore_ascii_case("INHERIT")
        })
        && statement
            .get(2)
            .is_some_and(|token| token.eq_ignore_ascii_case("OPTION"))
        && statement
            .get(3)
            .is_some_and(|token| token.eq_ignore_ascii_case("FOR"))
    {
        return;
    } else {
        (1usize, false)
    };

    for role in membership_role_names(&statement[roles_start..from_index]) {
        if revoke_set_only {
            memberships.insert(role, false);
        } else {
            memberships.remove(&role);
        }
    }
}

/// Return whether `tepp_app_runtime` appears in the bounded grantee list.
fn runtime_is_grantee(statement: &[&str], delimiter: usize, stop_keywords: &[&str]) -> bool {
    let grantee_end = statement[delimiter + 1..]
        .iter()
        .position(|token| {
            stop_keywords
                .iter()
                .any(|keyword| token.eq_ignore_ascii_case(keyword))
        })
        .map_or(statement.len(), |relative| delimiter + 1 + relative);
    statement[delimiter + 1..grantee_end]
        .iter()
        .any(|token| token.eq_ignore_ascii_case(RUNTIME_ROLE))
}

/// Return normalized role names from a comma-separated membership role list.
fn membership_role_names(tokens: &[&str]) -> Vec<String> {
    tokens
        .iter()
        .filter(|token| **token != "," && !token.eq_ignore_ascii_case("GROUP"))
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

/// Return an explicitly stated SET membership option, or `None` when omitted.
///
/// PostgreSQL accepts `OPTION` as the true spelling. Malformed SET values map
/// to true so the validation boundary fails closed rather than treating an
/// unrecognized membership clause as a safety restoration.
fn explicit_set_option(statement: &[&str]) -> Option<bool> {
    let with_index = statement
        .iter()
        .position(|token| token.eq_ignore_ascii_case("WITH"))?;
    let mut index = with_index + 1;
    while index < statement.len() {
        if statement[index].eq_ignore_ascii_case("SET") {
            return Some(
                !statement
                    .get(index + 1)
                    .is_some_and(|value| value.eq_ignore_ascii_case("FALSE")),
            );
        }
        if statement[index].eq_ignore_ascii_case("GRANTED") {
            break;
        }
        index += 1;
    }
    None
}

/// Return whether canonicalized role creation adds the runtime `IN ROLE`.
///
/// `migration_validation` maps CREATE ROLE/USER/GROUP to CREATE TYPE for the
/// shared object-name parser while leaving role attributes in place. PostgreSQL
/// creates `IN ROLE` memberships with SET enabled, so the runtime cannot use
/// that creation shortcut under the RLS contract.
fn create_runtime_role_in_role(statement: &[&str]) -> bool {
    if statement.len() < 3
        || !statement[0].eq_ignore_ascii_case("CREATE")
        || !statement[1].eq_ignore_ascii_case("TYPE")
        || !statement[2].eq_ignore_ascii_case(RUNTIME_ROLE)
    {
        return false;
    }
    statement[3..].windows(2).any(|window| {
        window[0].eq_ignore_ascii_case("IN") && window[1].eq_ignore_ascii_case("ROLE")
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
    fn set_capable_runtime_memberships_fail_closed() {
        for sql in [
            "GRANT reporting_operator TO tepp_app_runtime ;",
            "GRANT reporting_operator TO tepp_app_runtime WITH SET TRUE ;",
            "GRANT reporting_operator TO tepp_app_runtime WITH SET OPTION ;",
            "CREATE TYPE tepp_app_runtime NOSUPERUSER NOBYPASSRLS IN ROLE reporting_operator ;",
        ] {
            assert!(!runtime_membership_is_rls_safe(sql), "{sql}");
        }
    }

    #[test]
    fn explicit_set_false_or_later_revoke_restores_membership_safety() {
        for sql in [
            "GRANT reporting_operator TO tepp_app_runtime WITH INHERIT TRUE , SET FALSE ;",
            "GRANT reporting_operator TO tepp_app_runtime ; REVOKE SET OPTION FOR reporting_operator FROM tepp_app_runtime ;",
            "GRANT reporting_operator TO tepp_app_runtime ; REVOKE reporting_operator FROM tepp_app_runtime ;",
            "GRANT reporting_operator TO tepp_app_runtime ; GRANT reporting_operator TO tepp_app_runtime WITH SET FALSE ;",
        ] {
            assert!(runtime_membership_is_rls_safe(sql), "{sql}");
        }
    }
}
