//! Provider-owned export lookup stored-request GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/exports/by-idempotency/{idempotency_key}/request`
//! returns the stored naruon export-authorization request of the unique
//! accepted export that used that client key on `AnalysisRunLiveService` /
//! `tepp-loopback`. Lookup GET returns identity only. Stored-request GET
//! requires `export_id`. Operators who hold a 200 authorization receipt or
//! log key still need two hops. `NaruonLiveService` stays POST-only.
//! `LineageWeave` is refused on this naruon-owned adapter.
//! `tepp.scientific_acceptance.v1` never appears. This module does not
//! duplicate lookup GET/CLI (#465/#466), stored-request GET/CLI (#457/#459),
//! GET-by-id (#411), retrieval CLI (#417), collection GET/CLI (#443/#444),
//! export-authorize CLI (#410), analysis-run lookup (#380), or cancel
//! lineages (closed). Persistence remains GAP-003B. GAP-010 Figma/export
//! remains later work.

use crate::ApiError;
use crate::export_idempotency_lookup_http::{
    EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN, EXPORT_IDEMPOTENCY_LOOKUP_PREFIX,
};
use crate::naruon_http::{NARUON_EXPORT_PATH, NaruonHttpExchange, compose_https_target};
use crate::wire::require_nonempty;

/// Extra-segment that names the stored export-authorization request.
pub const EXPORT_IDEMPOTENCY_LOOKUP_STORED_REQUEST_SEGMENT: &str = "request";

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

/// Extract the opaque idempotency key from
/// `GET /v1/exports/by-idempotency/{idempotency_key}/request`.
///
/// The route is segmented before percent decoding, so an encoded `/` remains
/// data inside one opaque key rather than becoming an extra path segment.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for collection, GET-by-id, lookup
/// without `/request`, `{export_id}/request`, extra raw segments, a missing
/// `by-idempotency` prefix, reserved prefix used as the key, NUL, empty key,
/// or a hostile encoding, and [`ApiError::LimitExceeded`] when oversized.
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
    if key == EXPORT_IDEMPOTENCY_LOOKUP_PREFIX {
        return Err(ApiError::InvalidWirePayload);
    }
    if key.contains('\0') {
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

/// Build a credential-free naruon lookup stored-request GET exchange.
///
/// The builder refuses non-`https` origins and empty or oversized keys. It
/// does not inject credentials. The GET body is empty. The opaque key is
/// percent-encoded into exactly one path segment after `by-idempotency` and
/// before `/request`.
///
/// # Errors
///
/// Returns a fail-closed origin or identity error.
pub fn naruon_export_idempotency_lookup_stored_request_exchange(
    origin: &str,
    idempotency_key: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    require_nonempty(idempotency_key)?;
    if idempotency_key == EXPORT_IDEMPOTENCY_LOOKUP_PREFIX || idempotency_key.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if idempotency_key.len() > EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_key = encode_path_segment(idempotency_key);
    let target_path = format!(
        "{NARUON_EXPORT_PATH}/{EXPORT_IDEMPOTENCY_LOOKUP_PREFIX}/{encoded_key}/{EXPORT_IDEMPOTENCY_LOOKUP_STORED_REQUEST_SEGMENT}"
    );
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
    fn lookup_stored_request_exchange_is_metric_free_get_without_credentials() {
        let exchange = naruon_export_idempotency_lookup_stored_request_exchange(
            "https://tepp.example.test",
            "idem-9",
        )
        .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.test/v1/exports/by-idempotency/idem-9/request"
        );
        assert!(exchange.body.is_empty());
        assert!(
            !exchange
                .headers
                .iter()
                .any(|(name, _)| name.contains("authorization")
                    || name.contains("token")
                    || name.contains("idempotency"))
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
            refuse_metrics_on_export_lookup_stored_request_payload(r#"{"rmse":1.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
