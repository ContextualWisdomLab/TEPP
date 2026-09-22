//! Knowledge-cutoff corpus snapshots.

use crate::CorpusDocument;
use crate::CorpusSplitError;
use crate::cutoff_eligible;
use std::collections::BTreeMap;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use uuid::Uuid;

/// Immutable snapshot of documents eligible under one knowledge cutoff.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CorpusSnapshot {
    knowledge_cutoff: Option<KnowledgeCutoff>,
    documents: BTreeMap<Uuid, CorpusDocument>,
}

impl CorpusSnapshot {
    /// Create an empty snapshot whose knowledge horizon is not bound yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a document if it is eligible under this snapshot's knowledge cutoff.
    ///
    /// The first successful insertion binds the snapshot to `knowledge_cutoff`.
    /// Every later insertion must use that exact same cutoff so one snapshot can
    /// never combine evidence admitted under different historical horizons.
    ///
    /// # Errors
    ///
    /// Returns cutoff-mismatch, unavailability, or duplicate-identity errors.
    pub fn insert_if_eligible(
        &mut self,
        document: CorpusDocument,
        knowledge_cutoff: &KnowledgeCutoff,
    ) -> Result<(), CorpusSplitError> {
        if self
            .knowledge_cutoff
            .is_some_and(|bound_cutoff| bound_cutoff != *knowledge_cutoff)
        {
            return Err(CorpusSplitError::KnowledgeCutoffMismatch);
        }
        if !cutoff_eligible(&document.available_time, knowledge_cutoff) {
            return Err(CorpusSplitError::UnavailableAtCutoff);
        }
        if self.documents.contains_key(&document.document_id) {
            return Err(CorpusSplitError::DuplicateDocumentIdentity);
        }
        if self.knowledge_cutoff.is_none() {
            self.knowledge_cutoff = Some(*knowledge_cutoff);
        }
        self.documents.insert(document.document_id, document);
        Ok(())
    }

    /// Return the historical knowledge horizon bound to this snapshot.
    ///
    /// Empty snapshots return `None` until the first document is successfully
    /// admitted. Once bound, the cutoff cannot change.
    #[must_use]
    pub const fn knowledge_cutoff(&self) -> Option<KnowledgeCutoff> {
        self.knowledge_cutoff
    }

    /// Return whether the snapshot contains a document identity.
    #[must_use]
    pub fn contains(&self, document_id: Uuid) -> bool {
        self.documents.contains_key(&document_id)
    }

    /// Return the availability time retained for one admitted document.
    ///
    /// This preserves the distinction between a document omitted from another
    /// snapshot and a document that was genuinely unavailable at that earlier
    /// horizon. Absence alone is not evidence of future availability.
    #[must_use]
    pub fn available_time(&self, document_id: Uuid) -> Option<&AvailableTime> {
        self.documents
            .get(&document_id)
            .map(|document| &document.available_time)
    }

    /// Return the number of eligible documents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Return whether the snapshot is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Iterate document identities in sorted order.
    pub fn document_ids(&self) -> impl Iterator<Item = Uuid> + '_ {
        self.documents.keys().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::CorpusSnapshot;
    use crate::{CorpusDocument, CorpusSplitError};
    use temporal_core::{AvailableTime, KnowledgeCutoff};
    use uuid::Uuid;

    #[test]
    fn late_available_documents_are_excluded() {
        let mut snapshot = CorpusSnapshot::new();
        let cutoff = KnowledgeCutoff::parse_rfc3339("2026-03-01T00:00:00Z").expect("cutoff");
        let early = CorpusDocument::new(
            Uuid::now_v7(),
            AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        );
        let late = CorpusDocument::new(
            Uuid::now_v7(),
            AvailableTime::parse_rfc3339("2026-08-01T00:00:00Z").expect("a"),
        );
        assert_eq!(snapshot.knowledge_cutoff(), None);
        assert_eq!(snapshot.available_time(early.document_id), None);
        snapshot
            .insert_if_eligible(early.clone(), &cutoff)
            .expect("early");
        assert_eq!(snapshot.knowledge_cutoff(), Some(cutoff));
        assert_eq!(snapshot.available_time(early.document_id), Some(&early.available_time));
        assert_eq!(
            snapshot.insert_if_eligible(late, &cutoff),
            Err(CorpusSplitError::UnavailableAtCutoff)
        );
        assert_eq!(
            snapshot.insert_if_eligible(early.clone(), &cutoff),
            Err(CorpusSplitError::DuplicateDocumentIdentity)
        );
        assert_eq!(snapshot.len(), 1);
        assert!(!snapshot.is_empty());
        assert!(snapshot.contains(early.document_id));
        assert_eq!(snapshot.document_ids().count(), 1);
    }
}
