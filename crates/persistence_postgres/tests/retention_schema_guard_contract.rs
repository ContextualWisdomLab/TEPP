//! Database-layer vocabulary and tombstone/request consistency contracts.
//!
//! Application validation is not a substitute for `PostgreSQL` constraints. The
//! lifecycle migration must fail closed when callers bypass Rust renderers or
//! attempt to connect a tombstone to a different tenant, document, data class,
//! deletion kind, or incomplete deletion request.

const RETENTION_UP_SQL: &str =
    include_str!("../../../migrations/0007_retention_deletion_legal_hold.up.sql");
const RETENTION_DOWN_SQL: &str =
    include_str!("../../../migrations/0007_retention_deletion_legal_hold.down.sql");

#[test]
fn lifecycle_code_columns_have_database_allowlist_constraints() {
    for constraint in [
        "CONSTRAINT deletion_request_kind_known",
        "CONSTRAINT deletion_request_status_known",
        "CONSTRAINT evidence_tombstone_kind_known",
        "CONSTRAINT evidence_tombstone_reproduction_status_known",
    ] {
        assert!(
            RETENTION_UP_SQL.contains(constraint),
            "missing database allowlist constraint: {constraint}"
        );
    }

    for canonical_value in [
        "'logical_revocation'",
        "'cache_export_removal'",
        "'identity_tombstone'",
        "'requested'",
        "'completed'",
        "'blocked_by_hold'",
        "'reproduction_limited'",
        "'unavailable'",
        "'limited'",
        "'unaffected'",
    ] {
        assert!(
            RETENTION_UP_SQL.contains(canonical_value),
            "database vocabulary must include canonical value {canonical_value}"
        );
    }
}

#[test]
fn tombstone_insert_is_bound_to_its_completed_deletion_request() {
    for required_sql in [
        "CREATE OR REPLACE FUNCTION guard_evidence_tombstone_insert",
        "CREATE TRIGGER evidence_tombstone_guard_insert",
        "BEFORE INSERT ON evidence_tombstone",
        "deletion_request_id = NEW.deletion_request_id",
        "tenant_record_id = NEW.tenant_record_id",
        "target_document_id = NEW.tombstoned_document_id",
        "target_data_class_code = NEW.target_data_class_code",
        "deletion_kind_code = NEW.deletion_kind_code",
        "request_status_code = 'completed'",
        "tepp.current_tenant_record_id",
        "SECURITY DEFINER",
        "SET search_path = public, pg_temp",
    ] {
        assert!(
            RETENTION_UP_SQL.contains(required_sql),
            "missing fail-closed tombstone/request guard: {required_sql}"
        );
    }
}

#[test]
fn rollback_removes_tombstone_guard_before_its_function() {
    let trigger = RETENTION_DOWN_SQL
        .find("DROP TRIGGER IF EXISTS evidence_tombstone_guard_insert")
        .expect("tombstone guard trigger cleanup");
    let function = RETENTION_DOWN_SQL
        .find("DROP FUNCTION IF EXISTS guard_evidence_tombstone_insert")
        .expect("tombstone guard function cleanup");
    assert!(
        trigger < function,
        "rollback must drop the dependent tombstone trigger before its function"
    );
}
