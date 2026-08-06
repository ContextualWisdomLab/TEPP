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
    pub fn from_bytes(content: &[u8]) -> Result<Self, EvidenceError> {
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
        content: &[u8],
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        validate_content(content, maximum_bytes)?;

        Ok(Self {
            id: EvidenceId::new(),
            content_digest: ContentDigest::sha256(content),
            content: Arc::from(content),
        })
    }

    /// Serialize this artifact through the strict versioned JSON wire contract.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] if JSON serialization
    /// cannot represent the validated artifact.
    pub fn to_wire_json(&self) -> Result<String, EvidenceError> {
        crate::wire::serialize_source_artifact(self)
    }

    /// Reconstruct and validate an artifact from versioned JSON using the default limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, identifier, digest, size, or content-integrity
    /// error when the payload does not reconstruct this domain type exactly.
    pub fn from_wire_json(payload: &str) -> Result<Self, EvidenceError> {
        Self::from_wire_json_with_limit(payload, DEFAULT_SOURCE_ARTIFACT_BYTE_LIMIT)
    }

    /// Reconstruct and validate an artifact from versioned JSON with a byte limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, identifier, digest, size, or content-integrity
    /// error when the payload does not reconstruct this domain type exactly.
    pub fn from_wire_json_with_limit(
        payload: &str,
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        crate::wire::deserialize_source_artifact(payload, maximum_bytes)
    }

    pub(crate) fn from_wire_parts(
        id: EvidenceId,
        content_digest: ContentDigest,
        content: Vec<u8>,
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        validate_content(&content, maximum_bytes)?;
        if ContentDigest::sha256(&content) != content_digest {
            return Err(EvidenceError::ContentDigestMismatch);
        }

        Ok(Self {
            id,
            content_digest,
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
    pub fn verify_content(&self, candidate: &[u8]) -> bool {
        ContentDigest::sha256(candidate) == self.content_digest
    }
}

fn validate_content(content: &[u8], maximum_bytes: usize) -> Result<(), EvidenceError> {
    if content.is_empty() {
        return Err(EvidenceError::EmptySourceArtifact);
    }
    if content.len() > maximum_bytes {
        return Err(EvidenceError::SourceArtifactTooLarge);
    }
    Ok(())
}
