//! Provider-owned interpretation-run stored-request GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/interpretation-runs/{idempotency_key}/request`
//! returns the accepted metric-free `InterpretationRunRequest` on
//! `OrchestratorLiveService` / `tepp-orchestrator-loopback` so operators who
//! hold a retrieval identity do not replay POST. The stored request stays
//! `scientific_authority=false`. `tepp.scientific_acceptance.v1` never
//! appears. This module does not duplicate GET-by-id (#438), retrieval CLI
//! (#439), collection GET (#433), collection CLI (#436), create CLI (#425),
//! analysis-run stored-request GET (#377), cancel lineages (closed), Leiden,
//! or GAP-010 Figma/export. Persistence remains GAP-003B. Naruon and
//! `LineageWeave` are refused. `NaruonLiveService` stays POST-only.

use crate::error::OrchestratorLiveError;
use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
use crate::interpretation_run_lookup_http::INTERPRETATION_RUN_LOOKUP_PREFIX;
use crate::interpretation_run_retrieval_http::INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN;
use crate::request::{host_implies_table_access, require_nonempty, INTERPRETATION_RUN_PATH};

const FORBIDDEN_STORED_REQUEST_KEYS: [&str; 12] = [
    "rmse",
    "rmse_standard_error",
    "mean_bias",
    "bias_standard_error",
    "interval_coverage",
    "se_gate_accepted",
    "scientific_acceptance",
    "causal_score",
    "findings",
    "evidence_text",
    "report",
    "event_label",
];

/// Typed GET exchange for interpretation-run stored-request retrieval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpretationRunStoredRequestHttpExchange {
    /// HTTP method, always `GET`.
    pub method: &'static str,
    /// Absolute HTTPS target ending in `/v1/interpretation-runs/{key}/request`.
    pub target_url: String,
    /// Exact version, consumer, and content headers. No credentials.
    pub headers: Vec<(String, String)>,
    /// GET body, always empty.
    pub body: String,
}

/// Extract the opaque idempotency key from
/// `GET /v1/interpretation-runs/{key}/request`.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] for collection,
/// GET-by-id, extra segments, a hostile encoding, empty identity, slash, or
/// NUL, and [`OrchestratorLiveError::LimitExceeded`] when oversized.
pub fn interpretation_run_stored_request_path_id(
    path: &str,
) -> Result<String, OrchestratorLiveError> {
    let remainder = path
        .strip_prefix(INTERPRETATION_RUN_PATH)
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let (encoded_id, rest) = encoded
        .split_once('/')
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if rest != "request" || encoded_id.is_empty() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let idempotency_key = decode_path_segment(encoded_id)?;
    require_nonempty(&idempotency_key)?;
    if idempotency_key == INTERPRETATION_RUN_LOOKUP_PREFIX {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if idempotency_key.contains('/') || idempotency_key.contains('\0') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if idempotency_key.len() > INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    Ok(idempotency_key)
}

/// Whether `path` is the stored-request extra-segment resource.
#[must_use]
pub fn is_interpretation_run_stored_request_path(path: &str) -> bool {
    interpretation_run_stored_request_path_id(path).is_ok()
}

/// Build a credential-free contextual-orchestrator stored-request GET exchange.
///
/// # Errors
///
/// Returns a fail-closed origin or identity error.
pub fn contextual_orchestrator_interpretation_run_stored_request_exchange(
    origin: &str,
    idempotency_key: &str,
) -> Result<InterpretationRunStoredRequestHttpExchange, OrchestratorLiveError> {
    require_nonempty(origin)?;
    if !origin.starts_with("https://") || origin.ends_with('/') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let rest = origin
        .strip_prefix("https://")
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if rest.contains('@') || rest.contains('?') || rest.contains('#') || rest.contains('\\') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if host_implies_table_access(rest) {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    require_nonempty(idempotency_key)?;
    if idempotency_key == INTERPRETATION_RUN_LOOKUP_PREFIX {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if idempotency_key.contains('/') || idempotency_key.contains('\0') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if idempotency_key.len() > INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    let encoded_id = encode_path_segment(idempotency_key);
    Ok(InterpretationRunStoredRequestHttpExchange {
        method: "GET",
        target_url: format!("{origin}{INTERPRETATION_RUN_PATH}/{encoded_id}/request"),
        headers: vec![
            ("content-type".into(), "application/json".into()),
            (
                "tepp-consumer".into(),
                CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE.into(),
            ),
            ("tepp-contract-version".into(), "1".into()),
        ],
        body: String::new(),
    })
}

/// Refuse stored-request JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted for the GET request body.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when a forbidden
/// metric, evidence, or causal-score key is present.
pub fn refuse_metrics_on_interpretation_run_stored_request_payload(
    payload: &str,
) -> Result<(), OrchestratorLiveError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    if payload.contains("tepp.scientific_acceptance.v1") {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    refuse_metrics_on_json(&value)
}

fn refuse_metrics_on_json(value: &serde_json::Value) -> Result<(), OrchestratorLiveError> {
    match value {
        serde_json::Value::Object(object) => {
            if FORBIDDEN_STORED_REQUEST_KEYS
                .iter()
                .any(|key| object.contains_key(*key))
            {
                return Err(OrchestratorLiveError::InvalidWirePayload);
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

fn encode_path_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                let hex = b"0123456789ABCDEF";
                out.push('%');
                out.push(hex[usize::from(byte >> 4)] as char);
                out.push(hex[usize::from(byte & 0x0F)] as char);
            }
        }
    }
    out
}

fn decode_path_segment(value: &str) -> Result<String, OrchestratorLiveError> {
    let mut out = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(OrchestratorLiveError::InvalidWirePayload);
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
            _ => return Err(OrchestratorLiveError::InvalidWirePayload),
        }
    }
    let decoded = String::from_utf8(out).map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    if decoded.chars().any(char::is_control) {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok(decoded)
}

fn from_hex(byte: u8) -> Result<u8, OrchestratorLiveError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(OrchestratorLiveError::InvalidWirePayload),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        contextual_orchestrator_interpretation_run_stored_request_exchange,
        interpretation_run_stored_request_path_id, is_interpretation_run_stored_request_path,
    };
    use crate::error::OrchestratorLiveError;

    #[test]
    fn stored_request_exchange_is_metric_free_get_without_credentials() {
        let exchange = contextual_orchestrator_interpretation_run_stored_request_exchange(
            "https://tepp.example.test",
            "idem-a",
        )
        .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(exchange
            .target_url
            .ends_with("/v1/interpretation-runs/idem-a/request"));
        assert!(exchange.body.is_empty());
        assert!(!exchange.headers.iter().any(|(name, _)| name
            .eq_ignore_ascii_case("authorization")
            || name.eq_ignore_ascii_case("idempotency-key")));
        assert!(is_interpretation_run_stored_request_path(
            "/v1/interpretation-runs/idem-a/request"
        ));
        assert!(!is_interpretation_run_stored_request_path(
            "/v1/interpretation-runs/idem-a"
        ));
        assert_eq!(
            interpretation_run_stored_request_path_id("/v1/interpretation-runs/idem-a/request")
                .expect("id"),
            "idem-a"
        );
        assert_eq!(
            interpretation_run_stored_request_path_id("/v1/interpretation-runs/idem-a"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_stored_request_path_id("/v1/interpretation-runs/idem-a/cancel"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_stored_request_path_id("/v1/interpretation-runs/by-run-id/request"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_stored_request_path_id(
                "/v1/interpretation-runs/by-run-id/orch-run-1/request"
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_stored_request_exchange(
                "http://tepp.example.test",
                "idem-a"
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
    }
}
