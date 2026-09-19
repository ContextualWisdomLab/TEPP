//! Provider-owned export stored-request GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/exports/{export_id}/request` returns the
//! accepted naruon export-authorization request on `AnalysisRunLiveService`
//! / `tepp-loopback` so operators who hold a retrieval identity do not replay
//! POST. `NaruonLiveService` stays POST-only. `LineageWeave` is refused on this
//! naruon-owned adapter. `tepp.scientific_acceptance.v1` never appears. This
//! module does not duplicate GET-by-id (#411), retrieval CLI (#417),
//! collection GET/CLI (#443/#444), export-authorize CLI (#410), analysis-run
//! stored-request GET (#377), project-history stored-request GET (#455),
//! interpretation-run stored-request GET (#453), or cancel lineages (closed).
//! Persistence remains GAP-003B. GAP-010 Figma/export remains later work.

use crate::export_http::EXPORT_RETRIEVAL_ID_MAX_LEN;
use crate::naruon_http::{compose_https_target, NaruonHttpExchange};
use crate::wire::require_nonempty;
use crate::{ApiError, NARUON_EXPORT_PATH};

const FORBIDDEN_STORED_REQUEST_KEYS: [&str; 13] = [
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
    "report",
    "terminal_result",
];

/// Extract the opaque export identity from
/// `GET /v1/exports/{export_id}/request`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for collection, GET-by-id, extra
/// segments, a hostile encoding, or an empty identity, and
/// [`ApiError::LimitExceeded`] when oversized.
pub fn export_stored_request_path_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(NARUON_EXPORT_PATH)
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
    let export_id = decode_path_segment(encoded_id)?;
    require_nonempty(&export_id)?;
    if export_id.contains('/') || export_id.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if export_id.len() > EXPORT_RETRIEVAL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(export_id)
}

/// Whether `path` is the stored-request extra-segment resource.
#[must_use]
pub fn is_export_stored_request_path(path: &str) -> bool {
    export_stored_request_path_id(path).is_ok()
}

/// Refuse stored-request JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted for the GET request body. The original
/// authorization request may carry `tenant_workspace_id`, `principal_id`, and
/// `includes_source_text`; those keys are not scientific metrics.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present.
pub fn refuse_metrics_on_export_stored_request_payload(payload: &str) -> Result<(), ApiError> {
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

/// Build a credential-free naruon stored-request GET exchange.
///
/// # Errors
///
/// Returns a fail-closed origin or identity error.
pub fn naruon_export_stored_request_exchange(
    origin: &str,
    export_id: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    require_nonempty(export_id)?;
    if export_id.contains('/') || export_id.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if export_id.len() > EXPORT_RETRIEVAL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_id = encode_path_segment(export_id);
    let target_path = format!("{NARUON_EXPORT_PATH}/{encoded_id}/request");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "GET",
        target_url,
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "naruon".into()),
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
        export_stored_request_path_id, is_export_stored_request_path,
        naruon_export_stored_request_exchange, refuse_metrics_on_export_stored_request_payload,
    };
    use crate::ApiError;

    #[test]
    fn stored_request_exchange_is_naruon_get_without_credentials() {
        let exchange =
            naruon_export_stored_request_exchange("https://tepp.example.test", "export-1")
                .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(exchange
            .target_url
            .ends_with("/v1/exports/export-1/request"));
        assert!(exchange.body.is_empty());
        assert!(!exchange
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                || name.eq_ignore_ascii_case("idempotency-key")));
        assert!(is_export_stored_request_path(
            "/v1/exports/export-1/request"
        ));
        assert!(!is_export_stored_request_path("/v1/exports/export-1"));
        assert_eq!(
            export_stored_request_path_id("/v1/exports/export-1/request").expect("id"),
            "export-1"
        );
        assert_eq!(
            export_stored_request_path_id("/v1/exports/export-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_stored_request_path_id("/v1/exports/export-1/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_export_stored_request_exchange("http://tepp.example.test", "export-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(refuse_metrics_on_export_stored_request_payload(""), Ok(()));
        assert_eq!(
            refuse_metrics_on_export_stored_request_payload(r#"{"rmse":1.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
