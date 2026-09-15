//! Versioned JSON wire contracts for immutable evidence records.

use crate::{
    ContentDigest, DocumentRecord, EvidenceError, EvidenceId, PageLocation, SourceArtifact,
    SourceSpan, ValidatedDocumentRecordWire, ValidatedSourceArtifactWire,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// The only JSON wire-schema version accepted by this crate.
pub const WIRE_SCHEMA_VERSION: u16 = 1;

// A canonical JSON array encodes each u8 as at most three digits plus one comma.
const JSON_U8_ARRAY_WORST_CASE_BYTES_PER_CONTENT_BYTE: usize = 4;
// One UTF-8 input byte can expand to at most one six-byte `\u00XX` JSON escape.
const JSON_STRING_WORST_CASE_BYTES_PER_TEXT_BYTE: usize = 6;
// Fixed schema keys, UUIDv7, digest, punctuation, and a small future-version margin.
const EVIDENCE_WIRE_METADATA_ALLOWANCE_BYTES: usize = 256;
// Source-span wire contains only fixed-width identifiers, coordinates, and optional page geometry.
// A fixed cap keeps malformed external envelopes bounded before serde allocates String fields.
const SOURCE_SPAN_WIRE_BYTE_LIMIT: usize = 4 * 1024;

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
) -> Result<ValidatedSourceArtifactWire, EvidenceError> {
    validate_raw_wire_size(
        payload,
        maximum_bytes,
        JSON_U8_ARRAY_WORST_CASE_BYTES_PER_CONTENT_BYTE,
    )?;
    let wire: SourceArtifactWire = deserialize_wire(payload)?;
    let canonical = serialize_wire(&wire)?;
    validate_version(wire.schema_version)?;
    let artifact_id = EvidenceId::from_str(&wire.artifact_id)?;
    let content_digest = ContentDigest::from_str(&wire.content_sha256)?;
    let validated = ValidatedSourceArtifactWire::from_wire_parts(
        artifact_id,
        content_digest,
        wire.content_bytes,
        maximum_bytes,
    )?;
    if canonical != payload {
        return Err(EvidenceError::InvalidWirePayload);
    }
    Ok(validated)
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
) -> Result<ValidatedDocumentRecordWire, EvidenceError> {
    validate_raw_wire_size(
        payload,
        maximum_bytes,
        JSON_STRING_WORST_CASE_BYTES_PER_TEXT_BYTE,
    )?;
    let wire: DocumentRecordWire = deserialize_wire(payload)?;
    let canonical = serialize_wire(&wire)?;
    validate_version(wire.schema_version)?;
    let document_id = EvidenceId::from_str(&wire.document_id)?;
    let source_artifact_id = EvidenceId::from_str(&wire.source_artifact_id)?;
    let content_digest = ContentDigest::from_str(&wire.content_sha256)?;
    let validated = ValidatedDocumentRecordWire::from_wire_parts(
        document_id,
        source_artifact_id,
        content_digest,
        wire.text,
        maximum_bytes,
    )?;
    if canonical != payload {
        return Err(EvidenceError::InvalidWirePayload);
    }
    Ok(validated)
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
    if payload.len() > SOURCE_SPAN_WIRE_BYTE_LIMIT {
        return Err(EvidenceError::InvalidWirePayload);
    }
    let wire: SourceSpanWire = deserialize_wire(payload)?;
    let canonical = serialize_wire(&wire)?;
    validate_version(wire.schema_version)?;
    let document_id = EvidenceId::from_str(&wire.document_id)?;
    if document_id != document.id() {
        return Err(EvidenceError::SpanDocumentMismatch);
    }
    let page_location = wire.page_location.map(PageLocation::try_from).transpose()?;
    let span = SourceSpan::new(
        document,
        wire.byte_start,
        wire.byte_end,
        wire.scalar_start,
        wire.scalar_end,
        page_location,
    )?;
    if canonical != payload {
        return Err(EvidenceError::InvalidWirePayload);
    }
    Ok(span)
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

fn validate_raw_wire_size(
    payload: &str,
    maximum_content_bytes: usize,
    expansion_factor: usize,
) -> Result<(), EvidenceError> {
    let maximum_wire_bytes = maximum_content_bytes
        .saturating_mul(expansion_factor)
        .saturating_add(EVIDENCE_WIRE_METADATA_ALLOWANCE_BYTES);
    if payload.len() > maximum_wire_bytes {
        Err(EvidenceError::InvalidWirePayload)
    } else {
        Ok(())
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
    use super::{
        deserialize_wire, preflight_document_wire, preflight_source_artifact_wire, serialize_wire,
        validate_version,
    };
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

    #[test]
    fn artifact_preflight_bounds_decoded_byte_count_before_vec_deserialization() {
        let payload = r#"{"schema_version":1,"artifact_id":"id","content_sha256":"digest","content_bytes":[0,1,2]}"#;
        assert_eq!(preflight_source_artifact_wire(payload, 3), Ok(()));
        assert_eq!(
            preflight_source_artifact_wire(payload, 2),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn document_preflight_bounds_decoded_utf8_bytes_including_escapes() {
        let payload = r#"{"schema_version":1,"document_id":"doc","source_artifact_id":"source","content_sha256":"digest","text":"\u00e9\n"}"#;
        assert_eq!(preflight_document_wire(payload, 3), Ok(()));
        assert_eq!(
            preflight_document_wire(payload, 2),
            Err(EvidenceError::InvalidWirePayload)
        );
    }
}
