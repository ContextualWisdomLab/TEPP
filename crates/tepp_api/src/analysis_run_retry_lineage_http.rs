//! Provider-owned analysis-run retry-lineage GET contracts.
//!
//! GAP-003A tenth slice: `GET /v1/analysis-runs/{run_id}/retries` returns the
//! metric-free direct retry children of a listed run. Collection GET lists
//! parent and child independently. Retry HTTP clones without exposing
//! parent/child linkage. Stored-request GET inspects create fields, not
//! lineage. This module does not serve GET-by-id (#359), lifecycle POST
//! (#360), cancel HTTP (#361), loopback CLI (#362), collection GET (#368),
//! retry POST (#369), stored-request GET (#377), or cancel CLI (#378).
//! Persistence remains GAP-003B.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{
    ANALYSIS_RUN_STATUS_PATH, AnalysisRunStatusState, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
};
use serde::{Deserialize, Serialize};

/// Maximum length accepted for an opaque run identity in the retry-lineage path.
pub const ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN: usize = 128;

/// Maximum number of direct retry children returned for one parent.
pub const ANALYSIS_RUN_RETRY_LINEAGE_MAX_RETRIES: usize = 64;

/// Supported analysis-run retry-lineage contract version.
pub const ANALYSIS_RUN_RETRY_LINEAGE_CONTRACT_VERSION: u16 = 1;

const FORBIDDEN_RETRY_LINEAGE_KEYS: [&str; 14] = [
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

/// One metric-free direct retry child of a parent analysis run.
///
/// The row names the cloned attempt. It never carries a terminal result,
/// snapshot, or scientific-acceptance artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunRetryLineageItem {
    /// Opaque server-assigned child run identity.
    pub run_id: String,
    /// Current lifecycle state of the child.
    pub run_state: AnalysisRunStatusState,
    /// Exact request idempotency key of the child.
    pub idempotency_key: String,
}

impl AnalysisRunRetryLineageItem {
    /// Construct a validated metric-free retry-lineage child row.
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
        if self.run_id.len() > ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        Ok(())
    }
}

/// Metric-free retry lineage for one parent analysis run.
///
/// Operators inspect which cloned attempts exist for a failed or cancelled
/// parent. An empty `retries` array means the parent was never retried.
/// The payload never carries a terminal result or scientific-acceptance
/// artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunRetryLineage {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Opaque server-assigned parent run identity.
    pub run_id: String,
    /// Current lifecycle state of the parent.
    pub run_state: AnalysisRunStatusState,
    /// Exact request idempotency key of the parent.
    pub idempotency_key: String,
    /// Direct retry children, sorted by `run_id`.
    pub retries: Vec<AnalysisRunRetryLineageItem>,
}

impl AnalysisRunRetryLineage {
    /// Construct a validated metric-free retry-lineage payload.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, an oversized run
    /// identity, too many children, or an unsupported contract version.
    pub fn new(
        run_id: impl Into<String>,
        run_state: AnalysisRunStatusState,
        idempotency_key: impl Into<String>,
        retries: Vec<AnalysisRunRetryLineageItem>,
    ) -> Result<Self, ApiError> {
        let lineage = Self {
            contract_version: ANALYSIS_RUN_RETRY_LINEAGE_CONTRACT_VERSION,
            run_id: run_id.into(),
            run_state,
            idempotency_key: idempotency_key.into(),
            retries,
        };
        lineage.validate()?;
        Ok(lineage)
    }

    /// Parse and validate a retry-lineage payload with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate a retry-lineage payload with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_retry_lineage_payload(payload)?;
        let lineage: Self = from_json(payload)?;
        lineage.validate()?;
        Ok(lineage)
    }

    /// Serialize this retry-lineage payload after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_retry_lineage_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            ANALYSIS_RUN_RETRY_LINEAGE_CONTRACT_VERSION,
        )?;
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.run_id.len() > ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        if self.retries.len() > ANALYSIS_RUN_RETRY_LINEAGE_MAX_RETRIES {
            return Err(ApiError::LimitExceeded);
        }
        for item in &self.retries {
            item.validate()?;
        }
        Ok(())
    }
}

/// Refuse retry-lineage JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted for the GET request body. Non-object JSON
/// fails closed as invalid wire.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present or the payload is a non-empty non-object.
pub fn refuse_metrics_on_retry_lineage_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    let Some(object) = value.as_object() else {
        return Err(ApiError::InvalidWirePayload);
    };
    if FORBIDDEN_RETRY_LINEAGE_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

/// Extract the opaque parent identity from `GET /v1/analysis-runs/{run_id}/retries`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a collection path, extra
/// segments, a missing `/retries` suffix, cancel/retry/request/running/terminal
/// suffixes, or a hostile encoding, and [`ApiError::LimitExceeded`] when the
/// decoded identity exceeds [`ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN`].
pub(crate) fn analysis_run_retry_lineage_path_run_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(ANALYSIS_RUN_STATUS_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_suffix("/retries")
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let run_id = decode_path_segment(encoded)?;
    if run_id.len() > ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(run_id)
}

/// Build a provider-owned `GET` analysis-run retry-lineage exchange.
///
/// The builder refuses non-`https` origins and empty or oversized run
/// identifiers. It does not inject credentials. The GET body is empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// identity, and [`ApiError::LimitExceeded`] when the run identity exceeds
/// [`ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN`] bytes.
pub fn naruon_analysis_run_retry_lineage_exchange(
    origin: &str,
    run_id: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    require_nonempty(run_id)?;
    if run_id.len() > ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_run_id = encode_path_segment(run_id);
    let target_path = format!("{ANALYSIS_RUN_STATUS_PATH}/{encoded_run_id}/retries");
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

    fn sample_lineage() -> AnalysisRunRetryLineage {
        AnalysisRunRetryLineage::new(
            "tepp-run-1",
            AnalysisRunStatusState::Failed,
            "idem-1",
            vec![
                AnalysisRunRetryLineageItem::new(
                    "tepp-run-2",
                    AnalysisRunStatusState::Accepted,
                    "idem-retry-1",
                )
                .expect("child"),
            ],
        )
        .expect("lineage")
    }

    #[test]
    fn retry_lineage_round_trips_and_refuses_hostile_shapes() {
        let lineage = sample_lineage();
        let json = lineage.to_json().expect("json");
        assert_eq!(
            AnalysisRunRetryLineage::from_json(&json).expect("decode"),
            lineage
        );
        assert!(!json.contains("rmse"));
        assert!(!json.contains("scientific_acceptance"));
        assert!(!json.contains("terminal_result"));
        assert!(!json.contains("tenant_workspace_id"));
        assert!(!json.contains("snapshot_id"));

        assert_eq!(
            AnalysisRunRetryLineage::new("", AnalysisRunStatusState::Failed, "idem-1", Vec::new(),),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRetryLineage::new(
                "tepp-run-1",
                AnalysisRunStatusState::Failed,
                "",
                Vec::new(),
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRetryLineage::new(
                "a".repeat(ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN + 1),
                AnalysisRunStatusState::Failed,
                "idem-1",
                Vec::new(),
            ),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            AnalysisRunRetryLineageItem::new(
                "a".repeat(ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN + 1),
                AnalysisRunStatusState::Accepted,
                "idem-retry-1",
            ),
            Err(ApiError::LimitExceeded)
        );

        let mut unsupported = lineage.clone();
        unsupported.contract_version = 9;
        assert_eq!(
            unsupported.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunRetryLineage::from_json(
                r#"{"contract_version":9,"run_id":"tepp-run-1","run_state":"failed","idempotency_key":"idem-1","retries":[]}"#
            ),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunRetryLineage::from_json(
                r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"failed","idempotency_key":"idem-1","retries":[],"extra":true}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRetryLineage::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            AnalysisRunRetryLineage::from_json("[1,2,3]"),
            Err(ApiError::InvalidWirePayload)
        );
        let too_many = vec![
            AnalysisRunRetryLineageItem::new(
                "tepp-run-x",
                AnalysisRunStatusState::Accepted,
                "idem-x",
            )
            .expect("row");
            ANALYSIS_RUN_RETRY_LINEAGE_MAX_RETRIES + 1
        ];
        assert_eq!(
            AnalysisRunRetryLineage::new(
                "tepp-run-1",
                AnalysisRunStatusState::Failed,
                "idem-1",
                too_many,
            ),
            Err(ApiError::LimitExceeded)
        );
        let empty = AnalysisRunRetryLineage::new(
            "tepp-run-1",
            AnalysisRunStatusState::Accepted,
            "idem-1",
            Vec::new(),
        )
        .expect("empty");
        assert!(empty.retries.is_empty());
    }

    #[test]
    fn retry_lineage_payloads_refuse_scientific_metric_keys() {
        assert_eq!(refuse_metrics_on_retry_lineage_payload(""), Ok(()));
        assert_eq!(refuse_metrics_on_retry_lineage_payload("   "), Ok(()));
        assert_eq!(
            refuse_metrics_on_retry_lineage_payload(r#"{"run_id":"r"}"#),
            Ok(())
        );
        for key in FORBIDDEN_RETRY_LINEAGE_KEYS {
            let payload = format!(r#"{{"{key}":1,"run_id":"r"}}"#);
            assert_eq!(
                refuse_metrics_on_retry_lineage_payload(&payload),
                Err(ApiError::InvalidWirePayload),
                "key={key}"
            );
        }
        assert_eq!(
            refuse_metrics_on_retry_lineage_payload("[true]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_retry_lineage_payload("null"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn retry_lineage_path_decodes_identities_and_refuses_hostile_segments() {
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/tepp-run-1/retries")
                .expect("plain"),
            "tepp-run-1"
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/run%2dabc/retries")
                .expect("lower"),
            "run-abc"
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/run%2Dabc/retries")
                .expect("upper"),
            "run-abc"
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/tepp-run-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/tepp-run-1/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/tepp-run-1/retry"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/tepp-run-1/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/tepp-run-1/parent"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/by-idempotency/idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/tepp-run-1/running"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/tepp-run-1/terminal"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/other/tepp-run-1/retries"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs//retries"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/a/b/retries"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/%2F/retries"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id("/v1/analysis-runs/%00/retries"),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized = format!(
            "/v1/analysis-runs/{}/retries",
            "a".repeat(ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN + 1)
        );
        assert_eq!(
            analysis_run_retry_lineage_path_run_id(&oversized),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(decode_path_segment(""), Err(ApiError::InvalidWirePayload));
        assert_eq!(from_hex(b'0'), Ok(0));
        assert_eq!(from_hex(b'a'), Ok(10));
        assert_eq!(from_hex(b'F'), Ok(15));
    }

    #[test]
    fn retry_lineage_exchange_gets_https_path_without_credentials() {
        let exchange =
            naruon_analysis_run_retry_lineage_exchange("https://tepp.example.com", "tepp-run-1")
                .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.com/v1/analysis-runs/tepp-run-1/retries"
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
            naruon_analysis_run_retry_lineage_exchange("https://tepp.example.com", "run/../../etc")
                .expect("encoded");
        assert!(encoded.target_url.contains("run%2F..%2F..%2Fetc/retries"));

        assert_eq!(
            naruon_analysis_run_retry_lineage_exchange("http://tepp.example.com", "tepp-run-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_retry_lineage_exchange("https://tepp.example.com", ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_retry_lineage_exchange(
                "https://tepp.example.com",
                &"a".repeat(ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN + 1)
            ),
            Err(ApiError::LimitExceeded)
        );
    }
}
