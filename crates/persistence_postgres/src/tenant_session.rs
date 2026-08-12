//! Session tenant context for `PostgreSQL` row-level security.
//!
//! Application connections set GUC `tepp.current_tenant_record_id` to the active
//! tenant UUID text and assume role `tepp_app_runtime` so policies apply.

use uuid::Uuid;

/// `PostgreSQL` custom GUC used by tenant isolation policies.
pub const TENANT_SESSION_GUC: &str = "tepp.current_tenant_record_id";

/// Non-superuser application role created by the RLS migration.
pub const APP_RUNTIME_ROLE: &str = "tepp_app_runtime";

/// Render SQL that assumes the least-privilege application runtime role.
#[must_use]
pub fn assume_app_runtime_role_sql() -> String {
    format!("SET ROLE {APP_RUNTIME_ROLE}")
}

/// Render SQL that restores the login role after application work.
#[must_use]
pub fn reset_app_runtime_role_sql() -> String {
    "RESET ROLE".to_owned()
}

/// Render SQL that binds the session tenant GUC for RLS policies.
///
/// The third `set_config` argument is `false` so the value persists for the
/// session (not only the current transaction), matching long-lived pool
/// connections that re-bind tenant context per request.
#[must_use]
pub fn set_session_tenant_sql(tenant_record_id: Uuid) -> String {
    format!("SELECT set_config('{TENANT_SESSION_GUC}', '{tenant_record_id}', false)")
}

/// Render SQL that clears the session tenant GUC (fail-closed empty policies).
#[must_use]
pub fn clear_session_tenant_sql() -> String {
    format!("SELECT set_config('{TENANT_SESSION_GUC}', '', false)")
}

#[cfg(test)]
mod tests {
    use super::{
        APP_RUNTIME_ROLE, TENANT_SESSION_GUC, assume_app_runtime_role_sql,
        clear_session_tenant_sql, reset_app_runtime_role_sql, set_session_tenant_sql,
    };

    #[test]
    fn tenant_session_sql_binds_guc_and_role() {
        let tenant = uuid::Uuid::nil();
        let set_sql = set_session_tenant_sql(tenant);
        assert!(set_sql.contains(TENANT_SESSION_GUC));
        assert!(set_sql.contains(&tenant.to_string()));
        assert!(set_sql.contains("set_config"));
        assert!(set_sql.contains("false"));

        let clear = clear_session_tenant_sql();
        assert!(clear.contains(TENANT_SESSION_GUC));
        assert!(clear.contains("''"));

        let assume = assume_app_runtime_role_sql();
        assert!(assume.contains(APP_RUNTIME_ROLE));
        assert!(assume.starts_with("SET ROLE"));

        assert_eq!(reset_app_runtime_role_sql(), "RESET ROLE");
    }
}
