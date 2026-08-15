//! Retention policy and legal-hold windows must fail closed at every boundary.

use persistence_postgres::{
    LegalHoldRecord, PersistenceError, RetentionPolicyRecord, insert_legal_hold_sql,
    insert_retention_policy_sql,
};
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

fn system_time(value: &str) -> SystemTime {
    SystemTime::parse_rfc3339(value).expect("valid system time fixture")
}

fn available_time() -> AvailableTime {
    AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("valid available time fixture")
}

fn active_policy() -> RetentionPolicyRecord {
    RetentionPolicyRecord {
        retention_policy_id: Uuid::from_u128(1),
        tenant_record_id: Uuid::from_u128(2),
        data_class_code: "raw_source".into(),
        processing_purpose_code: "psychometric_analysis".into(),
        retention_period_days: 365,
        policy_status_code: "active".into(),
        authority_citation: "adr-0009".into(),
        system_time: system_time("2026-01-01T00:00:00Z"),
        system_to: None,
        available_time: available_time(),
    }
}

fn active_document_hold() -> LegalHoldRecord {
    LegalHoldRecord {
        legal_hold_id: Uuid::from_u128(3),
        tenant_record_id: Uuid::from_u128(2),
        hold_scope_code: "document".into(),
        held_document_id: Some(Uuid::from_u128(4)),
        hold_authority_code: "contract".into(),
        hold_status_code: "active".into(),
        authority_citation: "hold-authority".into(),
        system_time: system_time("2026-01-01T00:00:00Z"),
        system_to: None,
        available_time: available_time(),
    }
}

#[test]
fn retention_policy_requires_status_window_consistency_and_forward_closure() {
    let mut active_with_close = active_policy();
    active_with_close.system_to = Some(system_time("2026-02-01T00:00:00Z"));
    assert_eq!(
        insert_retention_policy_sql(&active_with_close),
        Err(PersistenceError::InvalidRetentionLifecycle)
    );

    let mut superseded_without_close = active_policy();
    superseded_without_close.policy_status_code = "superseded".into();
    assert_eq!(
        insert_retention_policy_sql(&superseded_without_close),
        Err(PersistenceError::InvalidRetentionLifecycle)
    );

    let mut superseded_with_reverse_close = superseded_without_close.clone();
    superseded_with_reverse_close.system_to = Some(system_time("2025-12-31T23:59:59Z"));
    assert_eq!(
        insert_retention_policy_sql(&superseded_with_reverse_close),
        Err(PersistenceError::InvalidRetentionLifecycle)
    );

    let mut superseded_with_forward_close = superseded_without_close;
    superseded_with_forward_close.system_to = Some(system_time("2026-02-01T00:00:00Z"));
    insert_retention_policy_sql(&superseded_with_forward_close)
        .expect("forward closed superseded policy");
}

#[test]
fn legal_hold_requires_status_window_consistency_and_forward_closure() {
    let mut active_with_close = active_document_hold();
    active_with_close.system_to = Some(system_time("2026-02-01T00:00:00Z"));
    assert_eq!(
        insert_legal_hold_sql(&active_with_close),
        Err(PersistenceError::InvalidRetentionLifecycle)
    );
    assert!(
        !active_with_close.blocks_deletion(
            active_with_close.tenant_record_id,
            active_with_close
                .held_document_id
                .expect("held document fixture")
        ),
        "a closed hold cannot block deletion even when its status is malformed as active"
    );

    let mut released_without_close = active_document_hold();
    released_without_close.hold_status_code = "released".into();
    assert_eq!(
        insert_legal_hold_sql(&released_without_close),
        Err(PersistenceError::InvalidRetentionLifecycle)
    );

    let mut released_with_reverse_close = released_without_close.clone();
    released_with_reverse_close.system_to = Some(system_time("2025-12-31T23:59:59Z"));
    assert_eq!(
        insert_legal_hold_sql(&released_with_reverse_close),
        Err(PersistenceError::InvalidRetentionLifecycle)
    );

    let mut released_with_forward_close = released_without_close;
    released_with_forward_close.system_to = Some(system_time("2026-02-01T00:00:00Z"));
    insert_legal_hold_sql(&released_with_forward_close).expect("forward closed released hold");
}
