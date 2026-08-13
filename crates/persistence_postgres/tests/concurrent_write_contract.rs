//! Public concurrent-write classification and atomic revise contracts.

use persistence_postgres::{
    DEADLOCK_DETECTED_SQLSTATE, DocumentRecord, EXCLUSION_VIOLATION_SQLSTATE, PersistenceError,
    SERIALIZATION_FAILURE_SQLSTATE, UNIQUE_VIOLATION_SQLSTATE, classify_write_conflict,
    revise_document_atomic_sql,
};
use temporal_core::{AvailableTime, EventTime, SystemTime};

fn sample_record() -> DocumentRecord {
    DocumentRecord {
        document_record_id: uuid::Uuid::nil(),
        tenant_record_id: uuid::Uuid::nil(),
        content_digest: "ab".repeat(32),
        available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        valid_from: EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("v"),
        valid_to: None,
        system_from: SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("s"),
        system_to: None,
        revision_number: 2,
    }
}

#[test]
fn public_conflict_classifier_and_atomic_revise_sql_are_stable() {
    assert_eq!(UNIQUE_VIOLATION_SQLSTATE, "23505");
    assert_eq!(SERIALIZATION_FAILURE_SQLSTATE, "40001");
    assert_eq!(DEADLOCK_DETECTED_SQLSTATE, "40P01");
    assert_eq!(EXCLUSION_VIOLATION_SQLSTATE, "23P01");
    assert_eq!(
        classify_write_conflict(UNIQUE_VIOLATION_SQLSTATE),
        Some(PersistenceError::DuplicateDocumentRecord)
    );
    assert_eq!(
        classify_write_conflict(SERIALIZATION_FAILURE_SQLSTATE),
        Some(PersistenceError::ConcurrentWriteConflict)
    );

    let sql = revise_document_atomic_sql(&sample_record()).expect("atomic revise");
    assert!(sql.contains("DO $tepp$"));
    assert!(sql.contains("GET DIAGNOSTICS closed_count = ROW_COUNT"));
    assert!(sql.contains("closed_count <> 1"));
    assert!(sql.contains("ERRCODE = 'serialization_failure'"));
    assert!(sql.contains("UPDATE document_record"));
    assert!(sql.contains("INSERT INTO document_record"));
    assert!(sql.contains("system_to IS NULL"));

    let mut invalid = sample_record();
    invalid.content_digest = "short".into();
    assert_eq!(
        revise_document_atomic_sql(&invalid),
        Err(PersistenceError::InvalidContentDigest)
    );
}
