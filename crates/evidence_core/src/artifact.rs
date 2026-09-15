//! Immutable source-artifact bytes and mutation detection.

use crate::{ContentDigest, EvidenceError, EvidenceId};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_SOURCE_ARTIFACT_BYTE_LIMIT: usize = 64 * 1024 * 1024;

/// An Evidence-issued, non-forgeable handle for one trusted source artifact.
///
/// The wrapped [`EvidenceId`] is intentionally private and there is no parser or
/// public conversion from a bare identifier. Callers obtain this handle only
/// from [`SourceArtifact::id`], so APIs that require source-artifact provenance
/// can reject syntactically valid but unbacked evidence identifiers by type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceArtifactId(EvidenceId);

impl SourceArtifactId {
    /// Return the underlying UUID value without enabling handle reconstruction.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0.as_uuid()
    }

    pub(crate) const fn evidence_id(self) -> EvidenceId {
        self.0
    }
}

impl fmt::Display for SourceArtifactId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl PartialEq<EvidenceId> for SourceArtifactId {
    fn eq(&self, other: &EvidenceId) -> bool {
        self.0 == *other
    }
}

impl PartialEq<SourceArtifactId> for EvidenceId {
    fn eq(&self, other: &SourceArtifactId) -> bool {
        *self == other.0
    }
}

/// An immutable source artifact whose identity is issued inside Evidence.
///
/// Generic JSON parsing never constructs this trusted domain type. New ingress
/// uses [`SourceArtifact::from_bytes`], which mints the artifact identity inside
/// the Evidence bounded context. Persisted/external JSON is represented by
/// [`ValidatedSourceArtifactWire`] and remains untrusted until an authenticated
/// Evidence-owned repository restoration boundary is implemented.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceArtifact {
    id: SourceArtifactId,
    content_digest: ContentDigest,
    content: Arc<[u8]>,
}

/// Canonical, structurally validated source-artifact wire input.
///
/// This type proves that the supplied artifact identifier is a valid UUIDv7,
/// the declared SHA-256 matches the exact bytes, the byte limit is respected,
/// and the JSON is canonical. It does **not** prove that Evidence issued,
/// authenticated, authorized, or previously persisted that identifier.
///
/// There is intentionally no public conversion from this wire type into
/// [`SourceArtifact`]. Persistence replay must authenticate and authorize the
/// stored record at an Evidence-owned repository boundary before a trusted
/// restoration API may be introduced; generic JSON parsing fails closed instead
/// of acting as that boundary.
///
/// ```compile_fail
/// use evidence_core::{SourceObservation, ValidatedSourceArtifactWire};
/// let wire = ValidatedSourceArtifactWire::from_json("{}").expect("validated wire");
/// let _ = SourceObservation::observe(&wire);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedSourceArtifactWire {
    artifact_id: EvidenceId,
    content_digest: ContentDigest,
    content: Arc<[u8]>,
}

impl SourceArtifact {
    /// Copy source bytes into an immutable owner-issued artifact using the default limit.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::EmptySourceArtifact`] for empty input or
    /// [`EvidenceError::SourceArtifactTooLarge`] when the default limit is
    /// exceeded.
    pub fn from_bytes(content: &[u8]) -> Result<Self, EvidenceError> {
        Self::from_bytes_with_limit(content, DEFAULT_SOURCE_ARTIFACT_BYTE_LIMIT)
    }

    /// Copy source bytes into an immutable owner-issued artifact bounded by `maximum_bytes`.
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
            id: SourceArtifactId(EvidenceId::new()),
            content_digest: ContentDigest::sha256(content),
            content: Arc::from(content),
        })
    }

    /// Serialize this trusted artifact through the strict versioned JSON wire contract.
    ///
    /// Serialization does not make a later JSON parse trusted; consumers receive
    /// [`ValidatedSourceArtifactWire`] until an authenticated repository boundary
    /// restores owner state.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] if JSON serialization
    /// cannot represent the validated artifact.
    pub fn to_wire_json(&self) -> Result<String, EvidenceError> {
        crate::wire::serialize_source_artifact(self)
    }

    /// Return the stable Evidence-issued source-artifact handle.
    #[must_use]
    pub const fn id(&self) -> SourceArtifactId {
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

impl ValidatedSourceArtifactWire {
    /// Parse canonical source-artifact JSON using the default content limit.
    ///
    /// Successful parsing validates structure and content integrity only. It
    /// does not promote the caller-selected artifact identity into owner-issued
    /// Evidence state.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, identifier, digest, canonicality, size, or
    /// content-integrity error.
    pub fn from_json(payload: &str) -> Result<Self, EvidenceError> {
        Self::from_json_with_limit(payload, DEFAULT_SOURCE_ARTIFACT_BYTE_LIMIT)
    }

    /// Parse canonical source-artifact JSON with an explicit content limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, identifier, digest, canonicality, size, or
    /// content-integrity error.
    pub fn from_json_with_limit(
        payload: &str,
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        crate::wire::deserialize_source_artifact(payload, maximum_bytes)
    }

    pub(crate) fn from_wire_parts(
        artifact_id: EvidenceId,
        content_digest: ContentDigest,
        content: Vec<u8>,
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        validate_content(&content, maximum_bytes)?;
        if ContentDigest::sha256(&content) != content_digest {
            return Err(EvidenceError::ContentDigestMismatch);
        }

        Ok(Self {
            artifact_id,
            content_digest,
            content: Arc::from(content),
        })
    }

    /// Return the caller-supplied artifact identifier after structural validation.
    #[must_use]
    pub const fn artifact_id(&self) -> EvidenceId {
        self.artifact_id
    }

    /// Return the canonical content digest after integrity validation.
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest {
        self.content_digest
    }

    /// Return the exact validated wire bytes.
    #[must_use]
    pub fn content(&self) -> &[u8] {
        &self.content
    }

    /// Return the validated wire content size in bytes.
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.content.len()
    }

    /// Return whether `candidate` has the validated wire digest.
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
