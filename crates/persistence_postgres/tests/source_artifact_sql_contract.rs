//! Source-artifact SQL must refuse invalid digests, sizes, and hostile labels.

use persistence_postgres::{
    PersistenceError, SourceArtifactRecord, assert_source_artifact_matches_sql,
    insert_source_artifact_sql, select_source_artifact_by_id_sql,
    source_artifacts_are_idempotent_matches,
};
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

fn clocks() -> (AvailableTime, SystemTime) {
    (
        AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
        SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
    )
}

fn artifact() -> SourceArtifactRecord {
    let (available, system) = clocks();
    SourceArtifactRecord {
        source_artifact_id: Uuid::from_u128(1),
        tenant_record_id: Uuid::nil(),
        content_sha256: "ab".repeat(32),
        source_size_bytes: 4,
        media_type_code: "text/plain".into(),
        protected_object_ref: None,
        system_time: system,
        available_time: available,
    }
}

#[test]
fn insert_sql_renders_open_object_ref_as_null() {
    let sql = insert_source_artifact_sql(&artifact()).expect("sql");
    assert!(sql.contains("INSERT INTO source_artifact"));
    assert!(sql.contains("content_sha256"));
    assert!(sql.contains("source_size_bytes"));
    assert!(sql.contains("media_type_code"));
    assert!(sql.contains("NULL"));
}

#[test]
fn insert_sql_renders_protected_object_ref() {
    let mut with_ref = artifact();
    with_ref.protected_object_ref = Some("s3://tepp/evidence/object".into());
    with_ref.source_size_bytes = 0;
    let sql = insert_source_artifact_sql(&with_ref).expect("sql");
    assert!(sql.contains("s3://tepp/evidence/object"));
    assert!(!sql.contains("NULL"));
}

#[test]
fn invalid_digest_size_and_hostile_labels_fail_closed() {
    let mut short = artifact();
    short.content_sha256 = "ab".into();
    assert_eq!(
        insert_source_artifact_sql(&short),
        Err(PersistenceError::InvalidSourceArtifact)
    );

    let mut uppercase = artifact();
    uppercase.content_sha256 = "AB".repeat(32);
    assert_eq!(
        insert_source_artifact_sql(&uppercase),
        Err(PersistenceError::InvalidSourceArtifact)
    );

    let mut negative = artifact();
    negative.source_size_bytes = -1;
    assert_eq!(
        insert_source_artifact_sql(&negative),
        Err(PersistenceError::InvalidSourceArtifact)
    );

    let mut empty_type = artifact();
    empty_type.media_type_code.clear();
    assert_eq!(
        insert_source_artifact_sql(&empty_type),
        Err(PersistenceError::InvalidSourceArtifact)
    );

    let mut hostile = artifact();
    hostile.media_type_code = "text/plain'; DROP TABLE".into();
    assert_eq!(
        insert_source_artifact_sql(&hostile),
        Err(PersistenceError::InvalidSourceArtifact)
    );
    hostile.media_type_code = "text/plain;role".into();
    assert_eq!(
        insert_source_artifact_sql(&hostile),
        Err(PersistenceError::InvalidSourceArtifact)
    );
    hostile.media_type_code = "text/plain\\".into();
    assert_eq!(
        insert_source_artifact_sql(&hostile),
        Err(PersistenceError::InvalidSourceArtifact)
    );
    hostile.media_type_code = "text/plain\nhtml".into();
    assert_eq!(
        insert_source_artifact_sql(&hostile),
        Err(PersistenceError::InvalidSourceArtifact)
    );
    hostile.media_type_code = "x".repeat(129);
    assert_eq!(
        insert_source_artifact_sql(&hostile),
        Err(PersistenceError::InvalidSourceArtifact)
    );

    let mut empty_ref = artifact();
    empty_ref.protected_object_ref = Some(String::new());
    assert_eq!(
        insert_source_artifact_sql(&empty_ref),
        Err(PersistenceError::InvalidSourceArtifact)
    );
    empty_ref.protected_object_ref = Some("obj'; DROP".into());
    assert_eq!(
        insert_source_artifact_sql(&empty_ref),
        Err(PersistenceError::InvalidSourceArtifact)
    );
}

#[test]
fn lookup_renders_primary_key_selection() {
    let lookup = select_source_artifact_by_id_sql(Uuid::from_u128(1));
    assert!(lookup.contains("FROM source_artifact"));
    assert!(lookup.contains("source_artifact_id"));
}

#[test]
fn insert_sql_is_idempotent_on_primary_key() {
    let sql = insert_source_artifact_sql(&artifact()).expect("sql");
    assert!(sql.contains("ON CONFLICT (source_artifact_id) DO NOTHING"));
}

#[test]
fn identical_records_are_idempotent_matches() {
    let first = artifact();
    let retry = artifact();
    assert!(source_artifacts_are_idempotent_matches(&first, &retry));
}

#[test]
fn divergent_identity_fields_are_not_idempotent_matches() {
    let first = artifact();
    let mut other = artifact();
    other.tenant_record_id = Uuid::from_u128(2);
    assert!(!source_artifacts_are_idempotent_matches(&first, &other));
    other = artifact();
    other.content_sha256 = "cd".repeat(32);
    assert!(!source_artifacts_are_idempotent_matches(&first, &other));
    other = artifact();
    other.source_size_bytes = 8;
    assert!(!source_artifacts_are_idempotent_matches(&first, &other));
    other = artifact();
    other.media_type_code = "text/csv".into();
    assert!(!source_artifacts_are_idempotent_matches(&first, &other));
    other = artifact();
    other.protected_object_ref = Some("s3://tepp/other".into());
    assert!(!source_artifacts_are_idempotent_matches(&first, &other));
    other = artifact();
    other.system_time = SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("s");
    assert!(!source_artifacts_are_idempotent_matches(&first, &other));
    other = artifact();
    other.available_time = AvailableTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("a");
    assert!(!source_artifacts_are_idempotent_matches(&first, &other));
}

#[test]
fn assert_sql_requires_every_stored_field_to_match() {
    let sql = assert_source_artifact_matches_sql(&artifact()).expect("assert");
    assert!(sql.contains("conflicting source artifact"));
    assert!(sql.contains("source_artifact_id"));
    assert!(sql.contains("tenant_record_id"));
    assert!(sql.contains("content_sha256"));
    assert!(sql.contains("source_size_bytes"));
    assert!(sql.contains("media_type_code"));
    assert!(sql.contains("protected_object_ref IS NOT DISTINCT FROM"));
    assert!(sql.contains("system_time"));
    assert!(sql.contains("available_time"));
}
