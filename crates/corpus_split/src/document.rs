//! Document observations eligible for cutoff-aware snapshots.

use temporal_core::AvailableTime;
use uuid::Uuid;

/// One document observation with availability provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusDocument {
    /// Stable analytical document identity.
    pub document_id: Uuid,
    /// When the document became available as evidence.
    pub available_time: AvailableTime,
}

impl CorpusDocument {
    /// Construct a document observation.
    #[must_use]
    pub const fn new(document_id: Uuid, available_time: AvailableTime) -> Self {
        Self {
            document_id,
            available_time,
        }
    }
}
