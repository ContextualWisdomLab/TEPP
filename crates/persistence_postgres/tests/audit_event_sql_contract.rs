//! Audit-event SQL must inspect source payloads through `try_record`.

use persistence_postgres::{
    ACTION_AUDIT_EVENT_APPEND, AuditEvent, AuditSourceInspection, PersistenceError,
    append_audit_sql,
};
use temporal_core::SystemTime;
use uuid::Uuid;

/// Author-owned customer renewal text that also names a partner review.
const RENEWAL_SOURCE_TEXT: &str = "Kim Park (kim.park@acme.example) escalated the ACME Corp Q2 renewal after the partner legal review stalled.";
const AUTHOR_SOURCE_IDENTITY: &str = "kim.park@acme.example";

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
fn insert_sql_renders_validated_action_after_clear_inspection() {
    let sql = append_audit_sql(&event(), AuditSourceInspection::CLEAR).expect("sql");
    assert!(sql.contains("INSERT INTO audit_event"));
    assert!(sql.contains("action_code"));
    assert!(sql.contains("revise"));
    assert!(sql.contains("recorded_system_time"));
    assert!(!sql.contains(RENEWAL_SOURCE_TEXT));
    assert!(!sql.contains(AUTHOR_SOURCE_IDENTITY));
    assert_eq!(ACTION_AUDIT_EVENT_APPEND, 2_001);
    assert_eq!(
        AuditSourceInspection::default(),
        AuditSourceInspection::CLEAR
    );
}

#[test]
fn renewal_source_text_and_identity_cannot_enter_audit_insert_sql() {
    let event = event();
    assert_eq!(
        append_audit_sql(
            &event,
            AuditSourceInspection {
                source_text: Some(RENEWAL_SOURCE_TEXT),
                source_identity: None,
                blanket_mask: false,
            },
        ),
        Err(PersistenceError::SourceTextNotAuditable)
    );
    assert_eq!(
        append_audit_sql(
            &event,
            AuditSourceInspection {
                source_text: None,
                source_identity: Some(AUTHOR_SOURCE_IDENTITY),
                blanket_mask: false,
            },
        ),
        Err(PersistenceError::SourceIdentityNotAuditable)
    );
    assert_eq!(
        append_audit_sql(
            &event,
            AuditSourceInspection {
                source_text: None,
                source_identity: None,
                blanket_mask: true,
            },
        ),
        Err(PersistenceError::BlanketMaskIsNotAuditAuthorization)
    );
    assert_eq!(
        append_audit_sql(
            &event,
            AuditSourceInspection {
                source_text: Some(RENEWAL_SOURCE_TEXT),
                source_identity: Some(AUTHOR_SOURCE_IDENTITY),
                blanket_mask: true,
            },
        ),
        Err(PersistenceError::SourceTextNotAuditable)
    );
}

#[test]
fn empty_and_hostile_action_codes_fail_closed() {
    let mut empty = event();
    empty.action_code.clear();
    assert_eq!(
        append_audit_sql(&empty, AuditSourceInspection::CLEAR),
        Err(PersistenceError::InvalidAuditEvent)
    );

    let mut hostile = event();
    hostile.action_code = "revise'; DROP TABLE".into();
    assert_eq!(
        append_audit_sql(&hostile, AuditSourceInspection::CLEAR),
        Err(PersistenceError::InvalidAuditEvent)
    );
    hostile.action_code = "revise;role".into();
    assert_eq!(
        append_audit_sql(&hostile, AuditSourceInspection::CLEAR),
        Err(PersistenceError::InvalidAuditEvent)
    );
    hostile.action_code = "revise\\".into();
    assert_eq!(
        append_audit_sql(&hostile, AuditSourceInspection::CLEAR),
        Err(PersistenceError::InvalidAuditEvent)
    );
    hostile.action_code = "revise\nrole".into();
    assert_eq!(
        append_audit_sql(&hostile, AuditSourceInspection::CLEAR),
        Err(PersistenceError::InvalidAuditEvent)
    );
    hostile.action_code = "x".repeat(129);
    assert_eq!(
        append_audit_sql(&hostile, AuditSourceInspection::CLEAR),
        Err(PersistenceError::InvalidAuditEvent)
    );
}
