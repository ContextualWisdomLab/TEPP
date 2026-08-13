//! Restored rows are untrusted until integrity revalidation (ADR 0013).

use persistence_postgres::{
    PersistenceError, RestoredAnalyticalSnapshot, backup_scope_tables, mark_restored_state_usable,
    restore_integrity_probe_sqls,
};
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff};
use uuid::Uuid;

fn valid_snapshot() -> RestoredAnalyticalSnapshot {
    RestoredAnalyticalSnapshot {
        tenant_record_id: Some(Uuid::nil()),
        content_sha256: "ab".repeat(32),
        available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        knowledge_cutoff: KnowledgeCutoff::parse_rfc3339("2026-06-01T00:00:00Z").expect("k"),
        valid_from: EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("vf"),
        valid_to: None,
        append_only_triggers_present: true,
    }
}

#[test]
fn valid_restored_snapshot_may_be_marked_usable() {
    let usable = mark_restored_state_usable(&valid_snapshot()).expect("usable");
    assert!(usable.is_usable());
}

#[test]
fn restore_integrity_fails_closed_on_missing_or_hostile_fields() {
    let mut missing_tenant = valid_snapshot();
    missing_tenant.tenant_record_id = None;
    assert_eq!(
        mark_restored_state_usable(&missing_tenant),
        Err(PersistenceError::RestoreIntegrityFailed)
    );

    let mut bad_digest = valid_snapshot();
    bad_digest.content_sha256 = "NOPE".into();
    assert_eq!(
        mark_restored_state_usable(&bad_digest),
        Err(PersistenceError::RestoreIntegrityFailed)
    );

    let mut future_available = valid_snapshot();
    future_available.available_time =
        AvailableTime::parse_rfc3339("2026-12-01T00:00:00Z").expect("later");
    assert_eq!(
        mark_restored_state_usable(&future_available),
        Err(PersistenceError::RestoreIntegrityFailed)
    );

    let mut inverted = valid_snapshot();
    inverted.valid_to = Some(EventTime::parse_rfc3339("2025-01-01T00:00:00Z").expect("earlier"));
    assert_eq!(
        mark_restored_state_usable(&inverted),
        Err(PersistenceError::RestoreIntegrityFailed)
    );

    let mut no_triggers = valid_snapshot();
    no_triggers.append_only_triggers_present = false;
    assert_eq!(
        mark_restored_state_usable(&no_triggers),
        Err(PersistenceError::RestoreIntegrityFailed)
    );
}

#[test]
fn restore_probe_sql_covers_digest_cutoff_window_and_triggers() {
    let probes = restore_integrity_probe_sqls();
    let joined = probes.join("\n");
    assert!(joined.contains("content_sha256"));
    assert!(joined.contains("^[0-9a-f]{64}$"));
    assert!(joined.contains("available_time"));
    assert!(joined.contains("valid_from"));
    assert!(joined.contains("valid_to"));
    assert!(joined.contains("reject_append_only_mutation"));
    assert!(joined.contains("restore integrity failed"));
    assert!(joined.contains("missing_manifest"));
    assert!(joined.contains("rm.tenant_record_id = sa.tenant_record_id"));
    assert!(joined.contains("pg_trigger"));
    assert!(joined.contains("t.tgenabled"));
    assert!(joined.contains("'source_artifact'"));
    assert!(joined.contains("'model_artifact'"));
    assert!(backup_scope_tables().contains(&"source_artifact"));
    assert!(backup_scope_tables().contains(&"reproducibility_manifest"));
    assert!(backup_scope_tables().contains(&"document_record"));
}
