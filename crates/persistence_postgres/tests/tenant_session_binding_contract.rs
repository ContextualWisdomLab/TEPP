//! Tenant-session binding contracts for live persistence entrypoints.

use persistence_postgres::{
    LiveDocumentRepository, RecordingSqlSession, ReproducibilityManifestRecord,
};
use temporal_core::{AvailableTime, SystemTime};

#[test]
fn tenant_scoped_insert_binds_session_context() {
    let tenant_record_id = uuid::Uuid::from_u128(12);
    let manifest = ReproducibilityManifestRecord {
        reproducibility_manifest_id: uuid::Uuid::from_u128(11),
        tenant_record_id,
        knowledge_cutoff: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z")
            .expect("knowledge cutoff must parse"),
        evidence_digest: "ab".repeat(32),
        code_commit_sha: "c".repeat(40),
        dependency_lock_digest: "de".repeat(32),
        system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z")
            .expect("system time must parse"),
        available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z")
            .expect("available time must parse"),
    };
    let mut repository = LiveDocumentRepository::new(RecordingSqlSession::new());

    repository
        .insert_reproducibility_manifest(&manifest)
        .expect("tenant-scoped insert must succeed");

    let executed = repository.session().executed();
    assert_eq!(
        executed.len(),
        2,
        "tenant-scoped insert must bind the tenant before the row mutation"
    );
    assert!(executed[0].contains("tepp.current_tenant_record_id"));
    assert!(executed[0].contains(&tenant_record_id.to_string()));
    assert!(executed[1].contains("INSERT INTO reproducibility_manifest"));
}
