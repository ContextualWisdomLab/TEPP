//! Provider-owned export idempotency-key lookup GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/exports/by-idempotency/{idempotency_key}`
//! returns the metric-free identity of the unique naruon export that used that
//! idempotency key on `AnalysisRunLiveService` / `tepp-loopback`. Retrieval GET
//! requires an `export_id`. Collection GET is a different stack. Operators who
//! hold a 200 authorization receipt or log key cannot jump to that export
//! without scanning identities. `NaruonLiveService` stays POST-only.
//! `LineageWeave` is refused on this naruon-owned adapter.
//! `tepp.scientific_acceptance.v1` never appears. This module does not
//! duplicate GET-by-id (#411), retrieval CLI (#417), collection GET/CLI
//! (#443/#444), stored-request GET/CLI (#457/#459), export-authorize CLI
//! (#410), analysis-run lookup GET (#380), or cancel lineages (closed).
//! Persistence remains GAP-003B. GAP-010 Figma/export remains later work.

use crate::export_http::{EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE, EXPORT_RETRIEVAL_ID_MAX_LEN};
use crate::naruon_http::{NARUON_EXPORT_PATH, NaruonHttpExchange, compose_https_target};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT};
use serde::{Deserialize, Serialize};

/// Maximum length accepted for an opaque idempotency key in the lookup path.
pub const EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN: usize = EXPORT_RETRIEVAL_ID_MAX_LEN;

/// Supported export idempotency-lookup contract version.
pub const EXPORT_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION: u16 = 1;

/// Reserved collection-relative prefix that names the lookup resource.
pub const EXPORT_IDEMPOTENCY_LOOKUP_PREFIX: &str = "by-idempotency";

const FORBIDDEN_EXPORT_LOOKUP_KEYS: [&str; 16] = [
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
    "principal_id",
    "includes_source_text",
];

/// Metric-free identity of one authorized export found by idempotency key.
///
/// Operators jump from a 200 authorization receipt or log key to the durable
/// `export_id` without scanning a collection. The payload never carries a
/// terminal result, source body, or scientific-acceptance artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportIdempotencyLookup {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Opaque server-assigned export identity.
    pub export_id: String,
    /// Stable machine-readable authorization decision code.
    pub decision_code: String,
    /// Exact per-export idempotency key that selected this identity.
    pub idempotency_key: String,
}

impl ExportIdempotencyLookup {
    /// Construct a validated metric-free export idempotency-lookup payload.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, an oversized
    /// identity, an unsupported contract version, or a decision other than
    /// purpose-bound export allowed.
    pub fn new(
        export_id: impl Into<String>,
        decision_code: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let lookup = Self {
            contract_version: EXPORT_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION,
            export_id: export_id.into(),
            decision_code: decision_code.into(),
            idempotency_key: idempotency_key.into(),
        };
        lookup.validate()?;
        Ok(lookup)
    }

    /// Parse and validate an export lookup payload with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate an export lookup payload with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_export_idempotency_lookup_payload(payload)?;
        let lookup: Self = from_json(payload)?;
        lookup.validate()?;
        Ok(lookup)
    }

    /// Serialize this lookup payload after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_export_idempotency_lookup_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            EXPORT_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION,
        )?;
        require_nonempty(&self.export_id)?;
        require_nonempty(&self.decision_code)?;
        require_nonempty(&self.idempotency_key)?;
        if self.decision_code != EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE {
            return Err(ApiError::AuthorizationDenied);
        }
        if self.export_id.contains('/')
            || self.export_id.contains('\0')
            || self.idempotency_key.contains('\0')
        {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.export_id == EXPORT_IDEMPOTENCY_LOOKUP_PREFIX
            || self.idempotency_key == EXPORT_IDEMPOTENCY_LOOKUP_PREFIX
        {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.export_id.len() > EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN
            || self.idempotency_key.len() > EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN
        {
            return Err(ApiError::LimitExceeded);
        }
        Ok(())
    }
}

/// Refuse export-lookup JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted for the GET request body. Non-object JSON
/// fails closed as invalid wire.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present at any nesting depth or the payload is a non-empty non-object.
pub fn refuse_metrics_on_export_idempotency_lookup_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    if !value.is_object() {
        return Err(ApiError::InvalidWirePayload);
    }
    if contains_forbidden_export_lookup_key(&value) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

fn contains_forbidden_export_lookup_key(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(object) => object.iter().any(|(key, value)| {
            FORBIDDEN_EXPORT_LOOKUP_KEYS.contains(&key.as_str())
                || contains_forbidden_export_lookup_key(value)
        }),
        serde_json::Value::Array(values) => values.iter().any(contains_forbidden_export_lookup_key),
        _ => false,
    }
}

/// Extract the opaque idempotency key from
/// `GET /v1/exports/by-idempotency/{key}`.
///
/// The route is segmented before percent decoding, so an encoded `/` remains
/// data inside one opaque key rather than becoming an extra path segment.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a collection path, GET-by-id,
/// extra raw segments, a missing `by-idempotency` prefix, stored-request
/// `/request` suffix, a reserved prefix used as the key, a NUL byte, or a
/// hostile encoding, and [`ApiError::LimitExceeded`] when the decoded key
/// exceeds [`EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN`].
pub(crate) fn export_idempotency_lookup_path_key(path: &str) -> Result<String, ApiError> {
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
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let key = decode_path_segment(encoded)?;
    require_nonempty(&key)?;
    if key == EXPORT_IDEMPOTENCY_LOOKUP_PREFIX {
        return Err(ApiError::InvalidWirePayload);
    }
    if key.len() > EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(key)
}

/// Build a provider-owned `GET` export idempotency-lookup exchange.
///
/// The builder refuses non-`https` origins and empty or oversized keys. It
/// does not inject credentials. The GET body is empty. The opaque key is
/// percent-encoded into exactly one path segment; the builder does not send an
/// `idempotency-key` header.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin, empty
/// key, NUL-containing key, or the reserved lookup prefix, and
/// [`ApiError::LimitExceeded`] when the key exceeds
/// [`EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN`] bytes.
pub fn naruon_export_idempotency_lookup_exchange(
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
    let target_path =
        format!("{NARUON_EXPORT_PATH}/{EXPORT_IDEMPOTENCY_LOOKUP_PREFIX}/{encoded_key}");
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
    use super::*;

    fn sample_lookup() -> ExportIdempotencyLookup {
        ExportIdempotencyLookup::new("export-1", EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE, "idem-1")
            .expect("lookup")
    }

    #[test]
    fn export_idempotency_lookup_round_trips_and_refuses_hostile_shapes() {
        let lookup = sample_lookup();
        let json = lookup.to_json().expect("json");
        assert_eq!(
            ExportIdempotencyLookup::from_json(&json).expect("decode"),
            lookup
        );
        assert!(!json.contains("rmse"));
        assert!(!json.contains("scientific_acceptance"));
        assert!(!json.contains("terminal_result"));
        assert!(!json.contains("tenant_workspace_id"));
        assert!(!json.contains("principal_id"));
        assert!(!json.contains("includes_source_text"));
        assert!(!json.contains("artifact_id"));

        assert_eq!(
            ExportIdempotencyLookup::new("", EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE, "idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ExportIdempotencyLookup::new("export-1", EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE, ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ExportIdempotencyLookup::new("export-1", "denied", "idem-1"),
            Err(ApiError::AuthorizationDenied)
        );
        assert!(
            ExportIdempotencyLookup::new(
                "export-1",
                EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                "scope/key"
            )
            .is_ok()
        );
        assert_eq!(
            ExportIdempotencyLookup::new(
                "a".repeat(EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1),
                EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                "idem-1",
            ),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            ExportIdempotencyLookup::new(
                "export-1",
                EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                "a".repeat(EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1),
            ),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            ExportIdempotencyLookup::new(
                EXPORT_IDEMPOTENCY_LOOKUP_PREFIX,
                EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                "idem-1",
            ),
            Err(ApiError::InvalidWirePayload)
        );

        let mut unsupported = lookup.clone();
        unsupported.contract_version = 9;
        assert_eq!(
            unsupported.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            ExportIdempotencyLookup::from_json(
                r#"{"contract_version":9,"export_id":"export-1","decision_code":"purpose_bound_export_allowed","idempotency_key":"idem-1"}"#
            ),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            ExportIdempotencyLookup::from_json(
                r#"{"contract_version":1,"export_id":"export-1","decision_code":"purpose_bound_export_allowed","idempotency_key":"idem-1","extra":true}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ExportIdempotencyLookup::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            ExportIdempotencyLookup::from_json("[1,2,3]"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn export_idempotency_lookup_payloads_refuse_scientific_metric_keys() {
        assert_eq!(
            refuse_metrics_on_export_idempotency_lookup_payload(""),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_export_idempotency_lookup_payload("   "),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_export_idempotency_lookup_payload(r#"{"export_id":"e"}"#),
            Ok(())
        );
        for key in FORBIDDEN_EXPORT_LOOKUP_KEYS {
            let payload = format!(r#"{{"{key}":1,"export_id":"e"}}"#);
            assert_eq!(
                refuse_metrics_on_export_idempotency_lookup_payload(&payload),
                Err(ApiError::InvalidWirePayload),
                "key={key}"
            );
        }
        assert_eq!(
            refuse_metrics_on_export_idempotency_lookup_payload(
                r#"{"safe":{"nested":{"rmse":1.0}}}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_export_idempotency_lookup_payload(
                r#"{"safe":[{"nested":{"scientific_acceptance":{}}}]}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_export_idempotency_lookup_payload(r#"{"safe":[{"value":1}]}"#),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_export_idempotency_lookup_payload("[true]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_export_idempotency_lookup_payload("null"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn export_idempotency_lookup_path_decodes_keys_and_refuses_hostile_segments() {
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/idem-1").expect("plain"),
            "idem-1"
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/key%2dabc")
                .expect("lower"),
            "key-abc"
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/key%2Dabc")
                .expect("upper"),
            "key-abc"
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/scope%2Fkey")
                .expect("encoded slash remains opaque key data"),
            "scope/key"
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/export-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/export-1/request"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/export-1/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/analysis-runs/by-idempotency/idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/a/b"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/%2F").expect("slash"),
            "/"
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/%00"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_idempotency_lookup_path_key("/v1/exports/by-idempotency/by-idempotency"),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized = format!(
            "/v1/exports/by-idempotency/{}",
            "a".repeat(EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1)
        );
        assert_eq!(
            export_idempotency_lookup_path_key(&oversized),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(decode_path_segment(""), Err(ApiError::InvalidWirePayload));
        assert_eq!(from_hex(b'0'), Ok(0));
        assert_eq!(from_hex(b'a'), Ok(10));
        assert_eq!(from_hex(b'F'), Ok(15));
    }
}
