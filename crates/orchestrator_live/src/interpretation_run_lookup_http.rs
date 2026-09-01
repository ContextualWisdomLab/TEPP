//! Provider-owned interpretation-run lookup GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}`
//! returns the metric-free identity of the unique accepted hypothetical run
//! that used that server-assigned `interpretation_run_id` on
//! `OrchestratorLiveService` / `tepp-orchestrator-loopback`. GET-by-id
//! requires the client `idempotency_key`. Collection GET is a different
//! stack. Operators who hold a 202 acceptance receipt or log `orch-run-N`
//! cannot jump to that identity without scanning pages. Rows stay
//! `claim_status=hypothetical` and `scientific_authority=false`.
//! `tepp.scientific_acceptance.v1` never appears. The lookup does not infer
//! causality or call a model provider. This module does not duplicate
//! GET-by-id (#438), retrieval CLI (#439), collection GET/CLI (#433/#436),
//! stored-request GET/CLI (#453/#454), create CLI (#425), export lookup
//! (#466), analysis-run lookup GET (#380), or cancel lineages (closed).
//! Persistence remains GAP-003B. Naruon and `LineageWeave` are refused.
//! `NaruonLiveService` stays POST-only.

use crate::error::OrchestratorLiveError;
use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
use crate::interpretation_run_retrieval_http::INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN;
use crate::request::{INTERPRETATION_RUN_PATH, host_implies_table_access, require_nonempty};

/// Reserved collection-relative prefix that names the lookup resource.
pub const INTERPRETATION_RUN_LOOKUP_PREFIX: &str = "by-run-id";

/// Maximum opaque `interpretation_run_id` length on the lookup path.
pub const INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN: usize = INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN;

/// Typed GET exchange for interpretation-run lookup by server-assigned id.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpretationRunLookupHttpExchange {
    /// HTTP method, always `GET`.
    pub method: &'static str,
    /// Absolute HTTPS target ending in `/v1/interpretation-runs/by-run-id/{id}`.
    pub target_url: String,
    /// Exact version, consumer, and content headers. No credentials.
    pub headers: Vec<(String, String)>,
    /// GET body, always empty.
    pub body: String,
}

/// Extract the opaque `interpretation_run_id` from
/// `GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}`.
///
/// The route is segmented before percent decoding, so an encoded `/` remains
/// data inside one opaque identity rather than becoming an extra path segment.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] for collection,
/// GET-by-id, extra raw segments, a missing `by-run-id` prefix, stored-request
/// `/request` suffix, reserved prefix used as the id, slash, NUL, empty
/// identity, or a hostile encoding, and
/// [`OrchestratorLiveError::LimitExceeded`] when oversized.
pub fn interpretation_run_lookup_path_id(path: &str) -> Result<String, OrchestratorLiveError> {
    let remainder = path
        .strip_prefix(INTERPRETATION_RUN_PATH)
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_prefix(INTERPRETATION_RUN_LOOKUP_PREFIX)
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_prefix('/')
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let interpretation_run_id = decode_path_segment(encoded)?;
    require_nonempty(&interpretation_run_id)?;
    if interpretation_run_id == INTERPRETATION_RUN_LOOKUP_PREFIX {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if interpretation_run_id.contains('/') || interpretation_run_id.contains('\0') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if interpretation_run_id.len() > INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    Ok(interpretation_run_id)
}

/// Whether `path` is the lookup-by-run-id resource.
#[must_use]
pub fn is_interpretation_run_lookup_path(path: &str) -> bool {
    interpretation_run_lookup_path_id(path).is_ok()
}

/// Build a credential-free contextual-orchestrator lookup GET exchange.
///
/// The builder refuses non-`https` origins and empty or oversized identities.
/// It does not inject credentials. The GET body is empty. The opaque id is
/// percent-encoded into exactly one path segment after `by-run-id`.
///
/// # Errors
///
/// Returns a fail-closed origin or identity error.
pub fn contextual_orchestrator_interpretation_run_lookup_exchange(
    origin: &str,
    interpretation_run_id: &str,
) -> Result<InterpretationRunLookupHttpExchange, OrchestratorLiveError> {
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
    require_nonempty(interpretation_run_id)?;
    if interpretation_run_id == INTERPRETATION_RUN_LOOKUP_PREFIX {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if interpretation_run_id.contains('/') || interpretation_run_id.contains('\0') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if interpretation_run_id.len() > INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    let encoded_id = encode_path_segment(interpretation_run_id);
    Ok(InterpretationRunLookupHttpExchange {
        method: "GET",
        target_url: format!(
            "{origin}{INTERPRETATION_RUN_PATH}/{INTERPRETATION_RUN_LOOKUP_PREFIX}/{encoded_id}"
        ),
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
        INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN, INTERPRETATION_RUN_LOOKUP_PREFIX,
        contextual_orchestrator_interpretation_run_lookup_exchange,
        interpretation_run_lookup_path_id, is_interpretation_run_lookup_path,
    };
    use crate::error::OrchestratorLiveError;

    #[test]
    fn lookup_exchange_is_metric_free_get_without_credentials() {
        let exchange = contextual_orchestrator_interpretation_run_lookup_exchange(
            "https://tepp.example.test",
            "orch-run-1",
        )
        .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(
            exchange
                .target_url
                .ends_with("/v1/interpretation-runs/by-run-id/orch-run-1")
        );
        assert!(exchange.body.is_empty());
        assert!(
            !exchange
                .headers
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                    || name.eq_ignore_ascii_case("idempotency-key"))
        );
        assert!(is_interpretation_run_lookup_path(
            "/v1/interpretation-runs/by-run-id/orch-run-1"
        ));
        assert!(!is_interpretation_run_lookup_path(
            "/v1/interpretation-runs/orch-run-1"
        ));
        assert!(!is_interpretation_run_lookup_path(
            "/v1/interpretation-runs/by-run-id/orch-run-1/request"
        ));
        assert_eq!(
            interpretation_run_lookup_path_id("/v1/interpretation-runs/by-run-id/orch-run-1")
                .expect("id"),
            "orch-run-1"
        );
        assert_eq!(INTERPRETATION_RUN_LOOKUP_PREFIX, "by-run-id");
    }

    #[test]
    fn lookup_path_and_origins_fail_closed() {
        assert_eq!(
            interpretation_run_lookup_path_id("/v1/interpretation-runs"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_lookup_path_id("/v1/interpretation-runs/by-run-id"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_lookup_path_id("/v1/interpretation-runs/by-run-id/"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_lookup_path_id("/v1/interpretation-runs/orch-run-1"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_lookup_path_id("/v1/interpretation-runs/by-run-id/orch-run-1/extra"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_lookup_path_id(
                "/v1/interpretation-runs/by-run-id/orch-run-1/request"
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_lookup_path_id("/v1/interpretation-runs/by-run-id/by-run-id"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_lookup_path_id("/v1/interpretation-runs/by-run-id/orch%2Fslash"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_lookup_path_id("/v1/interpretation-runs/by-run-id/%00"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            interpretation_run_lookup_path_id(&format!(
                "/v1/interpretation-runs/by-run-id/{}",
                "a".repeat(INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN + 1)
            )),
            Err(OrchestratorLiveError::LimitExceeded)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_lookup_exchange(
                "http://insecure.example",
                "orch-run-1",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_lookup_exchange(
                "https://postgres.example.test",
                "orch-run-1",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_lookup_exchange(
                "https://tepp.example.test",
                "by-run-id",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_lookup_exchange(
                "https://tepp.example.test",
                "orch/slash",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            contextual_orchestrator_interpretation_run_lookup_exchange(
                "https://tepp.example.test",
                "",
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
    }
}
