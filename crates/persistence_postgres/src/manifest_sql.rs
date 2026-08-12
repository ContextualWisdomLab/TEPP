//! SQL contracts for append-only reproducibility manifests (ADR 0013).

use crate::PersistenceError;
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

/// Immutable reproducibility manifest row bound to a tenant and digests.
///
/// Maps to `reproducibility_manifest` in migration `0001`. Digests are
/// lowercase hex `SHA-256` strings (exactly 64 `0-9a-f` characters);
/// `code_commit_sha` is a full Git object id (exactly 40 or 64 lowercase hex).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReproducibilityManifestRecord {
    /// Primary key for this manifest identity.
    pub reproducibility_manifest_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Knowledge cutoff applied when the bound run was estimated.
    pub knowledge_cutoff: AvailableTime,
    /// Hex-encoded evidence/content digest for the primary analytical payload.
    pub evidence_digest: String,
    /// Git commit identity of the producing code revision.
    pub code_commit_sha: String,
    /// Hex-encoded dependency-lock digest bound into the run.
    pub dependency_lock_digest: String,
    /// System/record time when the manifest was asserted.
    pub system_time: SystemTime,
    /// Availability time of the manifest evidence.
    pub available_time: AvailableTime,
}

impl ReproducibilityManifestRecord {
    /// Fail-closed field validation for digests and commit identity.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidContentDigest`] for non-hex digests or
    /// empty commit identities.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_sha256_hex(&self.evidence_digest)?;
        validate_sha256_hex(&self.dependency_lock_digest)?;
        validate_commit_sha(&self.code_commit_sha)?;
        Ok(())
    }
}

/// Render append-only insert SQL for a validated reproducibility manifest.
///
/// # Errors
///
/// Returns digest/commit validation failures before any SQL is produced.
pub fn insert_reproducibility_manifest_sql(
    record: &ReproducibilityManifestRecord,
) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO reproducibility_manifest (\
            reproducibility_manifest_id, tenant_record_id, knowledge_cutoff, \
            evidence_digest, code_commit_sha, dependency_lock_digest, \
            system_time, available_time\
        ) VALUES (\
            '{manifest_id}'::uuid, '{tenant_id}'::uuid, '{cutoff}'::timestamptz, \
            '{evidence}', '{commit}', '{lock_digest}', \
            '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        manifest_id = record.reproducibility_manifest_id,
        tenant_id = record.tenant_record_id,
        cutoff = record.knowledge_cutoff.to_rfc3339(),
        evidence = record.evidence_digest,
        commit = escape_literal(&record.code_commit_sha),
        lock_digest = record.dependency_lock_digest,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render selection by the unique digest triple used for idempotent lookup.
#[must_use]
pub fn select_reproducibility_manifest_by_digests_sql(
    evidence_digest: &str,
    code_commit_sha: &str,
    dependency_lock_digest: &str,
) -> String {
    format!(
        "SELECT reproducibility_manifest_id, tenant_record_id, knowledge_cutoff, \
                evidence_digest, code_commit_sha, dependency_lock_digest, \
                system_time, available_time \
         FROM reproducibility_manifest \
         WHERE evidence_digest = '{evidence}' \
           AND code_commit_sha = '{commit}' \
           AND dependency_lock_digest = '{lock_digest}' \
         LIMIT 1",
        evidence = escape_literal(evidence_digest),
        commit = escape_literal(code_commit_sha),
        lock_digest = escape_literal(dependency_lock_digest),
    )
}

/// Render selection by primary key under the active tenant RLS context.
#[must_use]
pub fn select_reproducibility_manifest_by_id_sql(reproducibility_manifest_id: Uuid) -> String {
    format!(
        "SELECT reproducibility_manifest_id, tenant_record_id, knowledge_cutoff, \
                evidence_digest, code_commit_sha, dependency_lock_digest, \
                system_time, available_time \
         FROM reproducibility_manifest \
         WHERE reproducibility_manifest_id = '{reproducibility_manifest_id}'::uuid \
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
        return Err(PersistenceError::InvalidContentDigest);
    }
    Ok(())
}

fn validate_commit_sha(value: &str) -> Result<(), PersistenceError> {
    if !matches!(value.len(), 40 | 64) || !is_lowercase_hex(value) {
        return Err(PersistenceError::InvalidContentDigest);
    }
    Ok(())
}

fn escape_literal(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::{
        ReproducibilityManifestRecord, insert_reproducibility_manifest_sql,
        select_reproducibility_manifest_by_digests_sql, select_reproducibility_manifest_by_id_sql,
        validate_commit_sha, validate_sha256_hex,
    };
    use crate::PersistenceError;
    use temporal_core::{AvailableTime, SystemTime};
    use uuid::Uuid;

    fn sample() -> ReproducibilityManifestRecord {
        ReproducibilityManifestRecord {
            reproducibility_manifest_id: Uuid::nil(),
            tenant_record_id: Uuid::nil(),
            knowledge_cutoff: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("k"),
            evidence_digest: "ab".repeat(32),
            code_commit_sha: "c".repeat(40),
            dependency_lock_digest: "de".repeat(32),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    #[test]
    fn manifest_sql_insert_and_select_fail_closed() {
        let record = sample();
        let insert = insert_reproducibility_manifest_sql(&record).expect("insert");
        assert!(insert.contains("INSERT INTO reproducibility_manifest"));
        assert!(insert.contains(&record.evidence_digest));
        assert!(insert.contains(&record.code_commit_sha));

        let by_digest = select_reproducibility_manifest_by_digests_sql(
            &record.evidence_digest,
            &record.code_commit_sha,
            &record.dependency_lock_digest,
        );
        assert!(by_digest.contains("evidence_digest ="));
        assert!(by_digest.contains("LIMIT 1"));

        let by_id = select_reproducibility_manifest_by_id_sql(Uuid::nil());
        assert!(by_id.contains("reproducibility_manifest_id"));

        let mut bad = record.clone();
        bad.evidence_digest = "short".into();
        assert_eq!(
            insert_reproducibility_manifest_sql(&bad),
            Err(PersistenceError::InvalidContentDigest)
        );
        bad = record.clone();
        bad.dependency_lock_digest = "nope".into();
        assert_eq!(
            insert_reproducibility_manifest_sql(&bad),
            Err(PersistenceError::InvalidContentDigest)
        );
        bad = record;
        bad.code_commit_sha.clear();
        assert_eq!(
            insert_reproducibility_manifest_sql(&bad),
            Err(PersistenceError::InvalidContentDigest)
        );

        assert!(validate_sha256_hex(&"ff".repeat(32)).is_ok());
        assert!(validate_sha256_hex("zz").is_err());
        // Length-correct but non-hex / uppercase must fail closed.
        assert!(validate_sha256_hex(&"g".repeat(64)).is_err());
        assert!(validate_sha256_hex(&"FF".repeat(32)).is_err());
        assert!(validate_commit_sha(&"ab".repeat(20)).is_ok());
        assert!(validate_commit_sha(&"cd".repeat(32)).is_ok());
        assert!(validate_commit_sha("abc123").is_err());
        assert!(validate_commit_sha("").is_err());
        assert!(validate_commit_sha(&"a".repeat(41)).is_err());
        assert!(validate_commit_sha(&"A".repeat(40)).is_err());
        assert!(validate_commit_sha("deadbeef-cafe_01").is_err());
        assert_eq!(super::escape_literal("a'b"), "a''b");
    }
}
