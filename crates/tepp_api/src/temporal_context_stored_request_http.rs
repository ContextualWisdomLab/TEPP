//! Provider-owned temporal-context stored-request GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/temporal-context/{idempotency_key}/request`
//! returns the accepted `LineageWeave` create request on `AnalysisRunLiveService`
//! / `tepp-loopback` so operators who hold a retrieval identity do not replay
//! POST. `inference_status` on the live projection remains
//! `temporal_association_only`. `tepp.scientific_acceptance.v1` never appears.
//! This module does not duplicate GET-by-id (#451), retrieval CLI (#452),
//! temporal-context CLI (#414), collection GET/CLI (#449/#450 closed),
//! project-history stored-request GET (#455), interpretation-run stored-request
//! GET (#453), export stored-request GET (#457), cancel lineages (closed),
//! Leiden, or GAP-010 Figma/export. Persistence remains GAP-003B. Naruon is
//! refused. `NaruonLiveService` stays POST-only.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::temporal_context_retrieval_http::{
    TEMPORAL_CONTEXT_RETRIEVAL_ID_MAX_LEN, validate_temporal_context_registry_identity,
};
use crate::wire::require_nonempty;
use crate::{ApiError, TEMPORAL_CONTEXT_PATH};

const FORBIDDEN_STORED_REQUEST_KEYS: [&str; 12] = [
    "rmse",
    "rmse_standard_error",
    "mean_bias",
    "bias_standard_error",
    "interval_coverage",
    "coverage_wilson_lower",
    "coverage_wilson_upper",
    "temporal_order_accuracy",
    "se_gate_accepted",
    "se_gate_k",
    "scientific_acceptance",
    "causal_score",
];

/// Extract the opaque idempotency key from
/// `GET /v1/temporal-context/{key}/request`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for collection, GET-by-id, extra
/// segments, a hostile encoding, or an empty identity, and
/// [`ApiError::LimitExceeded`] when oversized.
pub fn temporal_context_stored_request_path_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(TEMPORAL_CONTEXT_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    let (encoded_id, rest) = encoded
        .split_once('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    if rest != "request" || encoded_id.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    let idempotency_key = decode_path_segment(encoded_id)?;
    require_nonempty(&idempotency_key)?;
    if idempotency_key.contains('/') || idempotency_key.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if idempotency_key.len() > TEMPORAL_CONTEXT_RETRIEVAL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(idempotency_key)
}

/// Whether `path` is the stored-request extra-segment resource.
#[must_use]
pub fn is_temporal_context_stored_request_path(path: &str) -> bool {
    temporal_context_stored_request_path_id(path).is_ok()
}

/// Refuse stored-request JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted for the GET request body. The original create
/// request may carry `event_label` and `actor_references`; those keys are not
/// scientific metrics.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric or causal
/// key is present.
pub fn refuse_metrics_on_temporal_context_stored_request_payload(
    payload: &str,
) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    if !value.is_object() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_metrics_on_json(&value)
}

fn refuse_metrics_on_json(value: &serde_json::Value) -> Result<(), ApiError> {
    match value {
        serde_json::Value::Object(object) => {
            if object
                .get("schema_version")
                .and_then(serde_json::Value::as_str)
                == Some("tepp.scientific_acceptance.v1")
            {
                return Err(ApiError::InvalidWirePayload);
            }
            if FORBIDDEN_STORED_REQUEST_KEYS
                .iter()
                .any(|key| object.contains_key(*key))
            {
                return Err(ApiError::InvalidWirePayload);
            }
            for nested in object.values() {
                refuse_metrics_on_json(nested)?;
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for nested in items {
                refuse_metrics_on_json(nested)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Build a credential-free `LineageWeave` stored-request GET exchange.
///
/// # Errors
///
/// Returns a fail-closed origin or identity error.
pub fn lineageweave_temporal_context_stored_request_exchange(
    origin: &str,
    idempotency_key: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    validate_temporal_context_registry_identity(idempotency_key)?;
    if idempotency_key.contains('/') || idempotency_key.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if idempotency_key.len() > TEMPORAL_CONTEXT_RETRIEVAL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_id = encode_path_segment(idempotency_key);
    let target_path = format!("{TEMPORAL_CONTEXT_PATH}/{encoded_id}/request");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "GET",
        target_url,
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "lineageweave".into()),
            ("tepp-contract-version".into(), "1".into()),
        ],
        body: String::new(),
    })
}

fn encode_path_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + value.len() / 2);
    let hex = b"0123456789ABCDEF";
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(hex[usize::from(byte >> 4)] as char);
                out.push(hex[usize::from(byte & 0x0F)] as char);
            }
        }
    }
    out
}

fn decode_path_segment(value: &str) -> Result<String, ApiError> {
    let mut out = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(ApiError::InvalidWirePayload);
                }
                let hi = from_hex(bytes[index + 1])?;
                let lo = from_hex(bytes[index + 2])?;
                out.push((hi << 4) | lo);
                index += 3;
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(bytes[index]);
                index += 1;
            }
            _ => return Err(ApiError::InvalidWirePayload),
        }
    }
    let decoded = String::from_utf8(out).map_err(|_| ApiError::InvalidWirePayload)?;
    if decoded.chars().any(char::is_control) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(decoded)
}

fn from_hex(byte: u8) -> Result<u8, ApiError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(ApiError::InvalidWirePayload),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_temporal_context_stored_request_path,
        lineageweave_temporal_context_stored_request_exchange,
        refuse_metrics_on_temporal_context_stored_request_payload,
        temporal_context_stored_request_path_id,
    };
    use crate::ApiError;

    #[test]
    fn stored_request_exchange_is_lineageweave_get_without_credentials() {
        let exchange = lineageweave_temporal_context_stored_request_exchange(
            "https://tepp.example.test",
            "idem-a",
        )
        .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(
            exchange
                .target_url
                .ends_with("/v1/temporal-context/idem-a/request")
        );
        assert!(exchange.body.is_empty());
        assert!(
            !exchange
                .headers
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                    || name.eq_ignore_ascii_case("idempotency-key"))
        );
        assert!(is_temporal_context_stored_request_path(
            "/v1/temporal-context/idem-a/request"
        ));
        assert!(!is_temporal_context_stored_request_path(
            "/v1/temporal-context/idem-a"
        ));
        assert_eq!(
            temporal_context_stored_request_path_id("/v1/temporal-context/idem-a/request")
                .expect("id"),
            "idem-a"
        );
        assert_eq!(
            temporal_context_stored_request_path_id("/v1/temporal-context/idem-a"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            temporal_context_stored_request_path_id("/v1/temporal-context/idem-a/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            lineageweave_temporal_context_stored_request_exchange(
                "http://tepp.example.test",
                "idem-a"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_temporal_context_stored_request_payload(""),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_temporal_context_stored_request_payload(r#"{"rmse":1.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
