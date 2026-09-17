//! Fail-closed PostgreSQL role-membership boundary for the application runtime.
//!
//! Direct role attributes and lifecycle remain owned by `migration_validation`.
//! This module owns the complementary membership invariant: an RLS-protected
//! application runtime must not be able to become another role with `SET ROLE`.
//! PostgreSQL makes `SET` membership transitive and enables it by default, so a
//! runtime with any SET-capable membership can escape the direct
//! `NOSUPERUSER NOBYPASSRLS` proof if the target role is or later becomes
//! privileged.

const RUNTIME_ROLE: &str = "tepp_app_runtime";

/// Return whether the normalized migration keeps the runtime free of SET-capable memberships.
///
/// Object-privilege grants are excluded by the structural `ON` token before
/// `TO`. Role-membership grants to the runtime are accepted only when they
/// explicitly say `WITH SET FALSE`; PostgreSQL defaults a new membership's SET
/// option to true. `CREATE ROLE ... IN ROLE ...` is represented as `CREATE TYPE`
/// by the existing structural alias canonicalizer, so the same normalized copy
/// is checked for that SET-enabled creation form as well.
pub(super) fn runtime_membership_is_rls_safe(sql: &str) -> bool {
    let tokenized = sql.replace(';', " ; ").replace(',', " , ");
    let tokens = tokenized.split_whitespace().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < tokens.len() {
        let end = statement_end(&tokens, index);
        if tokens[index].eq_ignore_ascii_case("GRANT")
            && grant_gives_runtime_set_role(&tokens[index..end])
        {
            return false;
        }
        if tokens[index].eq_ignore_ascii_case("CREATE")
            && create_runtime_role_in_role(&tokens[index..end])
        {
            return false;
        }
        index = end.saturating_add(1);
    }
    true
}

/// Return the exclusive end of the semicolon-delimited statement containing `start`.
fn statement_end(tokens: &[&str], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| *token == ";")
        .map_or(tokens.len(), |relative| start + relative)
}

/// Return whether a normalized GRANT gives `tepp_app_runtime` SET ROLE authority.
///
/// PostgreSQL object grants contain `ON` before the grantee `TO`; role
/// membership grants do not. For a new membership SET defaults to true, so the
/// only admitted runtime membership is one that explicitly fixes SET to false.
fn grant_gives_runtime_set_role(statement: &[&str]) -> bool {
    let Some(to_index) = statement
        .iter()
        .position(|token| token.eq_ignore_ascii_case("TO"))
    else {
        return false;
    };
    if statement[..to_index]
        .iter()
        .any(|token| token.eq_ignore_ascii_case("ON"))
    {
        return false;
    }

    let grantee_end = statement[to_index + 1..]
        .iter()
        .position(|token| {
            token.eq_ignore_ascii_case("WITH") || token.eq_ignore_ascii_case("GRANTED")
        })
        .map_or(statement.len(), |relative| to_index + 1 + relative);
    let runtime_is_grantee = statement[to_index + 1..grantee_end]
        .iter()
        .any(|token| token.eq_ignore_ascii_case(RUNTIME_ROLE));
    if !runtime_is_grantee {
        return false;
    }

    !membership_explicitly_disables_set(statement)
}

/// Return whether the role-membership options contain exactly a `SET FALSE` refusal.
///
/// `SET OPTION` is PostgreSQL's spelling for `SET TRUE`. Missing SET is also
/// unsafe because SET defaults to true when the membership is created. Any
/// malformed or contradictory SET sequence therefore fails closed.
fn membership_explicitly_disables_set(statement: &[&str]) -> bool {
    let mut saw_false = false;
    let mut index = 0usize;
    while index < statement.len() {
        if statement[index].eq_ignore_ascii_case("SET") {
            let Some(value) = statement.get(index + 1) else {
                return false;
            };
            if value.eq_ignore_ascii_case("FALSE") {
                saw_false = true;
            } else {
                return false;
            }
            index += 2;
            continue;
        }
        index += 1;
    }
    saw_false
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
    fn explicit_set_false_membership_is_not_a_set_role_path() {
        assert!(runtime_membership_is_rls_safe(
            "GRANT reporting_operator TO tepp_app_runtime WITH INHERIT TRUE , SET FALSE ;"
        ));
    }
}
