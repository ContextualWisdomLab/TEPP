//! Provider-owned analysis-run idempotency-key lookup GET contracts.
//!
//! GAP-003A eleventh slice: `GET /v1/analysis-runs/by-idempotency/{key}`
//! returns the metric-free identity of the unique run that used that
//! idempotency key. Collection GET is cursor-paginated. Stored-request GET
//! and retry-lineage GET require a `run_id`. Retry HTTP mints a new key.
//! Operators with a 202 receipt or log key cannot jump to that run without
//! scanning pages. This module does not serve GET-by-id (#359), lifecycle
//! POST (#360), cancel HTTP (#361), loopback CLI (#362), collection GET
//! (#368), retry POST (#369), stored-request GET (#377), retry-lineage GET
//! (#379), or cancel CLI (#378). Persistence remains GAP-003B.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{
    ANALYSIS_RUN_STATUS_PATH, AnalysisRunStatusState, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
};
use serde::{Deserialize, Serialize};

/// Maximum length accepted for an opaque idempotency key in the lookup path.
pub const ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN: usize = 128;

/// Supported analysis-run idempotency-lookup contract version.
pub const ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION: u16 = 1;

/// Reserved collection-relative prefix that names the lookup resource.
pub const ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_PREFIX: &str = "by-idempotency";

const FORBIDDEN_IDEMPOTENCY_LOOKUP_KEYS: [&str; 14] = [
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

/// Metric-free identity of one analysis run found by idempotency key.
///
/// Operators jump from a 202 receipt or log key to the durable `run_id`
/// without scanning a cursor-paginated collection. The payload never carries
/// a terminal result, snapshot, or scientific-acceptance artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunIdempotencyLookup {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Opaque server-assigned run identity.
    pub run_id: String,
    /// Current lifecycle state.
    pub run_state: AnalysisRunStatusState,
    /// Exact request idempotency key that selected this run.
    pub idempotency_key: String,
}

impl AnalysisRunIdempotencyLookup {
    /// Construct a validated metric-free idempotency-lookup payload.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, an oversized
    /// identity, or an unsupported contract version.
    pub fn new(
        run_id: impl Into<String>,
        run_state: AnalysisRunStatusState,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let lookup = Self {
            contract_version: ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION,
            run_id: run_id.into(),
            run_state,
            idempotency_key: idempotency_key.into(),
        };
        lookup.validate()?;
        Ok(lookup)
    }

    /// Parse and validate an idempotency-lookup payload with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate an idempotency-lookup payload with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_idempotency_lookup_payload(payload)?;
        let lookup: Self = from_json(payload)?;
        lookup.validate()?;
        Ok(lookup)
    }

    /// Serialize this idempotency-lookup payload after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_idempotency_lookup_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION,
        )?;
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.run_id.len() > ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN
            || self.idempotency_key.len() > ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN
        {
            return Err(ApiError::LimitExceeded);
        }
        Ok(())
    }
}

/// Refuse idempotency-lookup JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted for the GET request body. Non-object JSON
/// fails closed as invalid wire.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present or the payload is a non-empty non-object.
pub fn refuse_metrics_on_idempotency_lookup_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    let Some(object) = value.as_object() else {
        return Err(ApiError::InvalidWirePayload);
    };
    if FORBIDDEN_IDEMPOTENCY_LOOKUP_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

/// Extract the opaque idempotency key from
/// `GET /v1/analysis-runs/by-idempotency/{key}`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a collection path, GET-by-id,
/// extra segments, a missing `by-idempotency` prefix, cancel/retry/request/
/// retries/running/terminal suffixes, or a hostile encoding, and
/// [`ApiError::LimitExceeded`] when the decoded key exceeds
/// [`ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN`].
pub(crate) fn analysis_run_idempotency_lookup_path_key(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(ANALYSIS_RUN_STATUS_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_prefix(ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_PREFIX)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let key = decode_path_segment(encoded)?;
    if key == ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_PREFIX {
        return Err(ApiError::InvalidWirePayload);
    }
    if key.len() > ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(key)
}

/// Build a provider-owned `GET` analysis-run idempotency-lookup exchange.
///
/// The builder refuses non-`https` origins and empty or oversized keys. It
/// does not inject credentials. The GET body is empty. The key travels in
/// the path; the builder does not send an `idempotency-key` header.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// key, and [`ApiError::LimitExceeded`] when the key exceeds
/// [`ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN`] bytes.
pub fn naruon_analysis_run_idempotency_lookup_exchange(
    origin: &str,
    idempotency_key: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    require_nonempty(idempotency_key)?;
    if idempotency_key.len() > ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_key = encode_path_segment(idempotency_key);
    let target_path = format!(
        "{ANALYSIS_RUN_STATUS_PATH}/{ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_PREFIX}/{encoded_key}"
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

    fn sample_lookup() -> AnalysisRunIdempotencyLookup {
        AnalysisRunIdempotencyLookup::new("tepp-run-1", AnalysisRunStatusState::Failed, "idem-1")
            .expect("lookup")
    }

    #[test]
    fn idempotency_lookup_round_trips_and_refuses_hostile_shapes() {
        let lookup = sample_lookup();
        let json = lookup.to_json().expect("json");
        assert_eq!(
            AnalysisRunIdempotencyLookup::from_json(&json).expect("decode"),
            lookup
        );
        assert!(!json.contains("rmse"));
        assert!(!json.contains("scientific_acceptance"));
        assert!(!json.contains("terminal_result"));
        assert!(!json.contains("tenant_workspace_id"));
        assert!(!json.contains("snapshot_id"));
        assert!(!json.contains("retried_from"));

        assert_eq!(
            AnalysisRunIdempotencyLookup::new("", AnalysisRunStatusState::Failed, "idem-1",),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunIdempotencyLookup::new("tepp-run-1", AnalysisRunStatusState::Failed, "",),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunIdempotencyLookup::new(
                "a".repeat(ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1),
                AnalysisRunStatusState::Failed,
                "idem-1",
            ),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            AnalysisRunIdempotencyLookup::new(
                "tepp-run-1",
                AnalysisRunStatusState::Failed,
                "a".repeat(ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1),
            ),
            Err(ApiError::LimitExceeded)
        );

        let mut unsupported = lookup.clone();
        unsupported.contract_version = 9;
        assert_eq!(
            unsupported.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunIdempotencyLookup::from_json(
                r#"{"contract_version":9,"run_id":"tepp-run-1","run_state":"failed","idempotency_key":"idem-1"}"#
            ),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunIdempotencyLookup::from_json(
                r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"failed","idempotency_key":"idem-1","extra":true}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunIdempotencyLookup::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            AnalysisRunIdempotencyLookup::from_json("[1,2,3]"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn idempotency_lookup_payloads_refuse_scientific_metric_keys() {
        assert_eq!(refuse_metrics_on_idempotency_lookup_payload(""), Ok(()));
        assert_eq!(refuse_metrics_on_idempotency_lookup_payload("   "), Ok(()));
        assert_eq!(
            refuse_metrics_on_idempotency_lookup_payload(r#"{"run_id":"r"}"#),
            Ok(())
        );
        for key in FORBIDDEN_IDEMPOTENCY_LOOKUP_KEYS {
            let payload = format!(r#"{{"{key}":1,"run_id":"r"}}"#);
            assert_eq!(
                refuse_metrics_on_idempotency_lookup_payload(&payload),
                Err(ApiError::InvalidWirePayload),
                "key={key}"
            );
        }
        assert_eq!(
            refuse_metrics_on_idempotency_lookup_payload("[true]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_idempotency_lookup_payload("null"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn idempotency_lookup_path_decodes_keys_and_refuses_hostile_segments() {
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/by-idempotency/idem-1")
                .expect("plain"),
            "idem-1"
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/by-idempotency/key%2dabc")
                .expect("lower"),
            "key-abc"
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/by-idempotency/key%2Dabc")
                .expect("upper"),
            "key-abc"
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/tepp-run-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/by-idempotency"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/by-idempotency/"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/tepp-run-1/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/tepp-run-1/retry"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/tepp-run-1/retries"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/tepp-run-1/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/tepp-run-1/running"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/tepp-run-1/terminal"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/other/by-idempotency/idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/by-idempotency/a/b"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/by-idempotency/%2F"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key("/v1/analysis-runs/by-idempotency/%00"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key(
                "/v1/analysis-runs/by-idempotency/by-idempotency"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized = format!(
            "/v1/analysis-runs/by-idempotency/{}",
            "a".repeat(ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1)
        );
        assert_eq!(
            analysis_run_idempotency_lookup_path_key(&oversized),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(decode_path_segment(""), Err(ApiError::InvalidWirePayload));
        assert_eq!(from_hex(b'0'), Ok(0));
        assert_eq!(from_hex(b'a'), Ok(10));
        assert_eq!(from_hex(b'F'), Ok(15));
    }

    #[test]
    fn idempotency_lookup_exchange_gets_https_path_without_credentials() {
        let exchange =
            naruon_analysis_run_idempotency_lookup_exchange("https://tepp.example.com", "idem-1")
                .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.com/v1/analysis-runs/by-idempotency/idem-1"
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

        let encoded = naruon_analysis_run_idempotency_lookup_exchange(
            "https://tepp.example.com",
            "key/../../etc",
        )
        .expect("encoded");
        assert!(
            encoded
                .target_url
                .contains("by-idempotency/key%2F..%2F..%2Fetc")
        );

        assert_eq!(
            naruon_analysis_run_idempotency_lookup_exchange("http://tepp.example.com", "idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_idempotency_lookup_exchange("https://tepp.example.com", ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_analysis_run_idempotency_lookup_exchange(
                "https://tepp.example.com",
                &"a".repeat(ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1)
            ),
            Err(ApiError::LimitExceeded)
        );
    }
}
