//! Provider-owned analysis-run retry-parent GET contracts.
//!
//! GAP-003A twelfth slice: `GET /v1/analysis-runs/{run_id}/parent` returns the
//! metric-free parent identity of a listed run. Retry-lineage GET lists
//! children of a parent. Collection GET lists parent and child independently
//! without `retried_from`. Idempotency-key lookup resolves a key to a
//! `run_id` without linkage. Operators looking at a retry child therefore
//! cannot see which parent it came from. This module does not serve GET-by-id
//! (#359), lifecycle POST (#360), cancel HTTP (#361), loopback CLI (#362),
//! collection GET (#368), retry POST (#369), stored-request GET (#377),
//! retry-lineage GET (#379), or idempotency-key lookup GET (#380).
//! Persistence remains GAP-003B.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{
    ANALYSIS_RUN_STATUS_PATH, AnalysisRunStatusState, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
};
use serde::{Deserialize, Serialize};

/// Maximum length accepted for an opaque run identity in the retry-parent path.
pub const ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN: usize = 128;

/// Supported analysis-run retry-parent contract version.
pub const ANALYSIS_RUN_RETRY_PARENT_CONTRACT_VERSION: u16 = 1;

const FORBIDDEN_RETRY_PARENT_KEYS: [&str; 14] = [
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
    "tenant_workspace_id",
];

/// One metric-free parent identity of a retry child.
///
/// The row names the original attempt. It never carries a terminal result,
/// snapshot, or scientific-acceptance artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunRetryParentItem {
    /// Opaque server-assigned parent run identity.
    pub run_id: String,
    /// Current lifecycle state of the parent.
    pub run_state: AnalysisRunStatusState,
    /// Exact request idempotency key of the parent.
    pub idempotency_key: String,
}

impl AnalysisRunRetryParentItem {
    /// Construct a validated metric-free retry-parent identity row.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities or an oversized run
    /// identity.
    pub fn new(
        run_id: impl Into<String>,
        run_state: AnalysisRunStatusState,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let item = Self {
            run_id: run_id.into(),
            run_state,
            idempotency_key: idempotency_key.into(),
        };
        item.validate()?;
        Ok(item)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.run_id.len() > ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        Ok(())
    }
}

/// Metric-free retry parent for one analysis run.
///
/// Operators inspect which listed run a retry child was cloned from.
/// `parent` is JSON `null` when the run was never retried from another run.
/// The payload never carries a terminal result or scientific-acceptance
/// artifact. The `parent` key is always present.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunRetryParent {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Opaque server-assigned child (or original) run identity.
    pub run_id: String,
    /// Current lifecycle state of the inspected run.
    pub run_state: AnalysisRunStatusState,
    /// Exact request idempotency key of the inspected run.
    pub idempotency_key: String,
    /// Direct parent identity, or `null` when this run was never retried.
    pub parent: Option<AnalysisRunRetryParentItem>,
}

impl AnalysisRunRetryParent {
    /// Construct a validated metric-free retry-parent payload.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, an oversized run
    /// identity, or an unsupported contract version.
    pub fn new(
        run_id: impl Into<String>,
        run_state: AnalysisRunStatusState,
        idempotency_key: impl Into<String>,
        parent: Option<AnalysisRunRetryParentItem>,
    ) -> Result<Self, ApiError> {
        let payload = Self {
            contract_version: ANALYSIS_RUN_RETRY_PARENT_CONTRACT_VERSION,
            run_id: run_id.into(),
            run_state,
            idempotency_key: idempotency_key.into(),
            parent,
        };
        payload.validate()?;
        Ok(payload)
    }

    /// Parse and validate a retry-parent payload with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate a retry-parent payload with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_retry_parent_payload(payload)?;
        let decoded: Self = from_json(payload)?;
        decoded.validate()?;
        Ok(decoded)
    }

    /// Serialize this retry-parent payload after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_retry_parent_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            ANALYSIS_RUN_RETRY_PARENT_CONTRACT_VERSION,
        )?;
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.run_id.len() > ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        if let Some(parent) = &self.parent {
            parent.validate()?;
        }
        Ok(())
    }
}

/// Refuse retry-parent JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted for the GET request body. Non-object JSON
/// fails closed as invalid wire.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present or the payload is a non-empty non-object.
pub fn refuse_metrics_on_retry_parent_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    let Some(object) = value.as_object() else {
        return Err(ApiError::InvalidWirePayload);
    };
    if FORBIDDEN_RETRY_PARENT_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

/// Extract the opaque run identity from `GET /v1/analysis-runs/{run_id}/parent`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a collection path, extra
/// segments, a missing `/parent` suffix, cancel/retry/retries/request/
/// running/terminal suffixes, or a hostile encoding, and
/// [`ApiError::LimitExceeded`] when the decoded identity exceeds
/// [`ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN`].
pub(crate) fn analysis_run_retry_parent_path_run_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(ANALYSIS_RUN_STATUS_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_suffix("/parent")
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let run_id = decode_path_segment(encoded)?;
    if run_id.len() > ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(run_id)
}

/// Build a provider-owned `GET` analysis-run retry-parent exchange.
///
/// The builder refuses non-`https` origins and empty or oversized run
/// identifiers. It does not inject credentials. The GET body is empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// identity, and [`ApiError::LimitExceeded`] when the run identity exceeds
/// [`ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN`] bytes.
pub fn naruon_analysis_run_retry_parent_exchange(
    origin: &str,
    run_id: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    require_nonempty(run_id)?;
    if run_id.len() > ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_run_id = encode_path_segment(run_id);
    let target_path = format!("{ANALYSIS_RUN_STATUS_PATH}/{encoded_run_id}/parent");
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
    if decoded.is_empty() || decoded.contains('/') || decoded.contains('\0') {
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
    use super::*;

    fn sample_parent() -> AnalysisRunRetryParent {
        AnalysisRunRetryParent::new(
            "tepp-run-2",
            AnalysisRunStatusState::Accepted,
            "idem-retry-1",
            Some(
                AnalysisRunRetryParentItem::new(
                    "tepp-run-1",
                    AnalysisRunStatusState::Failed,
                    "idem-1",
                )
                .expect("parent"),
            ),
        )
        .expect("payload")
    }

    #[test]
    fn retry_parent_round_trips_and_refuses_hostile_shapes() {
        let payload = sample_parent();
        let json = payload.to_json().expect("json");
        assert_eq!(
            AnalysisRunRetryParent::from_json(&json).expect("decode"),
            payload
        );
        assert!(json.contains("\"parent\":{") || json.contains("\"parent\": {"));
        assert!(!json.contains("rmse"));
        assert!(!json.contains("scientific_acceptance"));
        assert!(!json.contains("terminal_result"));
        assert!(!json.contains("tenant_workspace_id"));
        assert!(!json.contains("snapshot_id"));
        assert!(!json.contains("retried_from"));

        let original = AnalysisRunRetryParent::new(
            "tepp-run-1",
            AnalysisRunStatusState::Failed,
            "idem-1",
            None,
        )
        .expect("original");
        let original_json = original.to_json().expect("original json");
        assert!(original_json.contains("\"parent\":null"));
        assert_eq!(
            AnalysisRunRetryParent::from_json(&original_json).expect("null parent"),
            original
        );

        assert_eq!(
            AnalysisRunRetryParent::new("", AnalysisRunStatusState::Failed, "idem-1", None,),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRetryParent::new("tepp-run-1", AnalysisRunStatusState::Failed, "", None,),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRetryParent::new(
                "a".repeat(ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN + 1),
                AnalysisRunStatusState::Failed,
                "idem-1",
                None,
            ),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            AnalysisRunRetryParentItem::new(
                "a".repeat(ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN + 1),
                AnalysisRunStatusState::Failed,
                "idem-1",
            ),
            Err(ApiError::LimitExceeded)
        );

        let mut unsupported = payload.clone();
        unsupported.contract_version = 9;
        assert_eq!(
            unsupported.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunRetryParent::from_json(
                r#"{"contract_version":9,"run_id":"tepp-run-1","run_state":"failed","idempotency_key":"idem-1","parent":null}"#
            ),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunRetryParent::from_json(
                r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"failed","idempotency_key":"idem-1","parent":null,"extra":true}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRetryParent::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            AnalysisRunRetryParent::from_json("[1,2,3]"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn retry_parent_payloads_refuse_scientific_metric_keys() {
        assert_eq!(refuse_metrics_on_retry_parent_payload(""), Ok(()));
        assert_eq!(refuse_metrics_on_retry_parent_payload("   "), Ok(()));
        assert_eq!(
            refuse_metrics_on_retry_parent_payload(r#"{"run_id":"r"}"#),
            Ok(())
        );
        for key in FORBIDDEN_RETRY_PARENT_KEYS {
            let payload = format!(r#"{{"{key}":1,"run_id":"r"}}"#);
            assert_eq!(
                refuse_metrics_on_retry_parent_payload(&payload),
                Err(ApiError::InvalidWirePayload),
                "key={key}"
            );
        }
        assert_eq!(
            refuse_metrics_on_retry_parent_payload("[true]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_retry_parent_payload("null"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn retry_parent_path_decodes_identities_and_refuses_hostile_segments() {
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/tepp-run-1/parent")
                .expect("plain"),
            "tepp-run-1"
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/run%2dabc/parent")
                .expect("lower"),
            "run-abc"
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/run%2Dabc/parent")
                .expect("upper"),
            "run-abc"
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/tepp-run-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/tepp-run-1/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/tepp-run-1/retry"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/tepp-run-1/retries"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/tepp-run-1/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/by-idempotency/idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/tepp-run-1/running"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/tepp-run-1/terminal"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/other/tepp-run-1/parent"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs//parent"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/a/b/parent"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/%2F/parent"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id("/v1/analysis-runs/%00/parent"),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized = format!(
            "/v1/analysis-runs/{}/parent",
            "a".repeat(ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN + 1)
        );
        assert_eq!(
            analysis_run_retry_parent_path_run_id(&oversized),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(decode_path_segment(""), Err(ApiError::InvalidWirePayload));
        assert_eq!(from_hex(b'0'), Ok(0));
        assert_eq!(from_hex(b'a'), Ok(10));
        assert_eq!(from_hex(b'F'), Ok(15));
    }

    #[test]
    fn retry_parent_exchange_gets_https_path_without_credentials() {
        let exchange =
            naruon_analysis_run_retry_parent_exchange("https://tepp.example.com", "tepp-run-1")
                .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.com/v1/analysis-runs/tepp-run-1/parent"
        );
        assert!(exchange.body.is_empty());
        assert!(
            exchange
                .headers
                .iter()
                .any(|(name, value)| name == "tepp-consumer" && value == "naruon")
        );
        assert!(
            !exchange
                .headers
                .iter()
                .any(|(name, _)| name.contains("authorization")
                    || name.contains("copilot")
                    || name.contains("idempotency"))
        );

        let encoded =
            naruon_analysis_run_retry_parent_exchange("https://tepp.example.com", "run/../../etc")
                .expect("encoded");
        assert!(encoded.target_url.contains("run%2F..%2F..%2Fetc/parent"));

        assert_eq!(
            naruon_analysis_run_retry_parent_exchange("http://tepp.example.com", "tepp-run-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_retry_parent_exchange("https://tepp.example.com", ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_retry_parent_exchange(
                "https://tepp.example.com",
                &"a".repeat(ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN + 1)
            ),
            Err(ApiError::LimitExceeded)
        );
    }
}
