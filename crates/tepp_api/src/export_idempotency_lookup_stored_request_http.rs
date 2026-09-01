//! Export idempotency-key stored-request lookup contracts.
//!
//! `GET /v1/exports/by-idempotency/{idempotency_key}/request` was introduced as
//! a convenience lookup for an accepted Naruon export authorization. Review of
//! the first implementation showed that consumer-only lookup could search all
//! Naruon tenant namespaces and return the original request, including tenant
//! and principal identity. The route is therefore fail-closed until the
//! Analysis Run boundary has an explicit tenant-and-principal authorization
//! binding. The parser remains available so the live dispatcher can recognize
//! and reject the reserved resource deterministically; the client exchange
//! builder also refuses activation. `LineageWeave` remains outside this
//! Naruon-owned adapter and `tepp.scientific_acceptance.v1` is never admitted.

use crate::ApiError;
use crate::export_idempotency_lookup_http::{
    EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN, EXPORT_IDEMPOTENCY_LOOKUP_PREFIX,
};
use crate::naruon_http::{NARUON_EXPORT_PATH, NaruonHttpExchange, compose_https_target};
use crate::wire::require_nonempty;

/// Extra-segment that names the stored export-authorization request.
pub const EXPORT_IDEMPOTENCY_LOOKUP_STORED_REQUEST_SEGMENT: &str = "request";

const FORBIDDEN_STORED_REQUEST_KEYS: [&str; 15] = [
    "tenant_workspace_id",
    "principal_id",
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

/// Extract the opaque idempotency key from the reserved stored-request route.
///
/// Raw and percent-decoded slashes are rejected. Keeping the key in one route
/// segment avoids ambiguous normalization between proxies and the loopback
/// dispatcher.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for collection, GET-by-id, lookup
/// without `/request`, `{export_id}/request`, extra raw segments, a missing
/// `by-idempotency` prefix, reserved prefix used as the key, slash, NUL, empty
/// key, or hostile encoding, and [`ApiError::LimitExceeded`] when oversized.
pub fn export_idempotency_lookup_stored_request_path_key(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(NARUON_EXPORT_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_prefix(EXPORT_IDEMPOTENCY_LOOKUP_PREFIX)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    let (encoded_key, rest) = encoded
        .split_once('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    if rest != EXPORT_IDEMPOTENCY_LOOKUP_STORED_REQUEST_SEGMENT || encoded_key.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    let key = decode_path_segment(encoded_key)?;
    require_nonempty(&key)?;
    if key == EXPORT_IDEMPOTENCY_LOOKUP_PREFIX || key.contains('/') || key.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if key.len() > EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(key)
}

/// Whether `path` is the lookup stored-request extra-segment resource.
#[must_use]
pub fn is_export_idempotency_lookup_stored_request_path(path: &str) -> bool {
    export_idempotency_lookup_stored_request_path_key(path).is_ok()
}

/// Refuse stored-request JSON that carries sensitive identity or scientific keys.
///
/// Empty payloads are admitted for the GET request body. A serialized
/// [`crate::ExportAuthorizationRequest`] contains tenant and principal identity,
/// so it is intentionally rejected while this route lacks caller scope binding.
/// This gives the existing live dispatcher a fail-closed quarantine without
/// weakening the separate metric-free export identity lookup.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden identity,
/// scientific metric, report, or terminal-result key is present.
pub fn refuse_metrics_on_export_lookup_stored_request_payload(
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

/// Validate a would-be Naruon lookup stored-request GET and fail closed.
///
/// No exchange is emitted until the service can bind the lookup to both the
/// authorized tenant/workspace and principal. Valid origin/key syntax is still
/// checked so malformed callers receive the existing deterministic validation
/// errors instead of using quarantine as an input-validation bypass.
///
/// # Errors
///
/// Returns a fail-closed origin/identity error for invalid inputs and
/// [`ApiError::AuthorizationDenied`] for otherwise valid requests while the
/// route is quarantined.
pub fn naruon_export_idempotency_lookup_stored_request_exchange(
    origin: &str,
    idempotency_key: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    require_nonempty(idempotency_key)?;
    if idempotency_key == EXPORT_IDEMPOTENCY_LOOKUP_PREFIX
        || idempotency_key.contains('/')
        || idempotency_key.contains('\0')
    {
        return Err(ApiError::InvalidWirePayload);
    }
    if idempotency_key.len() > EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_key = encode_path_segment(idempotency_key);
    let target_path = format!(
        "{NARUON_EXPORT_PATH}/{EXPORT_IDEMPOTENCY_LOOKUP_PREFIX}/{encoded_key}/{EXPORT_IDEMPOTENCY_LOOKUP_STORED_REQUEST_SEGMENT}"
    );
    let _validated_target = compose_https_target(origin, &target_path)?;
    Err(ApiError::AuthorizationDenied)
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
    if decoded.is_empty() || decoded.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(decoded)
}

fn from_hex(byte: u8) -> Result<u8, ApiError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(ApiError::InvalidWirePayload),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EXPORT_IDEMPOTENCY_LOOKUP_STORED_REQUEST_SEGMENT,
        export_idempotency_lookup_stored_request_path_key,
        is_export_idempotency_lookup_stored_request_path,
        naruon_export_idempotency_lookup_stored_request_exchange,
        refuse_metrics_on_export_lookup_stored_request_payload,
    };
    use crate::ApiError;
    use crate::export_http::export_retrieval_path_id;
    use crate::export_idempotency_lookup_http::{
        EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN, EXPORT_IDEMPOTENCY_LOOKUP_PREFIX,
        export_idempotency_lookup_path_key,
    };

    #[test]
    fn lookup_stored_request_route_is_recognized_but_client_activation_is_quarantined() {
        assert_eq!(
            naruon_export_idempotency_lookup_stored_request_exchange(
                "https://tepp.example.test",
                "idem-9",
            ),
            Err(ApiError::AuthorizationDenied)
        );
        assert!(is_export_idempotency_lookup_stored_request_path(
            "/v1/exports/by-idempotency/idem-9/request"
        ));
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/idem-9/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports/by-idempotency/idem-9/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key(
                "/v1/exports/by-idempotency/idem-9/request"
            )
            .expect("key"),
            "idem-9"
        );
        assert_eq!(EXPORT_IDEMPOTENCY_LOOKUP_STORED_REQUEST_SEGMENT, "request");
        assert_eq!(EXPORT_IDEMPOTENCY_LOOKUP_PREFIX, "by-idempotency");
        assert_eq!(
            refuse_metrics_on_export_lookup_stored_request_payload(""),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_export_lookup_stored_request_payload(
                r#"{"tenant_workspace_id":"tenant-a","principal_id":"principal-a","artifact_id":"artifact-a"}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn lookup_stored_request_path_and_origins_fail_closed() {
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key("/v1/exports/by-idempotency/idem-9"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key("/v1/exports/idem-9/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key("/v1/exports/by-idempotency/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key(
                "/v1/exports/by-idempotency/idem-9/request/extra"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key(
                "/v1/exports/by-idempotency/idem-9/cancel"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key(
                "/v1/exports/by-idempotency/by-idempotency/request"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key(
                "/v1/exports/by-idempotency/%00/request"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key(
                "/v1/exports/by-idempotency/idem%2F9/request"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_stored_request_path_key(&format!(
                "/v1/exports/by-idempotency/{}/request",
                "a".repeat(EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            naruon_export_idempotency_lookup_stored_request_exchange(
                "http://tepp.example.test",
                "idem-9",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_export_idempotency_lookup_stored_request_exchange(
                "https://db.postgres.example",
                "idem-9",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_export_idempotency_lookup_stored_request_exchange(
                "https://tepp.example.test",
                "by-idempotency",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_export_idempotency_lookup_stored_request_exchange(
                "https://tepp.example.test",
                "idem/9",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_export_lookup_stored_request_payload(r#"{"rmse":1.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
