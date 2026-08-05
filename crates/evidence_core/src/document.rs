//! Immutable UTF-8 document records linked to source artifacts.

use crate::{ContentDigest, EvidenceError, EvidenceId};
use std::sync::Arc;

const DEFAULT_DOCUMENT_BYTE_LIMIT: usize = 16 * 1024 * 1024;

/// An immutable UTF-8 document derived from one source artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRecord {
    id: EvidenceId,
    source_artifact_id: EvidenceId,
    content_digest: ContentDigest,
    text: Arc<str>,
    scalar_length: usize,
}

impl DocumentRecord {
    /// Copy UTF-8 text into an immutable record using the default limit.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::EmptyDocument`] for empty text or
    /// [`EvidenceError::DocumentTooLarge`] when the default limit is exceeded.
    pub fn from_text(
        source_artifact_id: EvidenceId,
        text: impl AsRef<str>,
    ) -> Result<Self, EvidenceError> {
        Self::from_text_with_limit(source_artifact_id, text, DEFAULT_DOCUMENT_BYTE_LIMIT)
    }

    /// Copy UTF-8 text into an immutable record bounded by `maximum_bytes`.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::EmptyDocument`] for empty text or
    /// [`EvidenceError::DocumentTooLarge`] when `maximum_bytes` is exceeded.
    pub fn from_text_with_limit(
        source_artifact_id: EvidenceId,
        text: impl AsRef<str>,
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        let text = text.as_ref();
        if text.is_empty() {
            return Err(EvidenceError::EmptyDocument);
        }
        if text.len() > maximum_bytes {
            return Err(EvidenceError::DocumentTooLarge);
        }

        Ok(Self {
            id: EvidenceId::new(),
            source_artifact_id,
            content_digest: ContentDigest::sha256(text.as_bytes()),
            scalar_length: text.chars().count(),
            text: Arc::from(text),
        })
    }

    /// Return the stable document identifier.
    #[must_use]
    pub const fn id(&self) -> EvidenceId {
        self.id
    }

    /// Return the source-artifact identifier.
    #[must_use]
    pub const fn source_artifact_id(&self) -> EvidenceId {
        self.source_artifact_id
    }

    /// Return the canonical UTF-8 text digest.
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest {
        self.content_digest
    }

    /// Return the immutable UTF-8 text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the UTF-8 byte length.
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.text.len()
    }

    /// Return the Unicode-scalar length.
    #[must_use]
    pub const fn scalar_length(&self) -> usize {
        self.scalar_length
    }

    /// Return whether `candidate` has the recorded UTF-8 text digest.
    #[must_use]
    pub fn verify_text(&self, candidate: impl AsRef<str>) -> bool {
        ContentDigest::sha256(candidate.as_ref().as_bytes()) == self.content_digest
    }
}
