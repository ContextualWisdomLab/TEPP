//! Fail-closed restore integrity before analytical state is marked usable.

use crate::PersistenceError;
use crate::cutoff::is_cutoff_eligible;
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff};
use uuid::Uuid;

/// Tables that a TEPP backup must include and a restore must revalidate.
pub const BACKUP_SCOPE_TABLES: &[&str] = &[
    "tenant_record",
    "source_artifact",
    "document_record",
    "audit_event",
    "reproducibility_manifest",
    "corpus_split_manifest",
    "model_run",
    "model_artifact",
];

/// Restored row values that must be revalidated before use (ADR 0013).
///
/// A backup copy is untrusted. Callers supply the reconstructed identities
/// and clocks; [`mark_restored_state_usable`] is the only constructor of a
/// usable analytical state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoredAnalyticalSnapshot {
    /// Owning tenant; missing identity fails closed.
    pub tenant_record_id: Option<Uuid>,
    /// Canonical lowercase hex `SHA-256` of the bound source bytes.
    pub content_sha256: String,
    /// Availability time of the restored evidence.
    pub available_time: AvailableTime,
    /// Knowledge cutoff that the restored fit must honor.
    pub knowledge_cutoff: KnowledgeCutoff,
    /// Valid-time lower bound of the restored document version.
    pub valid_from: EventTime,
    /// Optional valid-time upper bound; must not precede `valid_from`.
    pub valid_to: Option<EventTime>,
    /// Whether append-only immutability triggers were present after restore.
    pub append_only_triggers_present: bool,
}

/// Opaque usable-state token produced only after restore integrity passes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreUsableState {
    usable: bool,
}

impl RestoreUsableState {
    /// Whether analytical reads may proceed against the restored snapshot.
    #[must_use]
    pub const fn is_usable(&self) -> bool {
        self.usable
    }
}

/// Return the physical tables a backup/restore pair must cover.
#[must_use]
pub fn backup_scope_tables() -> &'static [&'static str] {
    BACKUP_SCOPE_TABLES
}

/// SQL probes that fail closed when restored physical rows are unusable.
///
/// Each statement is a `DO` block that raises `restore integrity failed` when
/// a digest, tenant, temporal-window, or append-only trigger check fails.
#[must_use]
pub fn restore_integrity_probe_sqls() -> Vec<String> {
    vec![
        probe(
            "digest",
            "SELECT 1 FROM source_artifact \
             WHERE content_sha256 !~ '^[0-9a-f]{64}$' \
                OR tenant_record_id IS NULL",
        ),
        probe(
            "document_window",
            "SELECT 1 FROM document_record \
             WHERE tenant_record_id IS NULL \
                OR valid_to < valid_from \
                OR system_to < system_from",
        ),
        probe(
            "cutoff",
            "SELECT 1 FROM source_artifact \
             WHERE available_time > \
                (SELECT MAX(knowledge_cutoff) FROM reproducibility_manifest)",
        ),
        probe(
            "append_only",
            "SELECT 1 WHERE NOT EXISTS (\
                SELECT 1 FROM pg_proc \
                WHERE proname = 'reject_append_only_mutation')",
        ),
    ]
}

/// Revalidate a reconstructed snapshot and mark analytical state usable.
///
/// # Errors
///
/// Returns [`PersistenceError::RestoreIntegrityFailed`] when the tenant is
/// missing, the digest is not canonical `SHA-256`, availability exceeds the
/// cutoff, a valid window is inverted, or append-only triggers are absent.
pub fn mark_restored_state_usable(
    snapshot: &RestoredAnalyticalSnapshot,
) -> Result<RestoreUsableState, PersistenceError> {
    if snapshot.tenant_record_id.is_none() {
        return Err(PersistenceError::RestoreIntegrityFailed);
    }
    if !is_canonical_sha256(&snapshot.content_sha256) {
        return Err(PersistenceError::RestoreIntegrityFailed);
    }
    if !is_cutoff_eligible(&snapshot.available_time, &snapshot.knowledge_cutoff) {
        return Err(PersistenceError::RestoreIntegrityFailed);
    }
    if snapshot
        .valid_to
        .as_ref()
        .is_some_and(|until| until.instant() < snapshot.valid_from.instant())
    {
        return Err(PersistenceError::RestoreIntegrityFailed);
    }
    if !snapshot.append_only_triggers_present {
        return Err(PersistenceError::RestoreIntegrityFailed);
    }
    Ok(RestoreUsableState { usable: true })
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn probe(tag: &str, predicate: &str) -> String {
    format!(
        "DO $tepp_restore_{tag}$\n\
         BEGIN\n\
           IF EXISTS ({predicate}) THEN\n\
             RAISE EXCEPTION 'restore integrity failed';\n\
           END IF;\n\
         END\n\
         $tepp_restore_{tag}$"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        RestoredAnalyticalSnapshot, backup_scope_tables, is_canonical_sha256,
        mark_restored_state_usable, restore_integrity_probe_sqls,
    };
    use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff};
    use uuid::Uuid;

    fn sample() -> RestoredAnalyticalSnapshot {
        RestoredAnalyticalSnapshot {
            tenant_record_id: Some(Uuid::nil()),
            content_sha256: "09".repeat(32),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
            knowledge_cutoff: KnowledgeCutoff::parse_rfc3339("2026-01-01T00:00:00Z").expect("k"),
            valid_from: EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("vf"),
            valid_to: Some(EventTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("vt")),
            append_only_triggers_present: true,
        }
    }

    #[test]
    fn helpers_cover_digest_and_open_window_success() {
        assert!(is_canonical_sha256(&"ab".repeat(32)));
        assert!(!is_canonical_sha256("AB"));
        assert!(mark_restored_state_usable(&sample()).is_ok());
        assert!(
            restore_integrity_probe_sqls()
                .iter()
                .any(|sql| sql.contains("$tepp_restore_digest$"))
        );
        assert!(backup_scope_tables().len() >= 8);
    }
}
