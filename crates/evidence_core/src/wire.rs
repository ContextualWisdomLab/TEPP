//! Versioned JSON wire contracts for immutable evidence records.

use crate::{
    ContentDigest, DocumentRecord, EvidenceError, EvidenceId, PageLocation, SourceArtifact,
    SourceSpan, ValidatedDocumentRecordWire, ValidatedSourceArtifactWire,
};
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::fmt;
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceArtifactWirePreflight<'payload> {
    schema_version: u16,
    #[serde(borrow)]
    artifact_id: &'payload RawValue,
    #[serde(borrow)]
    content_sha256: &'payload RawValue,
    #[serde(borrow)]
    content_bytes: &'payload RawValue,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentRecordWirePreflight<'payload> {
    schema_version: u16,
    #[serde(borrow)]
    document_id: &'payload RawValue,
    #[serde(borrow)]
    source_artifact_id: &'payload RawValue,
    #[serde(borrow)]
    content_sha256: &'payload RawValue,
    #[serde(borrow)]
    text: &'payload RawValue,
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

struct BoundedByteArraySeed {
    maximum_bytes: usize,
}

struct BoundedByteArrayVisitor {
    maximum_bytes: usize,
}

impl<'de> DeserializeSeed<'de> for BoundedByteArraySeed {
    type Value = bool;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(BoundedByteArrayVisitor {
            maximum_bytes: self.maximum_bytes,
        })
    }
}

impl<'de> Visitor<'de> for BoundedByteArrayVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a byte array within the configured Evidence limit")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut decoded_bytes = 0_usize;
        while sequence.next_element::<u8>()?.is_some() {
            decoded_bytes = decoded_bytes.saturating_add(1);
        }
        Ok(decoded_bytes > self.maximum_bytes)
    }
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
    preflight_source_artifact_wire(payload, maximum_bytes)?;
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
    preflight_document_wire(payload, maximum_bytes)?;
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

fn preflight_source_artifact_wire(
    payload: &str,
    maximum_bytes: usize,
) -> Result<(), EvidenceError> {
    let wire: SourceArtifactWirePreflight<'_> = deserialize_wire(payload)?;
    validate_version(wire.schema_version)?;
    validate_metadata_raw_value(wire.artifact_id.get())?;
    validate_metadata_raw_value(wire.content_sha256.get())?;
    validate_bounded_u8_array_raw(wire.content_bytes.get(), maximum_bytes)
}

fn preflight_document_wire(payload: &str, maximum_bytes: usize) -> Result<(), EvidenceError> {
    let wire: DocumentRecordWirePreflight<'_> = deserialize_wire(payload)?;
    validate_version(wire.schema_version)?;
    validate_metadata_raw_value(wire.document_id.get())?;
    validate_metadata_raw_value(wire.source_artifact_id.get())?;
    validate_metadata_raw_value(wire.content_sha256.get())?;
    validate_bounded_json_string_raw(wire.text.get(), maximum_bytes)
}

fn validate_metadata_raw_value(raw: &str) -> Result<(), EvidenceError> {
    if raw.len() > EVIDENCE_WIRE_METADATA_ALLOWANCE_BYTES {
        Err(EvidenceError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

fn validate_bounded_u8_array_raw(raw: &str, maximum_bytes: usize) -> Result<(), EvidenceError> {
    let mut deserializer = serde_json::Deserializer::from_str(raw);
    let exceeded_limit = BoundedByteArraySeed { maximum_bytes }
        .deserialize(&mut deserializer)
        .map_err(|_| EvidenceError::InvalidWirePayload)?;
    deserializer
        .end()
        .map_err(|_| EvidenceError::InvalidWirePayload)?;
    if exceeded_limit {
        Err(EvidenceError::SourceArtifactTooLarge)
    } else {
        Ok(())
    }
}

fn validate_bounded_json_string_raw(raw: &str, maximum_bytes: usize) -> Result<(), EvidenceError> {
    let Some(inner) = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        return Err(EvidenceError::InvalidWirePayload);
    };
    let bytes = inner.as_bytes();
    let mut index = 0_usize;
    let mut decoded_bytes = 0_usize;
    let mut exceeded_limit = false;
    while index < bytes.len() {
        let (width, next_index) = if bytes[index] == b'\\' {
            decoded_escape_width(bytes, index)?
        } else {
            (1_usize, index + 1)
        };
        decoded_bytes = decoded_bytes.saturating_add(width);
        exceeded_limit |= decoded_bytes > maximum_bytes;
        index = next_index;
    }
    if exceeded_limit {
        Err(EvidenceError::DocumentTooLarge)
    } else {
        Ok(())
    }
}

fn decoded_escape_width(bytes: &[u8], slash_index: usize) -> Result<(usize, usize), EvidenceError> {
    let escape = *bytes
        .get(slash_index + 1)
        .ok_or(EvidenceError::InvalidWirePayload)?;
    match escape {
        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => Ok((1, slash_index + 2)),
        b'u' => decoded_unicode_escape_width(bytes, slash_index),
        _ => Err(EvidenceError::InvalidWirePayload),
    }
}

fn decoded_unicode_escape_width(
    bytes: &[u8],
    slash_index: usize,
) -> Result<(usize, usize), EvidenceError> {
    let first_end = slash_index.saturating_add(6);
    let first = parse_hex_quad(
        bytes
            .get(slash_index + 2..first_end)
            .ok_or(EvidenceError::InvalidWirePayload)?,
    )?;
    if (0xD800..=0xDBFF).contains(&first) {
        let second_end = first_end.saturating_add(6);
        let second_escape = bytes
            .get(first_end..second_end)
            .ok_or(EvidenceError::InvalidWirePayload)?;
        if !second_escape.starts_with(b"\\u") {
            return Err(EvidenceError::InvalidWirePayload);
        }
        let second = parse_hex_quad(&second_escape[2..])?;
        if !(0xDC00..=0xDFFF).contains(&second) {
            return Err(EvidenceError::InvalidWirePayload);
        }
        return Ok((4, second_end));
    }
    let scalar = char::from_u32(u32::from(first)).ok_or(EvidenceError::InvalidWirePayload)?;
    Ok((scalar.len_utf8(), first_end))
}

fn parse_hex_quad(bytes: &[u8]) -> Result<u16, EvidenceError> {
    if bytes.len() != 4 {
        return Err(EvidenceError::InvalidWirePayload);
    }
    let digits = std::str::from_utf8(bytes).map_err(|_| EvidenceError::InvalidWirePayload)?;
    u16::from_str_radix(digits, 16).map_err(|_| EvidenceError::InvalidWirePayload)
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
        EVIDENCE_WIRE_METADATA_ALLOWANCE_BYTES, decoded_escape_width, deserialize_wire,
        parse_hex_quad, preflight_document_wire, preflight_source_artifact_wire, serialize_wire,
        validate_bounded_json_string_raw, validate_bounded_u8_array_raw,
        validate_metadata_raw_value, validate_version,
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
            Err(EvidenceError::SourceArtifactTooLarge)
        );
        assert_eq!(
            preflight_source_artifact_wire("{}", 3),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn document_preflight_bounds_decoded_utf8_bytes_including_escapes() {
        let payload = r#"{"schema_version":1,"document_id":"doc","source_artifact_id":"source","content_sha256":"digest","text":"\u00e9\n"}"#;
        assert_eq!(preflight_document_wire(payload, 3), Ok(()));
        assert_eq!(
            preflight_document_wire(payload, 2),
            Err(EvidenceError::DocumentTooLarge)
        );
        assert_eq!(
            preflight_document_wire("{}", 3),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn byte_array_preflight_fails_closed_without_allocating_a_vec() {
        assert_eq!(validate_bounded_u8_array_raw("[0,1]", 2), Ok(()));
        assert_eq!(
            validate_bounded_u8_array_raw("[0,1]", 1),
            Err(EvidenceError::SourceArtifactTooLarge)
        );
        assert_eq!(
            validate_bounded_u8_array_raw("[0,1,256]", 1),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_u8_array_raw("[256]", 1),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_u8_array_raw("\"bytes\"", 8),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_u8_array_raw("[0] 1", 1),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn text_preflight_handles_plain_simple_unicode_and_surrogate_escapes() {
        assert_eq!(validate_bounded_json_string_raw("\"\"", 0), Ok(()));
        assert_eq!(validate_bounded_json_string_raw("\"abc\"", 3), Ok(()));
        assert_eq!(validate_bounded_json_string_raw(r#""\n""#, 1), Ok(()));
        assert_eq!(validate_bounded_json_string_raw(r#""\u00e9""#, 2), Ok(()));
        assert_eq!(
            validate_bounded_json_string_raw(r#""\uD83D\uDE00""#, 4),
            Ok(())
        );
        assert_eq!(validate_bounded_json_string_raw(r#""é""#, 2), Ok(()));
        assert_eq!(
            validate_bounded_json_string_raw("\"abc\"", 2),
            Err(EvidenceError::DocumentTooLarge)
        );
    }

    #[test]
    fn text_preflight_rejects_non_strings_and_invalid_escape_shapes() {
        assert_eq!(
            validate_bounded_json_string_raw("123", 3),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_json_string_raw("\"abc", 3),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_json_string_raw(r#""\q""#, 1),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_json_string_raw(r#""ab\q""#, 1),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_json_string_raw(r#""\uDC00""#, 3),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_json_string_raw(r#""\uD83D""#, 4),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_json_string_raw(r#""\uD83Dabcdef""#, 4),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_json_string_raw(r#""\uD83D\u0041""#, 4),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            validate_bounded_json_string_raw(r#""\u00G0""#, 3),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            decoded_escape_width(b"\\", 0),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn metadata_preflight_caps_caller_controlled_string_values() {
        assert_eq!(validate_metadata_raw_value("\"id\""), Ok(()));
        let oversized = "x".repeat(EVIDENCE_WIRE_METADATA_ALLOWANCE_BYTES + 1);
        assert_eq!(
            validate_metadata_raw_value(&oversized),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn hex_quad_parser_rejects_wrong_width_and_non_hex_digits() {
        assert_eq!(parse_hex_quad(b"00e9"), Ok(0x00e9));
        assert_eq!(
            parse_hex_quad(b"abc"),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            parse_hex_quad(b"00G0"),
            Err(EvidenceError::InvalidWirePayload)
        );
        assert_eq!(
            parse_hex_quad(&[0xff, b'0', b'0', b'0']),
            Err(EvidenceError::InvalidWirePayload)
        );
    }
}
