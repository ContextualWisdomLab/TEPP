//! Immutable UTF-8 document records linked to source artifacts.

use crate::{ContentDigest, EvidenceError, EvidenceId, SourceArtifactId};
use std::sync::Arc;

const DEFAULT_DOCUMENT_BYTE_LIMIT: usize = 16 * 1024 * 1024;

/// An immutable UTF-8 document derived from one Evidence-issued source artifact.
///
/// Owner creation requires [`SourceArtifactId`], a handle that cannot be parsed
/// or constructed from an arbitrary [`EvidenceId`]. This keeps source provenance
/// nominal at the aggregate boundary without copying source bytes into the
/// document record.
///
/// ```compile_fail
/// use evidence_core::{DocumentRecord, EvidenceId};
/// let arbitrary = EvidenceId::new();
/// let _ = DocumentRecord::from_text(arbitrary, "document");
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRecord {
    id: EvidenceId,
    source_artifact_id: EvidenceId,
    content_digest: ContentDigest,
    text: Arc<str>,
    scalar_length: usize,
}

/// Canonical, structurally validated document wire input.
///
/// Validation proves UUIDv7 syntax, UTF-8 text integrity, the declared SHA-256,
/// size bounds, and canonical JSON. It does **not** authenticate either the
/// caller-supplied document identity or source-artifact identity and therefore
/// cannot be promoted directly into [`DocumentRecord`]. Authenticated persistence
/// replay must be owned by a future Evidence repository boundary.
///
/// ```compile_fail
/// use evidence_core::{DocumentRecord, ValidatedDocumentRecordWire};
/// let wire = ValidatedDocumentRecordWire::from_json("{}").expect("validated wire");
/// let _: DocumentRecord = wire;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedDocumentRecordWire {
    document_id: EvidenceId,
    source_artifact_id: EvidenceId,
    content_digest: ContentDigest,
    text: Arc<str>,
    scalar_length: usize,
}

impl DocumentRecord {
    /// Copy UTF-8 text into an immutable owner record using the default limit.
    ///
    /// `source_artifact_id` can only originate from a trusted [`crate::SourceArtifact`].
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::EmptyDocument`] for empty text or
    /// [`EvidenceError::DocumentTooLarge`] when the default limit is exceeded.
    pub fn from_text(
        source_artifact_id: SourceArtifactId,
        text: impl AsRef<str>,
    ) -> Result<Self, EvidenceError> {
        Self::from_text_with_limit(source_artifact_id, text, DEFAULT_DOCUMENT_BYTE_LIMIT)
    }

    /// Copy UTF-8 text into an immutable owner record bounded by `maximum_bytes`.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::EmptyDocument`] for empty text or
    /// [`EvidenceError::DocumentTooLarge`] when `maximum_bytes` is exceeded.
    pub fn from_text_with_limit(
        source_artifact_id: SourceArtifactId,
        text: impl AsRef<str>,
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        let text = text.as_ref();
        validate_text(text, maximum_bytes)?;

        Ok(Self {
            id: EvidenceId::new(),
            source_artifact_id: source_artifact_id.evidence_id(),
            content_digest: ContentDigest::sha256(text.as_bytes()),
            scalar_length: text.chars().count(),
            text: Arc::from(text),
        })
    }

    /// Serialize this trusted document through the strict versioned JSON wire contract.
    ///
    /// Serialization does not authenticate a later parse. Generic JSON parsing
    /// returns [`ValidatedDocumentRecordWire`], not this owner-issued type.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] if JSON serialization
    /// cannot represent the validated document.
    pub fn to_wire_json(&self) -> Result<String, EvidenceError> {
        crate::wire::serialize_document(self)
    }

    /// Return the stable document identifier.
    #[must_use]
    pub const fn id(&self) -> EvidenceId {
        self.id
    }

    /// Return the source-artifact identifier committed by the trusted handle.
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

impl ValidatedDocumentRecordWire {
    /// Parse canonical document JSON using the default text-size limit.
    ///
    /// Successful parsing validates structure and content integrity only. It
    /// does not authenticate or promote caller-selected record identities.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, identifier, digest, canonicality, size, or
    /// content-integrity error.
    pub fn from_json(payload: &str) -> Result<Self, EvidenceError> {
        Self::from_json_with_limit(payload, DEFAULT_DOCUMENT_BYTE_LIMIT)
    }

    /// Parse canonical document JSON with an explicit text-size limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, identifier, digest, canonicality, size, or
    /// content-integrity error.
    pub fn from_json_with_limit(
        payload: &str,
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        crate::wire::deserialize_document(payload, maximum_bytes)
    }

    pub(crate) fn from_wire_parts(
        document_id: EvidenceId,
        source_artifact_id: EvidenceId,
        content_digest: ContentDigest,
        text: String,
        maximum_bytes: usize,
    ) -> Result<Self, EvidenceError> {
        validate_text(&text, maximum_bytes)?;
        if ContentDigest::sha256(text.as_bytes()) != content_digest {
            return Err(EvidenceError::ContentDigestMismatch);
        }
        let scalar_length = text.chars().count();

        Ok(Self {
            document_id,
            source_artifact_id,
            content_digest,
            text: Arc::from(text),
            scalar_length,
        })
    }

    /// Return the caller-supplied document identifier after structural validation.
    #[must_use]
    pub const fn document_id(&self) -> EvidenceId {
        self.document_id
    }

    /// Return the caller-supplied source-artifact identifier after structural validation.
    #[must_use]
    pub const fn source_artifact_id(&self) -> EvidenceId {
        self.source_artifact_id
    }

    /// Return the canonical text digest after integrity validation.
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest {
        self.content_digest
    }

    /// Return the exact validated UTF-8 text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the validated UTF-8 byte length.
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.text.len()
    }

    /// Return the validated Unicode-scalar length.
    #[must_use]
    pub const fn scalar_length(&self) -> usize {
        self.scalar_length
    }

    /// Return whether `candidate` has the validated wire text digest.
    #[must_use]
    pub fn verify_text(&self, candidate: impl AsRef<str>) -> bool {
        ContentDigest::sha256(candidate.as_ref().as_bytes()) == self.content_digest
    }
}

fn validate_text(text: &str, maximum_bytes: usize) -> Result<(), EvidenceError> {
    if text.is_empty() {
        return Err(EvidenceError::EmptyDocument);
    }
    if text.len() > maximum_bytes {
        return Err(EvidenceError::DocumentTooLarge);
    }
    Ok(())
}
