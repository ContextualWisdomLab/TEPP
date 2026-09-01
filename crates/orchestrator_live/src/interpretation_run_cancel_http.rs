//! Provider-owned interpretation-run cancel HTTP contracts.
//!
//! GAP-003A unique slice: `POST /v1/interpretation-runs/{idempotency_key}/cancel`
//! removes one accepted hypothetical identity from the in-memory
//! `OrchestratorLiveService` / `tepp-orchestrator-loopback` registry. The
//! response stays metric-free with `claim_status=hypothetical`,
//! `scientific_authority=false`, and `cancelled=true`.
//! `tepp.scientific_acceptance.v1` never appears. Cancel does not infer
//! causality or call a model provider. This module does not duplicate
//! interpretation-run CLI (#425), collection GET (#433), collection CLI
//! (#436), GET-by-id HTTP (#438), retrieval CLI (#439), analysis-run cancel
//! HTTP (#361), Leiden, or GAP-010 Figma/export. Persistence remains
//! GAP-003B. Naruon and `LineageWeave` are refused. `NaruonLiveService`
//! stays POST-only for analysis-run and export.

use serde::{Deserialize, Serialize};

use crate::error::OrchestratorLiveError;
use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
use crate::interpretation_run_collection_http::INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN;
use crate::mode::OrchestrationMode;
use crate::request::{
    host_implies_table_access, require_nonempty, to_json, HYPOTHETICAL_CLAIM_STATUS,
    INTERPRETATION_RUN_PATH,
};

/// Maximum opaque idempotency-key length on the cancel path.
pub const INTERPRETATION_RUN_CANCEL_ID_MAX_LEN: usize =
    INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN;

const FORBIDDEN_CANCEL_KEYS: [&str; 14] = [
    "rmse",
    "rmse_standard_error",
    "mean_bias",
    "bias_standard_error",
    "interval_coverage",
    "coverage_wilson_lower",
    "coverage_wilson_upper",
    "temporal_order_accuracy",
    "se_gate_accepted",
    "scientific_acceptance",
    "causal_score",
    "causality",
    "evidence_span_ids",
    "findings",
];

/// Typed POST exchange for interpretation-run cancel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpretationRunCancelHttpExchange {
    /// HTTP method, always `POST`.
    pub method: &'static str,
    /// Absolute HTTPS target ending in `/v1/interpretation-runs/{key}/cancel`.
    pub target_url: String,
    /// Exact version, consumer, and content headers. No credentials.
    pub headers: Vec<(String, String)>,
    /// POST body, always empty.
    pub body: String,
}

/// Metric-free cancelled interpretation-run identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterpretationRunCancelled {
    /// Server-assigned opaque interpretation-run identity.
    pub interpretation_run_id: String,
    /// Exact request idempotency key that minted the stored run.
    pub idempotency_key: String,
    /// Selected orchestration mode.
    pub orchestration_mode: OrchestrationMode,
    /// Fixed claim boundary: accepted output is hypothetical.
    pub claim_status: String,
    /// Always `false`; LLM output is never scientific authority.
    pub scientific_authority: bool,
    /// Always `true` on a successful cancel receipt.
    pub cancelled: bool,
}

impl InterpretationRunCancelled {
    /// Construct a validated cancelled identity.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, a non-hypothetical
    /// claim, scientific authority, or `cancelled=false`.
    pub fn new(
        interpretation_run_id: impl Into<String>,
        idempotency_key: impl Into<String>,
        orchestration_mode: OrchestrationMode,
        claim_status: impl Into<String>,
        scientific_authority: bool,
    ) -> Result<Self, OrchestratorLiveError> {
        let item = Self {
            interpretation_run_id: interpretation_run_id.into(),
            idempotency_key: idempotency_key.into(),
            orchestration_mode,
            claim_status: claim_status.into(),
            scientific_authority,
            cancelled: true,
        };
        item.validate()?;
        Ok(item)
    }

    fn validate(&self) -> Result<(), OrchestratorLiveError> {
        require_nonempty(&self.interpretation_run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.idempotency_key.len() > INTERPRETATION_RUN_CANCEL_ID_MAX_LEN {
            return Err(OrchestratorLiveError::LimitExceeded);
        }
        if self.claim_status != HYPOTHETICAL_CLAIM_STATUS || self.scientific_authority {
            return Err(OrchestratorLiveError::ScientificAuthorityRefused);
        }
        if !self.cancelled {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        Ok(())
    }

    /// Serialize this cancelled identity after metric refusal.
    ///
    /// # Errors
    ///
    /// Returns a validation or metric-key error.
    pub fn to_json(&self) -> Result<String, OrchestratorLiveError> {
        self.validate()?;
        let payload = to_json(self)?;
        refuse_metrics_on_interpretation_run_cancel_payload(&payload)?;
        Ok(payload)
    }
}

/// Extract the opaque idempotency key from
/// `POST /v1/interpretation-runs/{key}/cancel`.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] for the collection
/// path, GET-by-id path, extra segments, a hostile encoding, empty identity,
/// slash, or NUL, and [`OrchestratorLiveError::LimitExceeded`] when oversized.
pub fn interpretation_run_cancel_path_id(path: &str) -> Result<String, OrchestratorLiveError> {
    let remainder = path
        .strip_prefix(INTERPRETATION_RUN_PATH)
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_suffix("/cancel")
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let idempotency_key = decode_path_segment(encoded)?;
    require_nonempty(&idempotency_key)?;
    if idempotency_key.contains('/') || idempotency_key.contains('\0') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if idempotency_key.len() > INTERPRETATION_RUN_CANCEL_ID_MAX_LEN {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    Ok(idempotency_key)
}

/// Refuse cancel JSON that already carries scientific-metric or evidence keys.
///
/// Empty payloads are admitted for the POST request body.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when a forbidden key
/// is present, the scientific-acceptance schema is claimed, or nonempty JSON
/// is not an object.
pub fn refuse_metrics_on_interpretation_run_cancel_payload(
    payload: &str,
) -> Result<(), OrchestratorLiveError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    let Some(object) = value.as_object() else {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    };
    if object
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        == Some("tepp.scientific_acceptance.v1")
    {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if FORBIDDEN_CANCEL_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
    {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok(())
}

/// Build a credential-free contextual-orchestrator cancel exchange.
///
/// # Errors
///
/// Returns a fail-closed origin or identity error.
pub fn contextual_orchestrator_interpretation_run_cancel_exchange(
    origin: &str,
    idempotency_key: &str,
) -> Result<InterpretationRunCancelHttpExchange, OrchestratorLiveError> {
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
    if idempotency_key.len() > INTERPRETATION_RUN_CANCEL_ID_MAX_LEN {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    let encoded_id = encode_path_segment(idempotency_key);
    Ok(InterpretationRunCancelHttpExchange {
        method: "POST",
        target_url: format!("{origin}{INTERPRETATION_RUN_PATH}/{encoded_id}/cancel"),
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
        contextual_orchestrator_interpretation_run_cancel_exchange,
        interpretation_run_cancel_path_id, refuse_metrics_on_interpretation_run_cancel_payload,
        InterpretationRunCancelled, INTERPRETATION_RUN_CANCEL_ID_MAX_LEN,
    };
    use crate::error::OrchestratorLiveError;
    use crate::mode::OrchestrationMode;

    #[test]
    fn cancel_exchange_is_metric_free_post_without_credentials() {
        let exchange = contextual_orchestrator_interpretation_run_cancel_exchange(
            "https://tepp.example.test",
            "idem-a",
        )
        .expect("exchange");
        assert_eq!(exchange.method, "POST");
        assert!(exchange
            .target_url
            .ends_with("/v1/interpretation-runs/idem-a/cancel"));
        assert!(exchange.body.is_empty());
        assert!(!exchange
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                || name.eq_ignore_ascii_case("idempotency-key")));
        assert_eq!(
            interpretation_run_cancel_path_id("/v1/interpretation-runs/idem-a/cancel").expect("id"),
            "idem-a"
        );
        let item = InterpretationRunCancelled::new(
            "orch-run-1",
            "idem-a",
            OrchestrationMode::Direct,
            "hypothetical",
            false,
        )
        .expect("item");
        let json = item.to_json().expect("json");
        assert!(json.contains("\"cancelled\":true"));
        assert!(!json.contains("rmse"));
        assert!(!json.contains("evidence_span_ids"));
        assert!(!json.contains("tepp.scientific_acceptance.v1"));
    }

    #[test]
    fn cancel_path_and_payloads_fail_closed() {
        assert_eq!(
            interpretation_run_cancel_path_id("/v1/interpretation-runs"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_cancel_path_id("/v1/interpretation-runs/idem-a"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_cancel_path_id("/v1/interpretation-runs/idem-a/extra/cancel"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_cancel_path_id("/v1/analysis-runs/idem-a/cancel"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_cancel_path_id(&format!(
                "/v1/interpretation-runs/{}/cancel",
                "a".repeat(INTERPRETATION_RUN_CANCEL_ID_MAX_LEN + 1)
            )),
            Err(OrchestratorLiveError::LimitExceeded)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_cancel_exchange(
                "http://insecure.example",
                "idem-a",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_cancel_exchange(
                "https://tepp.example.test",
                "idem/slash",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_interpretation_run_cancel_payload(""),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_interpretation_run_cancel_payload(r#"{"rmse":1.0}"#),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_interpretation_run_cancel_payload("[]"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
    }
}
