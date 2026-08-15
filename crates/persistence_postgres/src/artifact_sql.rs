//! SQL contracts for append-only source artifacts (ADR 0008 / ADR 0013).

use crate::PersistenceError;
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

/// One append-only source artifact independent of document identity.
///
/// Maps to `source_artifact`. The identity is never the content digest:
/// identical bytes may be acquired in different tenant or provenance
/// contexts. Digests are lowercase hex `SHA-256`. Size must be non-negative.
/// Media type and optional object-store references are fail-closed labels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceArtifactRecord {
    /// Artifact identity (independent of the content digest).
    pub source_artifact_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Canonical `SHA-256` of the immutable source bytes.
    pub content_sha256: String,
    /// Declared payload size in bytes; must be `>= 0`.
    pub source_size_bytes: i64,
    /// Media type token (for example `text/plain`).
    pub media_type_code: String,
    /// Optional protected object-store reference.
    pub protected_object_ref: Option<String>,
    /// System/record time when the artifact identity was asserted.
    pub system_time: SystemTime,
    /// Availability time of the artifact evidence.
    pub available_time: AvailableTime,
}

impl SourceArtifactRecord {
    /// Fail-closed digest, size, and label validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidSourceArtifact`] when the digest is
    /// not canonical lowercase hex `SHA-256`, the size is negative, or a
    /// label is empty, oversized, or hostile.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_sha256_hex(&self.content_sha256)?;
        if self.source_size_bytes < 0 {
            return Err(PersistenceError::InvalidSourceArtifact);
        }
        validate_artifact_label(&self.media_type_code)?;
        if let Some(object_ref) = &self.protected_object_ref {
            validate_artifact_label(object_ref)?;
        }
        Ok(())
    }
}

/// Render insert SQL for a validated source artifact.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidSourceArtifact`] before any SQL is produced.
pub fn insert_source_artifact_sql(
    record: &SourceArtifactRecord,
) -> Result<String, PersistenceError> {
    record.validate()?;
    let object_ref_sql = match &record.protected_object_ref {
        Some(value) => format!("'{value}'"),
        None => "NULL".to_owned(),
    };
    Ok(format!(
        "INSERT INTO source_artifact (\
            source_artifact_id, tenant_record_id, content_sha256, source_size_bytes, \
            media_type_code, protected_object_ref, system_time, available_time\
        ) VALUES (\
            '{artifact}'::uuid, '{tenant}'::uuid, '{digest}', {size}, \
            '{media}', {object_ref_sql}, '{system}'::timestamptz, '{available}'::timestamptz\
        ) ON CONFLICT (source_artifact_id) DO NOTHING",
        artifact = record.source_artifact_id,
        tenant = record.tenant_record_id,
        digest = record.content_sha256,
        size = record.source_size_bytes,
        media = record.media_type_code,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Compare two validated artifacts for ADR 0013 idempotent-retry equality.
#[must_use]
pub fn source_artifacts_are_idempotent_matches(
    left: &SourceArtifactRecord,
    right: &SourceArtifactRecord,
) -> bool {
    left == right
}

/// Render a fail-closed assertion that the stored row matches `record`.
///
/// Used after `INSERT ... ON CONFLICT DO NOTHING` so a retry of the same
/// immutable identity succeeds and a same-id payload change raises
/// `conflicting source artifact`.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidSourceArtifact`] before any SQL is produced.
pub fn assert_source_artifact_matches_sql(
    record: &SourceArtifactRecord,
) -> Result<String, PersistenceError> {
    record.validate()?;
    let object_ref_sql = match &record.protected_object_ref {
        Some(value) => format!("'{value}'"),
        None => "NULL".to_owned(),
    };
    Ok(format!(
        "DO $tepp_source_artifact_idempotent$\n\
         BEGIN\n\
           IF NOT EXISTS (\n\
             SELECT 1 FROM source_artifact\n\
             WHERE source_artifact_id = '{artifact}'::uuid\n\
               AND tenant_record_id = '{tenant}'::uuid\n\
               AND content_sha256 = '{digest}'\n\
               AND source_size_bytes = {size}\n\
               AND media_type_code = '{media}'\n\
               AND protected_object_ref IS NOT DISTINCT FROM {object_ref_sql}\n\
               AND system_time = '{system}'::timestamptz\n\
               AND available_time = '{available}'::timestamptz\n\
           ) THEN\n\
             RAISE EXCEPTION 'conflicting source artifact';\n\
           END IF;\n\
         END\n\
         $tepp_source_artifact_idempotent$",
        artifact = record.source_artifact_id,
        tenant = record.tenant_record_id,
        digest = record.content_sha256,
        size = record.source_size_bytes,
        media = record.media_type_code,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render selection of a source artifact by primary key.
#[must_use]
pub fn select_source_artifact_by_id_sql(source_artifact_id: Uuid) -> String {
    format!(
        "SELECT source_artifact_id, tenant_record_id, content_sha256, source_size_bytes, \
                media_type_code, protected_object_ref, system_time, available_time \
         FROM source_artifact \
         WHERE source_artifact_id = '{source_artifact_id}'::uuid \
         LIMIT 1"
    )
}

fn is_lowercase_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn validate_sha256_hex(value: &str) -> Result<(), PersistenceError> {
    if value.len() != 64 || !is_lowercase_hex(value) {
        return Err(PersistenceError::InvalidSourceArtifact);
    }
    Ok(())
}

fn validate_artifact_label(value: &str) -> Result<(), PersistenceError> {
    if value.is_empty()
        || value.len() > 128
        || value
            .chars()
            .any(|ch| ch.is_control() || ch == '\'' || ch == ';' || ch == '\\')
    {
        return Err(PersistenceError::InvalidSourceArtifact);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        SourceArtifactRecord, assert_source_artifact_matches_sql, insert_source_artifact_sql,
        is_lowercase_hex, select_source_artifact_by_id_sql,
        source_artifacts_are_idempotent_matches, validate_artifact_label, validate_sha256_hex,
    };
    use crate::PersistenceError;
    use temporal_core::{AvailableTime, SystemTime};
    use uuid::Uuid;

    fn sample() -> SourceArtifactRecord {
        SourceArtifactRecord {
            source_artifact_id: Uuid::nil(),
            tenant_record_id: Uuid::nil(),
            content_sha256: "ab".repeat(32),
            source_size_bytes: 4,
            media_type_code: "text/plain".into(),
            protected_object_ref: None,
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    #[test]
    fn artifact_sql_covers_valid_and_fail_closed_paths() {
        let insert = insert_source_artifact_sql(&sample()).expect("insert");
        assert!(insert.contains("INSERT INTO source_artifact"));
        assert!(insert.contains("ON CONFLICT (source_artifact_id) DO NOTHING"));
        assert!(insert.contains("NULL"));
        assert!(source_artifacts_are_idempotent_matches(
            &sample(),
            &sample()
        ));
        let assertion = assert_source_artifact_matches_sql(&sample()).expect("assert");
        assert!(assertion.contains("conflicting source artifact"));
        assert_eq!(
            assert_source_artifact_matches_sql(&SourceArtifactRecord {
                source_size_bytes: -1,
                ..sample()
            }),
            Err(PersistenceError::InvalidSourceArtifact)
        );

        let mut with_ref = sample();
        with_ref.protected_object_ref = Some("s3://tepp/object".into());
        with_ref.source_size_bytes = 0;
        let referenced = insert_source_artifact_sql(&with_ref).expect("ref");
        assert!(referenced.contains("s3://tepp/object"));
        let referenced_assertion =
            assert_source_artifact_matches_sql(&with_ref).expect("referenced assertion");
        assert!(referenced_assertion.contains("s3://tepp/object"));
        assert!(referenced_assertion.contains("IS NOT DISTINCT FROM 's3://tepp/object'"));

        assert_eq!(
            insert_source_artifact_sql(&SourceArtifactRecord {
                content_sha256: "nope".into(),
                ..sample()
            }),
            Err(PersistenceError::InvalidSourceArtifact)
        );
        assert_eq!(
            insert_source_artifact_sql(&SourceArtifactRecord {
                source_size_bytes: -1,
                ..sample()
            }),
            Err(PersistenceError::InvalidSourceArtifact)
        );
        assert!(validate_sha256_hex(&"a1".repeat(32)).is_ok());
        assert!(validate_sha256_hex("x").is_err());
        assert!(validate_sha256_hex(&"AB".repeat(32)).is_err());
        assert!(validate_artifact_label("text/plain").is_ok());
        assert!(validate_artifact_label("").is_err());
        assert!(validate_artifact_label("text/plain';x").is_err());
        assert!(validate_artifact_label("text/plain;x").is_err());
        assert!(validate_artifact_label("text/plain\\").is_err());
        assert!(validate_artifact_label("text/plain\n").is_err());
        assert!(validate_artifact_label(&"x".repeat(129)).is_err());
        assert!(is_lowercase_hex("a1"));
        assert!(!is_lowercase_hex("AB"));
        assert!(select_source_artifact_by_id_sql(Uuid::nil()).contains("FROM source_artifact"));
    }
}
