//! Provider-owned analysis-run stored-request GET contracts.
//!
//! GAP-003A ninth slice: `GET /v1/analysis-runs/{run_id}/request` returns the
//! metric-free stored create fields operators need before retry — snapshot,
//! cutoff, model contract, and output profile. Collection GET lists identity
//! only. GET-by-id (#359) remains a later slice on this stack. Retry HTTP
//! (#369) clones blindly. This module does not serve lifecycle POST, cancel,
//! collection GET, retry POST, or loopback CLI. Persistence remains GAP-003B.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{
    ANALYSIS_RUN_STATUS_PATH, AnalysisRunStatusState, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
};
use serde::{Deserialize, Serialize};

/// Maximum length accepted for an opaque run identity in the stored-request path.
pub const ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN: usize = 128;

/// Supported analysis-run stored-request contract version.
pub const ANALYSIS_RUN_STORED_REQUEST_CONTRACT_VERSION: u16 = 1;

const FORBIDDEN_STORED_REQUEST_KEYS: [&str; 14] = [
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

/// Metric-free stored create fields for one analysis run.
///
/// Operators inspect snapshot, cutoff, model contract, and output profile
/// before retry. The payload never carries a terminal result or
/// scientific-acceptance artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunStoredRequest {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Opaque server-assigned run identity.
    pub run_id: String,
    /// Current lifecycle state.
    pub run_state: AnalysisRunStatusState,
    /// Exact request idempotency key of this run.
    pub idempotency_key: String,
    /// Immutable corpus/evidence snapshot identity.
    pub snapshot_id: String,
    /// Knowledge cutoff instant as an ISO-8601 / RFC 3339 string.
    pub knowledge_cutoff: String,
    /// Versioned model/backend contract identity.
    pub model_contract_version: String,
    /// Requested output profile name.
    pub output_profile: String,
}

impl AnalysisRunStoredRequest {
    /// Construct a validated metric-free stored-request payload.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, an oversized run
    /// identity, or an unsupported contract version.
    pub fn new(
        run_id: impl Into<String>,
        run_state: AnalysisRunStatusState,
        idempotency_key: impl Into<String>,
        snapshot_id: impl Into<String>,
        knowledge_cutoff: impl Into<String>,
        model_contract_version: impl Into<String>,
        output_profile: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let stored = Self {
            contract_version: ANALYSIS_RUN_STORED_REQUEST_CONTRACT_VERSION,
            run_id: run_id.into(),
            run_state,
            idempotency_key: idempotency_key.into(),
            snapshot_id: snapshot_id.into(),
            knowledge_cutoff: knowledge_cutoff.into(),
            model_contract_version: model_contract_version.into(),
            output_profile: output_profile.into(),
        };
        stored.validate()?;
        Ok(stored)
    }

    /// Parse and validate a stored-request payload with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate a stored-request payload with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_stored_request_payload(payload)?;
        let stored: Self = from_json(payload)?;
        stored.validate()?;
        Ok(stored)
    }

    /// Serialize this stored-request payload after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_stored_request_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            ANALYSIS_RUN_STORED_REQUEST_CONTRACT_VERSION,
        )?;
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        require_nonempty(&self.snapshot_id)?;
        require_nonempty(&self.knowledge_cutoff)?;
        require_nonempty(&self.model_contract_version)?;
        require_nonempty(&self.output_profile)?;
        if self.run_id.len() > ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        Ok(())
    }
}

/// Refuse stored-request JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted for the GET request body. Non-object JSON
/// fails closed as invalid wire.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present or the payload is a non-empty non-object.
pub fn refuse_metrics_on_stored_request_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    let Some(object) = value.as_object() else {
        return Err(ApiError::InvalidWirePayload);
    };
    if FORBIDDEN_STORED_REQUEST_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

/// Extract the opaque run identity from `GET /v1/analysis-runs/{run_id}/request`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a collection path, extra
/// segments, a missing `/request` suffix, cancel/retry/running/terminal
/// suffixes, or a hostile encoding, and [`ApiError::LimitExceeded`] when the
/// decoded identity exceeds [`ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN`].
pub(crate) fn analysis_run_stored_request_path_run_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(ANALYSIS_RUN_STATUS_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_suffix("/request")
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let run_id = decode_path_segment(encoded)?;
    if run_id.len() > ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(run_id)
}

/// Build a provider-owned `GET` analysis-run stored-request exchange.
///
/// The builder refuses non-`https` origins and empty or oversized run
/// identifiers. It does not inject credentials. The GET body is empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// identity, and [`ApiError::LimitExceeded`] when the run identity exceeds
/// [`ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN`] bytes.
pub fn naruon_analysis_run_stored_request_exchange(
    origin: &str,
    run_id: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    require_nonempty(run_id)?;
    if run_id.len() > ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_run_id = encode_path_segment(run_id);
    let target_path = format!("{ANALYSIS_RUN_STATUS_PATH}/{encoded_run_id}/request");
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

    fn sample_stored() -> AnalysisRunStoredRequest {
        AnalysisRunStoredRequest::new(
            "tepp-run-1",
            AnalysisRunStatusState::Failed,
            "idem-1",
            "snapshot-1",
            "2026-08-01T00:00:00Z",
            "tepp-analysis-run-v1",
            "calibrated_event_measurement",
        )
        .expect("stored")
    }

    #[test]
    fn stored_request_round_trips_and_refuses_hostile_shapes() {
        let stored = sample_stored();
        let json = stored.to_json().expect("json");
        assert_eq!(
            AnalysisRunStoredRequest::from_json(&json).expect("decode"),
            stored
        );
        assert!(!json.contains("rmse"));
        assert!(!json.contains("scientific_acceptance"));
        assert!(!json.contains("terminal_result"));
        assert!(!json.contains("tenant_workspace_id"));

        assert_eq!(
            AnalysisRunStoredRequest::new(
                "",
                AnalysisRunStatusState::Failed,
                "idem-1",
                "snapshot-1",
                "2026-08-01T00:00:00Z",
                "tepp-analysis-run-v1",
                "calibrated_event_measurement",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunStoredRequest::new(
                "tepp-run-1",
                AnalysisRunStatusState::Failed,
                "",
                "snapshot-1",
                "2026-08-01T00:00:00Z",
                "tepp-analysis-run-v1",
                "calibrated_event_measurement",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunStoredRequest::new(
                "a".repeat(ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN + 1),
                AnalysisRunStatusState::Failed,
                "idem-1",
                "snapshot-1",
                "2026-08-01T00:00:00Z",
                "tepp-analysis-run-v1",
                "calibrated_event_measurement",
            ),
            Err(ApiError::LimitExceeded)
        );

        let mut unsupported = stored.clone();
        unsupported.contract_version = 9;
        assert_eq!(
            unsupported.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunStoredRequest::from_json(
                r#"{"contract_version":9,"run_id":"tepp-run-1","run_state":"failed","idempotency_key":"idem-1","snapshot_id":"snapshot-1","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"tepp-analysis-run-v1","output_profile":"calibrated_event_measurement"}"#
            ),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunStoredRequest::from_json(
                r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"failed","idempotency_key":"idem-1","snapshot_id":"snapshot-1","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"tepp-analysis-run-v1","output_profile":"calibrated_event_measurement","extra":true}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunStoredRequest::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            AnalysisRunStoredRequest::from_json("[1,2,3]"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn stored_request_payloads_refuse_scientific_metric_keys() {
        assert_eq!(refuse_metrics_on_stored_request_payload(""), Ok(()));
        assert_eq!(refuse_metrics_on_stored_request_payload("   "), Ok(()));
        assert_eq!(
            refuse_metrics_on_stored_request_payload(r#"{"run_id":"r"}"#),
            Ok(())
        );
        for key in FORBIDDEN_STORED_REQUEST_KEYS {
            let payload = format!(r#"{{"{key}":1,"run_id":"r"}}"#);
            assert_eq!(
                refuse_metrics_on_stored_request_payload(&payload),
                Err(ApiError::InvalidWirePayload),
                "key={key}"
            );
        }
        assert_eq!(
            refuse_metrics_on_stored_request_payload("[true]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_stored_request_payload("null"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn stored_request_path_decodes_identities_and_refuses_hostile_segments() {
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/tepp-run-1/request")
                .expect("plain"),
            "tepp-run-1"
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/run%2dabc/request")
                .expect("lower"),
            "run-abc"
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/run%2Dabc/request")
                .expect("upper"),
            "run-abc"
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/tepp-run-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/tepp-run-1/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/tepp-run-1/retry"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/tepp-run-1/running"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/tepp-run-1/terminal"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/other/tepp-run-1/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs//request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/a/b/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/%2F/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id("/v1/analysis-runs/%00/request"),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized = format!(
            "/v1/analysis-runs/{}/request",
            "a".repeat(ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN + 1)
        );
        assert_eq!(
            analysis_run_stored_request_path_run_id(&oversized),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(decode_path_segment(""), Err(ApiError::InvalidWirePayload));
        assert_eq!(from_hex(b'0'), Ok(0));
        assert_eq!(from_hex(b'a'), Ok(10));
        assert_eq!(from_hex(b'F'), Ok(15));
    }

    #[test]
    fn stored_request_exchange_gets_https_path_without_credentials() {
        let exchange =
            naruon_analysis_run_stored_request_exchange("https://tepp.example.com", "tepp-run-1")
                .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.com/v1/analysis-runs/tepp-run-1/request"
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

        let encoded = naruon_analysis_run_stored_request_exchange(
            "https://tepp.example.com",
            "run/../../etc",
        )
        .expect("encoded");
        assert!(encoded.target_url.contains("run%2F..%2F..%2Fetc/request"));

        assert_eq!(
            naruon_analysis_run_stored_request_exchange("http://tepp.example.com", "tepp-run-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_stored_request_exchange("https://tepp.example.com", ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_stored_request_exchange(
                "https://tepp.example.com",
                &"a".repeat(ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN + 1)
            ),
            Err(ApiError::LimitExceeded)
        );
    }
}
