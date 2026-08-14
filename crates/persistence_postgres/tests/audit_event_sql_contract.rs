//! Audit-event SQL must refuse empty or hostile action codes before insert.

use persistence_postgres::{AuditEvent, PersistenceError, append_audit_sql};
use temporal_core::SystemTime;
use uuid::Uuid;

fn event() -> AuditEvent {
    AuditEvent {
        audit_event_id: Uuid::from_u128(1),
        tenant_record_id: Uuid::nil(),
        action_code: "revise".into(),
        subject_record_id: Uuid::from_u128(2),
        recorded_system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
    }
}

#[test]
fn insert_sql_renders_validated_action() {
    let sql = append_audit_sql(&event()).expect("sql");
    assert!(sql.contains("INSERT INTO audit_event"));
    assert!(sql.contains("action_code"));
    assert!(sql.contains("revise"));
    assert!(sql.contains("recorded_system_time"));
}

#[test]
fn empty_and_hostile_action_codes_fail_closed() {
    let mut empty = event();
    empty.action_code.clear();
    assert_eq!(
        append_audit_sql(&empty),
        Err(PersistenceError::InvalidAuditEvent)
    );

    let mut hostile = event();
    hostile.action_code = "revise'; DROP TABLE".into();
    assert_eq!(
        append_audit_sql(&hostile),
        Err(PersistenceError::InvalidAuditEvent)
    );
    hostile.action_code = "revise;role".into();
    assert_eq!(
        append_audit_sql(&hostile),
        Err(PersistenceError::InvalidAuditEvent)
    );
    hostile.action_code = "revise\\".into();
    assert_eq!(
        append_audit_sql(&hostile),
        Err(PersistenceError::InvalidAuditEvent)
    );
    hostile.action_code = "revise\nrole".into();
    assert_eq!(
        append_audit_sql(&hostile),
        Err(PersistenceError::InvalidAuditEvent)
    );
    hostile.action_code = "x".repeat(129);
    assert_eq!(
        append_audit_sql(&hostile),
        Err(PersistenceError::InvalidAuditEvent)
    );
}
