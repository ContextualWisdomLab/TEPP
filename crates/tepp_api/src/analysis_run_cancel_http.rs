//! Provider-owned analysis-run cancel HTTP contracts.
//!
//! GAP-003A fifth slice: `POST /v1/analysis-runs/{run_id}/cancel` is the
//! operator-visible cancel path on the shared loopback listener. Accepted and
//! running runs become metric-free `cancelled` status. Succeeded and failed
//! runs cannot be cancelled. Cancel bodies and cancelled status JSON refuse
//! RMSE, bias, coverage, SE-gate, scientific-acceptance, and report keys.
//! This module does not serve GET status (#359) and does not record running
//! or terminal POST transitions (#360). Persistence remains GAP-003B.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target, standard_headers};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{ANALYSIS_RUN_STATUS_PATH, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT};
use serde::{Deserialize, Serialize};

/// Maximum length accepted for an opaque run identity in the cancel path.
pub const ANALYSIS_RUN_CANCEL_ID_MAX_LEN: usize = 128;

/// Supported analysis-run cancel contract version.
pub const ANALYSIS_RUN_CANCEL_CONTRACT_VERSION: u16 = 1;

const FORBIDDEN_CANCEL_KEYS: [&str; 12] = [
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
];

/// Versioned cancel request for one accepted or running analysis run.
///
/// Path `run_id` and header `idempotency-key` must match these fields when a
/// body is present. An empty POST body is also admitted on the loopback
/// listener and uses the path identity plus the idempotency header.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunCancelRequest {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Opaque server-assigned run identity.
    pub run_id: String,
    /// Exact request idempotency key of the accepted run.
    pub idempotency_key: String,
}

impl AnalysisRunCancelRequest {
    /// Construct a validated cancel request.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities or an unsupported
    /// contract version.
    pub fn new(
        run_id: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let request = Self {
            contract_version: ANALYSIS_RUN_CANCEL_CONTRACT_VERSION,
            run_id: run_id.into(),
            idempotency_key: idempotency_key.into(),
        };
        request.validate()?;
        Ok(request)
    }

    /// Parse and validate a cancel request with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate a cancel request with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_cancel_payload(payload)?;
        let request: Self = from_json(payload)?;
        request.validate()?;
        Ok(request)
    }

    /// Serialize this cancel request after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_cancel_payload(&payload)?;
        Ok(payload)
    }

    pub(crate) fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, ANALYSIS_RUN_CANCEL_CONTRACT_VERSION)?;
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.run_id.len() > ANALYSIS_RUN_CANCEL_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        Ok(())
    }
}

/// Refuse cancel JSON that already carries scientific-metric keys.
///
/// Empty bodies are admitted (the loopback listener treats them as
/// header-and-path cancel). Non-object JSON fails closed as invalid wire.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present or the payload is a non-empty non-object.
pub fn refuse_metrics_on_cancel_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    let Some(object) = value.as_object() else {
        return Err(ApiError::InvalidWirePayload);
    };
    if FORBIDDEN_CANCEL_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

/// Build a provider-owned `POST` analysis-run cancel exchange.
///
/// The builder refuses non-`https` origins and empty or oversized run
/// identifiers. It does not inject credentials.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// identity, and [`ApiError::LimitExceeded`] when the run identity exceeds
/// [`ANALYSIS_RUN_CANCEL_ID_MAX_LEN`] bytes.
pub fn naruon_analysis_run_cancel_exchange(
    origin: &str,
    request: &AnalysisRunCancelRequest,
) -> Result<NaruonHttpExchange, ApiError> {
    request.validate()?;
    let encoded_run_id = encode_path_segment(&request.run_id);
    let target_path = format!("{ANALYSIS_RUN_STATUS_PATH}/{encoded_run_id}/cancel");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "POST",
        target_url,
        headers: standard_headers(&request.idempotency_key),
        body: request.to_json()?,
    })
}

/// Extract the opaque run identity from `POST /v1/analysis-runs/{run_id}/cancel`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a collection path, extra
/// segments, a missing `/cancel` suffix, or a hostile encoding, and
/// [`ApiError::LimitExceeded`] when the decoded identity exceeds
/// [`ANALYSIS_RUN_CANCEL_ID_MAX_LEN`].
pub(crate) fn analysis_run_cancel_path_run_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(ANALYSIS_RUN_STATUS_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_suffix("/cancel")
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let run_id = decode_path_segment(encoded)?;
    if run_id.len() > ANALYSIS_RUN_CANCEL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(run_id)
}

pub(crate) fn encode_path_segment(value: &str) -> String {
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
    use crate::DEFAULT_ANALYSIS_RUN_BYTE_LIMIT;

    fn sample_request() -> AnalysisRunCancelRequest {
        AnalysisRunCancelRequest::new("tepp-run-1", "idem-1").expect("request")
    }

    #[test]
    fn cancel_request_round_trips_and_refuses_hostile_shapes() {
        let request = sample_request();
        let json = request.to_json().expect("json");
        assert_eq!(
            AnalysisRunCancelRequest::from_json(&json).expect("decode"),
            request
        );
        assert!(!json.contains("rmse"));
        assert!(!json.contains("scientific_acceptance"));

        assert_eq!(
            AnalysisRunCancelRequest::new("", "idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCancelRequest::new("tepp-run-1", ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCancelRequest::new("a".repeat(ANALYSIS_RUN_CANCEL_ID_MAX_LEN + 1), "idem-1"),
            Err(ApiError::LimitExceeded)
        );

        let mut unsupported = request.clone();
        unsupported.contract_version = 9;
        assert_eq!(
            unsupported.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunCancelRequest::from_json(
                r#"{"contract_version":9,"run_id":"tepp-run-1","idempotency_key":"idem-1"}"#
            ),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunCancelRequest::from_json(
                r#"{"contract_version":1,"run_id":"tepp-run-1","idempotency_key":"idem-1","extra":true}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCancelRequest::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        let mut oversized_json = request.clone();
        oversized_json.idempotency_key = "x".repeat(DEFAULT_ANALYSIS_RUN_BYTE_LIMIT);
        assert_eq!(oversized_json.to_json(), Err(ApiError::LimitExceeded));
        assert_eq!(
            AnalysisRunCancelRequest::from_json("[1,2,3]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCancelRequest::from_json("not-json"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn cancel_payloads_refuse_scientific_metric_keys() {
        assert_eq!(refuse_metrics_on_cancel_payload(""), Ok(()));
        assert_eq!(refuse_metrics_on_cancel_payload("   "), Ok(()));
        assert_eq!(
            refuse_metrics_on_cancel_payload(r#"{"run_id":"r"}"#),
            Ok(())
        );
        for key in FORBIDDEN_CANCEL_KEYS {
            let payload = format!(r#"{{"{key}":1,"run_id":"r"}}"#);
            assert_eq!(
                refuse_metrics_on_cancel_payload(&payload),
                Err(ApiError::InvalidWirePayload),
                "key={key}"
            );
            let with_contract = format!(
                r#"{{"contract_version":1,"run_id":"tepp-run-1","idempotency_key":"idem-1","{key}":0}}"#
            );
            assert_eq!(
                AnalysisRunCancelRequest::from_json(&with_contract),
                Err(ApiError::InvalidWirePayload),
                "dto key={key}"
            );
        }
        assert_eq!(
            refuse_metrics_on_cancel_payload("[true]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_cancel_payload("null"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn cancel_path_decodes_identities_and_refuses_hostile_segments() {
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/tepp-run-1/cancel").expect("plain"),
            "tepp-run-1"
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/run%2dabc/cancel").expect("lower"),
            "run-abc"
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/run%2Dabc/cancel").expect("upper"),
            "run-abc"
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/tepp-run-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/tepp-run-1/running"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/tepp-run-1/terminal"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/other/tepp-run-1/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs//cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/a/b/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/%2F/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/%00/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/%/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/%2/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/%2G/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/run space/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id("/v1/analysis-runs/%80/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized = format!(
            "/v1/analysis-runs/{}/cancel",
            "a".repeat(ANALYSIS_RUN_CANCEL_ID_MAX_LEN + 1)
        );
        assert_eq!(
            analysis_run_cancel_path_run_id(&oversized),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(decode_path_segment(""), Err(ApiError::InvalidWirePayload));
        assert_eq!(decode_path_segment("%"), Err(ApiError::InvalidWirePayload));
        assert_eq!(decode_path_segment("%2"), Err(ApiError::InvalidWirePayload));
        assert_eq!(
            decode_path_segment("%2g"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(from_hex(b'g'), Err(ApiError::InvalidWirePayload));
        assert_eq!(from_hex(b'0'), Ok(0));
        assert_eq!(from_hex(b'a'), Ok(10));
        assert_eq!(from_hex(b'F'), Ok(15));
    }

    #[test]
    fn cancel_exchange_posts_https_path_without_credentials() {
        let request = sample_request();
        let exchange = naruon_analysis_run_cancel_exchange("https://tepp.example.com", &request)
            .expect("exchange");
        assert_eq!(exchange.method, "POST");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.com/v1/analysis-runs/tepp-run-1/cancel"
        );
        assert!(!exchange.body.is_empty());
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
                .any(|(name, _)| name.contains("authorization") || name.contains("copilot"))
        );

        let encoded = AnalysisRunCancelRequest::new("run/../../etc", "key").expect("encoded");
        let exchange = naruon_analysis_run_cancel_exchange("https://tepp.example.com", &encoded)
            .expect("encoded exchange");
        assert!(exchange.target_url.contains("run%2F..%2F..%2Fetc/cancel"));

        assert_eq!(
            naruon_analysis_run_cancel_exchange("http://tepp.example.com", &request),
            Err(ApiError::InvalidWirePayload)
        );
        let mut empty_id = request.clone();
        empty_id.run_id.clear();
        assert_eq!(
            naruon_analysis_run_cancel_exchange("https://tepp.example.com", &empty_id),
            Err(ApiError::InvalidWirePayload)
        );
        let mut oversized = request;
        oversized.run_id = "a".repeat(ANALYSIS_RUN_CANCEL_ID_MAX_LEN + 1);
        assert_eq!(
            naruon_analysis_run_cancel_exchange("https://tepp.example.com", &oversized),
            Err(ApiError::LimitExceeded)
        );
    }
}
