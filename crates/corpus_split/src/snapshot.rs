//! Knowledge-cutoff corpus snapshots.

use crate::CorpusDocument;
use crate::CorpusSplitError;
use crate::cutoff_eligible;
use std::collections::BTreeMap;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use uuid::Uuid;

/// Immutable snapshot of documents eligible under a knowledge cutoff.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CorpusSnapshot {
    documents: BTreeMap<Uuid, CorpusDocument>,
}

impl CorpusSnapshot {
    /// Create an empty snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a document if it is eligible under `knowledge_cutoff`.
    ///
    /// # Errors
    ///
    /// Returns unavailability or duplicate-identity errors.
    pub fn insert_if_eligible(
        &mut self,
        document: CorpusDocument,
        knowledge_cutoff: &KnowledgeCutoff,
    ) -> Result<(), CorpusSplitError> {
        if !cutoff_eligible(&document.available_time, knowledge_cutoff) {
            return Err(CorpusSplitError::UnavailableAtCutoff);
        }
        if self.documents.contains_key(&document.document_id) {
            return Err(CorpusSplitError::DuplicateDocumentIdentity);
        }
        self.documents.insert(document.document_id, document);
        Ok(())
    }

    /// Return whether the snapshot contains a document identity.
    #[must_use]
    pub fn contains(&self, document_id: Uuid) -> bool {
        self.documents.contains_key(&document_id)
    }

    /// Return the availability time retained for one snapshot document.
    #[must_use]
    pub fn available_time(&self, document_id: Uuid) -> Option<AvailableTime> {
        self.documents
            .get(&document_id)
            .map(|document| document.available_time)
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
        snapshot
            .insert_if_eligible(early.clone(), &cutoff)
            .expect("early");
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
        assert_eq!(
            snapshot.available_time(early.document_id),
            Some(early.available_time)
        );
        assert_eq!(snapshot.available_time(Uuid::nil()), None);
        assert_eq!(snapshot.document_ids().count(), 1);
    }
}
