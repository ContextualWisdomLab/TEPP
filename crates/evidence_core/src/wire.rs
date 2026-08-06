//! Versioned JSON wire contracts for immutable evidence records.

use crate::{
    ContentDigest, DocumentRecord, EvidenceError, EvidenceId, PageLocation, SourceArtifact,
    SourceSpan,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// The only JSON wire-schema version accepted by this crate.
pub const WIRE_SCHEMA_VERSION: u16 = 1;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceArtifactWire {
    schema_version: u16,
    artifact_id: String,
    content_sha256: String,
    content_bytes: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DocumentRecordWire {
    schema_version: u16,
    document_id: String,
    source_artifact_id: String,
    content_sha256: String,
    text: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceSpanWire {
    schema_version: u16,
    document_id: String,
    byte_start: usize,
    byte_end: usize,
    scalar_start: usize,
    scalar_end: usize,
    page_location: Option<PageLocationWire>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PageLocationWire {
    page_number: u32,
    page_width: f64,
    page_height: f64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

pub(crate) fn serialize_source_artifact(
    artifact: &SourceArtifact,
) -> Result<String, EvidenceError> {
    serialize_wire(&SourceArtifactWire {
        schema_version: WIRE_SCHEMA_VERSION,
        artifact_id: artifact.id().to_string(),
        content_sha256: artifact.content_digest().to_string(),
        content_bytes: artifact.content().to_vec(),
    })
}

pub(crate) fn deserialize_source_artifact(
    payload: &str,
    maximum_bytes: usize,
) -> Result<SourceArtifact, EvidenceError> {
    let wire: SourceArtifactWire = deserialize_wire(payload)?;
    validate_version(wire.schema_version)?;
    let artifact_id = EvidenceId::from_str(&wire.artifact_id)?;
    let content_digest = ContentDigest::from_str(&wire.content_sha256)?;
    SourceArtifact::from_wire_parts(
        artifact_id,
        content_digest,
        wire.content_bytes,
        maximum_bytes,
    )
}

pub(crate) fn serialize_document(document: &DocumentRecord) -> Result<String, EvidenceError> {
    serialize_wire(&DocumentRecordWire {
        schema_version: WIRE_SCHEMA_VERSION,
        document_id: document.id().to_string(),
        source_artifact_id: document.source_artifact_id().to_string(),
        content_sha256: document.content_digest().to_string(),
        text: document.text().to_owned(),
    })
}

pub(crate) fn deserialize_document(
    payload: &str,
    maximum_bytes: usize,
) -> Result<DocumentRecord, EvidenceError> {
    let wire: DocumentRecordWire = deserialize_wire(payload)?;
    validate_version(wire.schema_version)?;
    let document_id = EvidenceId::from_str(&wire.document_id)?;
    let source_artifact_id = EvidenceId::from_str(&wire.source_artifact_id)?;
    let content_digest = ContentDigest::from_str(&wire.content_sha256)?;
    DocumentRecord::from_wire_parts(
        document_id,
        source_artifact_id,
        content_digest,
        wire.text,
        maximum_bytes,
    )
}

pub(crate) fn serialize_source_span(span: &SourceSpan) -> Result<String, EvidenceError> {
    serialize_wire(&SourceSpanWire {
        schema_version: WIRE_SCHEMA_VERSION,
        document_id: span.document_id().to_string(),
        byte_start: span.byte_start(),
        byte_end: span.byte_end(),
        scalar_start: span.scalar_start(),
        scalar_end: span.scalar_end(),
        page_location: span.page_location().map(PageLocationWire::from),
    })
}

pub(crate) fn deserialize_source_span(
    payload: &str,
    document: &DocumentRecord,
) -> Result<SourceSpan, EvidenceError> {
    let wire: SourceSpanWire = deserialize_wire(payload)?;
    validate_version(wire.schema_version)?;
    let document_id = EvidenceId::from_str(&wire.document_id)?;
    if document_id != document.id() {
        return Err(EvidenceError::SpanDocumentMismatch);
    }
    let page_location = wire.page_location.map(PageLocation::try_from).transpose()?;
    SourceSpan::new(
        document,
        wire.byte_start,
        wire.byte_end,
        wire.scalar_start,
        wire.scalar_end,
        page_location,
    )
}

impl From<PageLocation> for PageLocationWire {
    fn from(location: PageLocation) -> Self {
        Self {
            page_number: location.page_number(),
            page_width: location.page_width(),
            page_height: location.page_height(),
            x: location.x(),
            y: location.y(),
            width: location.width(),
            height: location.height(),
        }
    }
}

impl TryFrom<PageLocationWire> for PageLocation {
    type Error = EvidenceError;

    fn try_from(wire: PageLocationWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.page_number,
            wire.page_width,
            wire.page_height,
            wire.x,
            wire.y,
            wire.width,
            wire.height,
        )
    }
}

fn validate_version(version: u16) -> Result<(), EvidenceError> {
    if version == WIRE_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(EvidenceError::UnsupportedWireVersion)
    }
}

fn serialize_wire<T: Serialize>(value: &T) -> Result<String, EvidenceError> {
    serde_json::to_string(value).map_err(|_| EvidenceError::InvalidWirePayload)
}

fn deserialize_wire<'payload, T>(payload: &'payload str) -> Result<T, EvidenceError>
where
    T: Deserialize<'payload>,
{
    serde_json::from_str(payload).map_err(|_| EvidenceError::InvalidWirePayload)
}

#[cfg(test)]
mod tests {
    use super::{deserialize_wire, serialize_wire, validate_version};
    use crate::EvidenceError;
    use serde::Serialize;
    use serde::ser::Serializer;

    struct SerializationFailure;

    impl Serialize for SerializationFailure {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("intentional test failure"))
        }
    }

    #[test]
    fn serialization_failures_are_redacted() {
        assert_eq!(
            serialize_wire(&SerializationFailure),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn deserialization_failures_are_redacted() {
        assert_eq!(
            deserialize_wire::<Vec<u8>>("not JSON"),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn supported_and_unsupported_versions_are_distinct() {
        assert_eq!(validate_version(1), Ok(()));
        assert_eq!(
            validate_version(2),
            Err(EvidenceError::UnsupportedWireVersion)
        );
    }
}
