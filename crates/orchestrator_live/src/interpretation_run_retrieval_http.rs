//! Provider-owned interpretation-run GET-by-id contracts.
//!
//! GAP-003A unique slice: `GET /v1/interpretation-runs/{idempotency_key}`
//! returns one accepted metric-free hypothetical identity on
//! `OrchestratorLiveService` / `tepp-orchestrator-loopback` so operators who
//! hold a collection identity do not replay POST. Collection rows stay
//! `claim_status=hypothetical` and `scientific_authority=false`.
//! `tepp.scientific_acceptance.v1` never appears. The retrieval does not infer
//! causality or call a model provider. This module does not duplicate
//! interpretation-run CLI (#425), collection GET (#433), collection CLI
//! (#436), project-history GET-by-id (#429), retrieval CLI (#431),
//! analysis-run GET-by-id (#359), Leiden, or GAP-010 Figma/export.
//! Persistence remains GAP-003B. Naruon and `LineageWeave` are refused.
//! `NaruonLiveService` stays POST-only.

use crate::error::OrchestratorLiveError;
use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
use crate::interpretation_run_collection_http::{
    refuse_metrics_on_interpretation_run_collection_payload, InterpretationRunCollectionItem,
    INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN,
};
use crate::request::{
    host_implies_table_access, require_nonempty, to_json, INTERPRETATION_RUN_PATH,
};

/// Maximum opaque idempotency-key length on the retrieval path.
pub const INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN: usize =
    INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN;

/// Typed GET exchange for interpretation-run GET-by-id.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpretationRunRetrievalHttpExchange {
    /// HTTP method, always `GET`.
    pub method: &'static str,
    /// Absolute HTTPS target ending in `/v1/interpretation-runs/{key}`.
    pub target_url: String,
    /// Exact version, consumer, and content headers. No credentials.
    pub headers: Vec<(String, String)>,
    /// GET body, always empty.
    pub body: String,
}

/// Extract the opaque idempotency key from `GET /v1/interpretation-runs/{key}`.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] for the collection
/// path, extra segments, a hostile encoding, empty identity, slash, or NUL,
/// and [`OrchestratorLiveError::LimitExceeded`] when oversized.
pub fn interpretation_run_retrieval_path_id(path: &str) -> Result<String, OrchestratorLiveError> {
    let remainder = path
        .strip_prefix(INTERPRETATION_RUN_PATH)
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let idempotency_key = decode_path_segment(encoded)?;
    require_nonempty(&idempotency_key)?;
    if idempotency_key.contains('/') || idempotency_key.contains('\0') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if idempotency_key.len() > INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    Ok(idempotency_key)
}

/// Serialize one metric-free retrieval identity.
///
/// # Errors
///
/// Returns a validation or metric-key error.
pub fn interpretation_run_retrieval_item_json(
    item: &InterpretationRunCollectionItem,
) -> Result<String, OrchestratorLiveError> {
    let payload = to_json(item)?;
    refuse_metrics_on_interpretation_run_collection_payload(&payload)?;
    Ok(payload)
}

/// Build a credential-free contextual-orchestrator GET-by-id exchange.
///
/// # Errors
///
/// Returns a fail-closed origin or identity error.
pub fn contextual_orchestrator_interpretation_run_retrieval_exchange(
    origin: &str,
    idempotency_key: &str,
) -> Result<InterpretationRunRetrievalHttpExchange, OrchestratorLiveError> {
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
    if idempotency_key.contains('/') || idempotency_key.contains('\0') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if idempotency_key.len() > INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    let encoded_id = encode_path_segment(idempotency_key);
    Ok(InterpretationRunRetrievalHttpExchange {
        method: "GET",
        target_url: format!("{origin}{INTERPRETATION_RUN_PATH}/{encoded_id}"),
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
        contextual_orchestrator_interpretation_run_retrieval_exchange,
        interpretation_run_retrieval_item_json, interpretation_run_retrieval_path_id,
        INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN,
    };
    use crate::error::OrchestratorLiveError;
    use crate::interpretation_run_collection_http::InterpretationRunCollectionItem;
    use crate::mode::OrchestrationMode;
    use crate::request::INTERPRETATION_RUN_PATH;

    #[test]
    fn retrieval_exchange_is_metric_free_get_without_credentials() {
        let exchange = contextual_orchestrator_interpretation_run_retrieval_exchange(
            "https://tepp.example.test",
            "idem-a",
        )
        .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(exchange
            .target_url
            .ends_with("/v1/interpretation-runs/idem-a"));
        assert!(exchange.body.is_empty());
        assert!(!exchange
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                || name.eq_ignore_ascii_case("idempotency-key")));
        assert_eq!(
            interpretation_run_retrieval_path_id("/v1/interpretation-runs/idem-a").expect("id"),
            "idem-a"
        );
        let item = InterpretationRunCollectionItem::new(
            "orch-run-1",
            "idem-a",
            OrchestrationMode::Direct,
            "hypothetical",
            false,
        )
        .expect("item");
        let json = interpretation_run_retrieval_item_json(&item).expect("json");
        assert!(!json.contains("rmse"));
        assert!(!json.contains("evidence_span_ids"));
        assert!(!json.contains("tepp.scientific_acceptance.v1"));
        assert!(json.contains("\"claim_status\":\"hypothetical\""));
        assert_eq!(INTERPRETATION_RUN_PATH, "/v1/interpretation-runs");
    }

    #[test]
    fn retrieval_path_and_origins_fail_closed() {
        assert_eq!(
            interpretation_run_retrieval_path_id("/v1/interpretation-runs"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_retrieval_path_id("/v1/interpretation-runs/"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_retrieval_path_id("/v1/interpretation-runs/idem-a/extra"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_retrieval_path_id("/v1/analysis-runs/idem-a"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_retrieval_path_id("/v1/interpretation-runs/idem%2Fslash"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_retrieval_path_id("/v1/interpretation-runs/%00"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_retrieval_path_id(&format!(
                "/v1/interpretation-runs/{}",
                "a".repeat(INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN + 1)
            )),
            Err(OrchestratorLiveError::LimitExceeded)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_retrieval_exchange(
                "http://insecure.example",
                "idem-a",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_retrieval_exchange(
                "https://postgres.example.test",
                "idem-a",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_retrieval_exchange(
                "https://tepp.example.test",
                "idem/slash",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_retrieval_exchange(
                "https://tepp.example.test",
                "",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_retrieval_path_id("/v1/interpretation-runs/%zz"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
    }
}
