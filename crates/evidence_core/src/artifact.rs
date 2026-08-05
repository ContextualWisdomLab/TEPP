//! Immutable source-artifact bytes and mutation detection.

use crate::{ContentDigest, EvidenceError, EvidenceId};
use std::sync::Arc;

const DEFAULT_SOURCE_ARTIFACT_BYTE_LIMIT: usize = 64 * 1024 * 1024;

/// An immutable source artifact identified independently from its content hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceArtifact {
    id: EvidenceId,
    content_digest: ContentDigest,
    content: Arc<[u8]>,
}

impl SourceArtifact {
    /// Copy source bytes into an immutable artifact using the default limit.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::EmptySourceArtifact`] for empty input or
    /// [`EvidenceError::SourceArtifactTooLarge`] when the default limit is
    /// exceeded.
    pub fn from_bytes(content: impl AsRef<[u8]>) -> Result<Self, EvidenceError> {
        Self::from_bytes_with_limit(content, DEFAULT_SOURCE_ARTIFACT_BYTE_LIMIT)
    }

    /// Copy source bytes into an immutable artifact bounded by `maximum_bytes`.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::EmptySourceArtifact`] for empty input or
    /// [`EvidenceError::SourceArtifactTooLarge`] when `maximum_bytes` is
    /// exceeded.
    pub fn from_bytes_with_limit(
        content: impl AsRef<[u8]>,
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        let content = content.as_ref();
        if content.is_empty() {
            return Err(EvidenceError::EmptySourceArtifact);
        }
        if content.len() > maximum_bytes {
            return Err(EvidenceError::SourceArtifactTooLarge);
        }

        Ok(Self {
            id: EvidenceId::new(),
            content_digest: ContentDigest::sha256(content),
            content: Arc::from(content),
        })
    }

    /// Return the stable artifact identifier.
    #[must_use]
    pub const fn id(&self) -> EvidenceId {
        self.id
    }

    /// Return the canonical content digest.
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest {
        self.content_digest
    }

    /// Return the immutable artifact bytes.
    #[must_use]
    pub fn content(&self) -> &[u8] {
        &self.content
    }

    /// Return the immutable artifact size in bytes.
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.content.len()
    }

    /// Return whether `candidate` has the recorded content digest.
    #[must_use]
    pub fn verify_content(&self, candidate: impl AsRef<[u8]>) -> bool {
        ContentDigest::sha256(candidate) == self.content_digest
    }
}
