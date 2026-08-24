//! In-memory bitemporal document store for contract-level adapters.

use crate::PersistenceError;
use crate::cutoff::is_cutoff_eligible;
use std::collections::{BTreeMap, BTreeSet};
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff, SystemTime};
use uuid::Uuid;

/// One bitemporal document version retained for historical replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRecord {
    /// Stable document identity shared across revisions.
    pub document_record_id: Uuid,
    /// Tenant boundary for authorization and isolation.
    pub tenant_record_id: Uuid,
    /// Hex-encoded content digest for the version payload.
    pub content_digest: String,
    /// When the document became available as evidence.
    pub available_time: AvailableTime,
    /// Inclusive event/valid-time start for this version's meaning.
    pub valid_from: EventTime,
    /// Exclusive or open event/valid-time end.
    pub valid_to: Option<EventTime>,
    /// Inclusive system-time start of this recorded version.
    pub system_from: SystemTime,
    /// Exclusive system-time end after supersession; `None` if current.
    pub system_to: Option<SystemTime>,
    /// Monotonic revision counter within the document identity.
    pub revision_number: u64,
}

/// Closed operational-log action for an inspected `audit_event` append.
pub const ACTION_AUDIT_EVENT_APPEND: u16 = 2_001;

/// Source payloads inspected before an `audit_event` insert is rendered.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AuditSourceInspection<'payload> {
    /// Raw source text that must not enter the audit row.
    pub source_text: Option<&'payload str>,
    /// Source identity that must not enter the audit row.
    pub source_identity: Option<&'payload str>,
    /// Whether a blanket PII mask is being treated as an insert grant.
    pub blanket_mask: bool,
}

impl AuditSourceInspection<'static> {
    /// Inspection that asserts no source text, source identity, or blanket mask.
    pub const CLEAR: Self = Self {
        source_text: None,
        source_identity: None,
        blanket_mask: false,
    };
}

/// Append-only audit row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEvent {
    /// Unique audit identity.
    pub audit_event_id: Uuid,
    /// Tenant boundary.
    pub tenant_record_id: Uuid,
    /// Stable action vocabulary code.
    pub action_code: String,
    /// Subject record identity referenced by the action.
    pub subject_record_id: Uuid,
    /// System time when the audit row was recorded.
    pub recorded_system_time: SystemTime,
}

/// In-memory bitemporal document and audit store.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentStore {
    versions: Vec<DocumentRecord>,
    audit_ids: BTreeSet<Uuid>,
    audit_count: usize,
}

impl DocumentStore {
    /// Create an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert the first system-time version of a document identity.
    ///
    /// # Errors
    ///
    /// Returns digest or duplicate-identity errors.
    pub fn insert(&mut self, record: DocumentRecord) -> Result<(), PersistenceError> {
        validate_digest(&record.content_digest)?;
        if self.has_document(record.document_record_id) {
            return Err(PersistenceError::DuplicateDocumentRecord);
        }
        self.versions.push(record);
        Ok(())
    }

    /// Append a revised version and close the previously open system-time row.
    ///
    /// # Errors
    ///
    /// Returns digest or historical-identity errors.
    pub fn revise(&mut self, record: DocumentRecord) -> Result<(), PersistenceError> {
        validate_digest(&record.content_digest)?;
        if !self.close_open_system_version(record.document_record_id, record.system_from) {
            return Err(PersistenceError::HistoricalVersionNotFound);
        }
        self.versions.push(record);
        Ok(())
    }

    fn has_document(&self, document_record_id: Uuid) -> bool {
        self.versions
            .iter()
            .any(|existing| existing.document_record_id == document_record_id)
    }

    fn close_open_system_version(
        &mut self,
        document_record_id: Uuid,
        system_from: SystemTime,
    ) -> bool {
        let mut closed = false;
        for existing in &mut self.versions {
            if is_open_version_for(existing, document_record_id) {
                existing.system_to = Some(system_from);
                closed = true;
            }
        }
        closed
    }

    /// Return the version visible as of a system-time instant.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::HistoricalVersionNotFound`] when no version
    /// covers the as-of system time.
    pub fn as_known_at(
        &self,
        document_record_id: Uuid,
        known_at: &SystemTime,
    ) -> Result<DocumentRecord, PersistenceError> {
        self.versions
            .iter()
            .filter(|row| row.document_record_id == document_record_id)
            .filter(|row| system_covers(row, known_at))
            .max_by_key(|row| row.revision_number)
            .cloned()
            .ok_or(PersistenceError::HistoricalVersionNotFound)
    }

    /// Return the version valid at an event time under a system-time as-of.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::HistoricalVersionNotFound`] when no version
    /// satisfies both dimensions.
    pub fn as_valid_at(
        &self,
        document_record_id: Uuid,
        valid_at: &EventTime,
        known_at: &SystemTime,
    ) -> Result<DocumentRecord, PersistenceError> {
        self.versions
            .iter()
            .filter(|row| row.document_record_id == document_record_id)
            .filter(|row| system_covers(row, known_at))
            .filter(|row| valid_covers(row, valid_at))
            .max_by_key(|row| row.revision_number)
            .cloned()
            .ok_or(PersistenceError::HistoricalVersionNotFound)
    }

    /// List currently open document versions eligible under a knowledge cutoff.
    ///
    /// # Errors
    ///
    /// Currently infallible; reserved for future adapter I/O failures.
    pub fn eligible_at_cutoff(
        &self,
        knowledge_cutoff: &KnowledgeCutoff,
    ) -> Result<Vec<DocumentRecord>, PersistenceError> {
        let mut latest: BTreeMap<Uuid, DocumentRecord> = BTreeMap::new();
        for row in &self.versions {
            if !is_currently_open(row) {
                continue;
            }
            if !is_cutoff_eligible(&row.available_time, knowledge_cutoff) {
                continue;
            }
            latest.insert(row.document_record_id, row.clone());
        }
        Ok(latest.into_values().collect())
    }

    /// Append an immutable audit event after operational-log inspection.
    ///
    /// # Errors
    ///
    /// Returns source-payload, action-code, or identity-reuse failures.
    pub fn append_audit(
        &mut self,
        event: &AuditEvent,
        inspection: AuditSourceInspection<'_>,
    ) -> Result<(), PersistenceError> {
        crate::document_sql::append_audit_sql(event, inspection)?;
        if self.audit_ids.contains(&event.audit_event_id) {
            return Err(PersistenceError::ImmutableAuditViolation);
        }
        self.audit_ids.insert(event.audit_event_id);
        self.audit_count += 1;
        Ok(())
    }

    /// Return the number of retained audit events.
    #[must_use]
    pub const fn audit_count(&self) -> usize {
        self.audit_count
    }
}

fn validate_digest(digest: &str) -> Result<(), PersistenceError> {
    if !is_sha256_hex(digest) {
        return Err(PersistenceError::InvalidContentDigest);
    }
    Ok(())
}

fn is_sha256_hex(digest: &str) -> bool {
    let length_ok = digest.len() == 64;
    let hex_ok = digest.chars().all(|ch| ch.is_ascii_hexdigit());
    length_ok & hex_ok
}

fn is_currently_open(row: &DocumentRecord) -> bool {
    row.system_to.is_none()
}

fn is_open_version_for(row: &DocumentRecord, document_record_id: Uuid) -> bool {
    row.document_record_id == document_record_id && is_currently_open(row)
}

fn system_covers(row: &DocumentRecord, known_at: &SystemTime) -> bool {
    if row.system_from > *known_at {
        return false;
    }
    match row.system_to {
        None => true,
        Some(system_to) => *known_at < system_to,
    }
}

fn valid_covers(row: &DocumentRecord, valid_at: &EventTime) -> bool {
    if row.valid_from > *valid_at {
        return false;
    }
    match row.valid_to {
        None => true,
        Some(valid_to) => *valid_at < valid_to,
    }
}

#[cfg(test)]
mod tests {
    use super::{AuditEvent, AuditSourceInspection, DocumentRecord, DocumentStore};
    use crate::PersistenceError;
    use temporal_core::{AvailableTime, EventTime, SystemTime};

    fn sample_record(revision: u64) -> DocumentRecord {
        DocumentRecord {
            document_record_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            content_digest: "f".repeat(64),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
            valid_from: EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("v"),
            valid_to: None,
            system_from: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            system_to: None,
            revision_number: revision,
        }
    }

    #[test]
    fn invalid_digest_and_missing_history_fail_closed() {
        let mut store = DocumentStore::new();
        let mut bad = sample_record(1);
        bad.content_digest = "short".into();
        assert_eq!(
            store.insert(bad),
            Err(PersistenceError::InvalidContentDigest)
        );
        assert_eq!(
            store.revise(sample_record(2)),
            Err(PersistenceError::HistoricalVersionNotFound)
        );
        assert_eq!(
            store.as_known_at(
                uuid::Uuid::nil(),
                &SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s")
            ),
            Err(PersistenceError::HistoricalVersionNotFound)
        );
        let good = sample_record(1);
        store.insert(good.clone()).expect("first insert");
        assert_eq!(
            store.insert(good),
            Err(PersistenceError::DuplicateDocumentRecord)
        );
        // A second document identity ensures revise walks a non-matching open row.
        let mut other = sample_record(1);
        other.document_record_id = uuid::Uuid::now_v7();
        other.content_digest = "c".repeat(64);
        store.insert(other).expect("second identity");
        let mut revised = sample_record(2);
        revised.content_digest = "d".repeat(64);
        revised.system_from =
            SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("later system");
        store.revise(revised).expect("revise first identity only");
        let audit = AuditEvent {
            audit_event_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            action_code: "x".into(),
            subject_record_id: uuid::Uuid::nil(),
            recorded_system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
        };
        store
            .append_audit(&audit, AuditSourceInspection::CLEAR)
            .expect("first audit");
        assert_eq!(
            store.append_audit(&audit, AuditSourceInspection::CLEAR),
            Err(PersistenceError::ImmutableAuditViolation)
        );
        assert_eq!(store.audit_count(), 1);
        let mut refused = audit.clone();
        refused.audit_event_id = uuid::Uuid::from_u128(1);
        assert_eq!(
            store.append_audit(
                &refused,
                AuditSourceInspection {
                    source_text: Some("source"),
                    source_identity: None,
                    blanket_mask: false,
                },
            ),
            Err(PersistenceError::SourceTextNotAuditable)
        );
        assert_eq!(store.audit_count(), 1);
    }

    #[test]
    fn revised_versions_are_skipped_by_cutoff_listing() {
        let mut store = DocumentStore::new();
        let mut first = sample_record(1);
        first.document_record_id = uuid::Uuid::now_v7();
        first.tenant_record_id = uuid::Uuid::now_v7();
        let document_id = first.document_record_id;
        store.insert(first).expect("insert");
        let mut second = sample_record(2);
        second.document_record_id = document_id;
        second.tenant_record_id = uuid::Uuid::now_v7();
        second.system_from =
            SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("later system");
        store.revise(second).expect("revise");
        let eligible = store
            .eligible_at_cutoff(
                &temporal_core::KnowledgeCutoff::parse_rfc3339("2026-12-01T00:00:00Z")
                    .expect("cutoff"),
            )
            .expect("eligible");
        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].revision_number, 2);

        // Closed system interval excludes post-supersession as-known-at queries.
        assert_eq!(
            store
                .as_known_at(
                    document_id,
                    &SystemTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("after both")
                )
                .expect("current")
                .revision_number,
            2
        );
    }

    #[test]
    fn helper_predicates_cover_both_match_arms() {
        use super::{
            is_currently_open, is_open_version_for, is_sha256_hex, system_covers, valid_covers,
        };
        assert!(is_sha256_hex(&"ab".repeat(32)));
        assert!(!is_sha256_hex("nope"));
        let open = sample_record(1);
        assert!(is_currently_open(&open));
        assert!(is_open_version_for(&open, uuid::Uuid::nil()));
        assert!(!is_open_version_for(&open, uuid::Uuid::now_v7()));
        let mut closed = open.clone();
        closed.system_to = Some(SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("st"));
        assert!(!is_currently_open(&closed));
        assert!(system_covers(
            &open,
            &SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("k")
        ));
        assert!(!system_covers(
            &open,
            &SystemTime::parse_rfc3339("2025-01-01T00:00:00Z").expect("early")
        ));
        assert!(!system_covers(
            &closed,
            &SystemTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("late")
        ));
        assert!(system_covers(
            &closed,
            &SystemTime::parse_rfc3339("2026-01-15T00:00:00Z").expect("mid")
        ));
        let mut bounded = open.clone();
        bounded.valid_to = Some(EventTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("vt"));
        assert!(valid_covers(
            &bounded,
            &EventTime::parse_rfc3339("2026-01-15T00:00:00Z").expect("mid")
        ));
        assert!(!valid_covers(
            &bounded,
            &EventTime::parse_rfc3339("2025-01-01T00:00:00Z").expect("early")
        ));
        assert!(!valid_covers(
            &bounded,
            &EventTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("late")
        ));
        assert!(valid_covers(
            &open,
            &EventTime::parse_rfc3339("2026-06-01T00:00:00Z").expect("open late")
        ));
    }

    #[test]
    fn closed_valid_windows_and_ineligible_open_rows_are_exercised() {
        let mut store = DocumentStore::new();
        let document = uuid::Uuid::now_v7();
        let record = DocumentRecord {
            document_record_id: document,
            tenant_record_id: uuid::Uuid::now_v7(),
            content_digest: "a".repeat(64),
            available_time: AvailableTime::parse_rfc3339("2026-08-01T00:00:00Z").expect("a"),
            valid_from: EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("vf"),
            valid_to: Some(EventTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("vt")),
            system_from: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("sf"),
            system_to: Some(SystemTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("st")),
            revision_number: 1,
        };
        store.insert(record).expect("insert bounded");

        assert_eq!(
            store
                .as_valid_at(
                    document,
                    &EventTime::parse_rfc3339("2026-01-15T00:00:00Z").expect("inside"),
                    &SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("known")
                )
                .expect("inside window")
                .revision_number,
            1
        );
        assert_eq!(
            store.as_valid_at(
                document,
                &EventTime::parse_rfc3339("2026-02-15T00:00:00Z").expect("outside"),
                &SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("known")
            ),
            Err(PersistenceError::HistoricalVersionNotFound)
        );
        assert_eq!(
            store.as_known_at(
                document,
                &SystemTime::parse_rfc3339("2026-04-01T00:00:00Z").expect("after system close")
            ),
            Err(PersistenceError::HistoricalVersionNotFound)
        );

        // Open row present but unavailable at an early cutoff is filtered out.
        let open = DocumentRecord {
            document_record_id: uuid::Uuid::now_v7(),
            tenant_record_id: uuid::Uuid::now_v7(),
            content_digest: "b".repeat(64),
            available_time: AvailableTime::parse_rfc3339("2026-09-01T00:00:00Z").expect("late"),
            valid_from: EventTime::parse_rfc3339("2026-09-01T00:00:00Z").expect("vf"),
            valid_to: None,
            system_from: SystemTime::parse_rfc3339("2026-09-01T00:00:00Z").expect("sf"),
            system_to: None,
            revision_number: 1,
        };
        store.insert(open).expect("late open");
        let eligible = store
            .eligible_at_cutoff(
                &temporal_core::KnowledgeCutoff::parse_rfc3339("2026-01-01T00:00:00Z")
                    .expect("early cutoff"),
            )
            .expect("eligible");
        assert!(eligible.is_empty());
    }
}
