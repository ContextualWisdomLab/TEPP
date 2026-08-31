//! Provider-owned analysis-run status/read HTTP exchange contracts.
//!
//! This module exposes a fail-closed `GET /v1/analysis-runs/{run_id}` exchange
//! builder so modular consumers (, ) can poll accepted runs
//! without inventing routes, retry coefficients, or credential headers
//! locally. TEPP remains the sole authority for lifecycle state, terminal
//! results, and evidence binding.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target, standard_headers};
use crate::wire::require_nonempty;
use crate::{ANALYSIS_RUN_STATUS_PATH, ApiError};

/// Maximum length accepted for an opaque run identity in the status path.
pub const ANALYSIS_RUN_ID_MAX_LEN: usize = 128;

/// Parsed loopback analysis-run route after percent-decoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AnalysisRunLiveRoute {
    /// `GET /v1/analysis-runs/{run_id}`
    Status { run_id: String },
    /// `POST /v1/analysis-runs/{run_id}/running`
    Running { run_id: String },
    /// `POST /v1/analysis-runs/{run_id}/terminal`
    Terminal { run_id: String },
}

/// Build a provider-owned `GET` analysis-run status exchange.
///
/// The caller supplies the TEPP origin and the opaque server-assigned run
/// identity returned in the accepted receipt. The builder refuses non-`https`
/// origins and empty or oversized run identifiers but performs no credential
/// injection: the caller must supply its own authorization header through the
/// transport layer if the deployment requires it.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin, a
/// table-access URL, an empty run identifier, or a run identifier exceeding
/// [`ANALYSIS_RUN_ID_MAX_LEN`] bytes.
pub fn naruon_analysis_run_status_exchange(
    origin: &str,
    run_id: &str,
    idempotency_key: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    let target_path = encoded_run_path(run_id, None)?;
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "GET",
        target_url,
        headers: standard_headers(idempotency_key),
        body: String::new(),
    })
}

fn encoded_run_path(run_id: &str, suffix: Option<&str>) -> Result<String, ApiError> {
    require_nonempty(run_id)?;
    if run_id.len() > ANALYSIS_RUN_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_run_id = encode_path_segment(run_id);
    match suffix {
        None => Ok(format!("{ANALYSIS_RUN_STATUS_PATH}/{encoded_run_id}")),
        Some(suffix) => Ok(format!(
            "{ANALYSIS_RUN_STATUS_PATH}/{encoded_run_id}/{suffix}"
        )),
    }
}

/// Percent-encode one `URI` path segment without double-encoding safe chars.
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

/// Decode one status-path segment and refuse empty, slash, or hostile values.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for truncated encodings, non-UTF-8
/// octets, empty results, or a decoded slash/NUL.
pub(crate) fn decode_path_segment(value: &str) -> Result<String, ApiError> {
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

fn require_run_id_length(run_id: &str) -> Result<(), ApiError> {
    if run_id.len() > ANALYSIS_RUN_ID_MAX_LEN {
        Err(ApiError::LimitExceeded)
    } else {
        Ok(())
    }
}

/// Parse `GET` status and `POST` running/terminal loopback routes.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a collection path, extra
/// segments, an unknown suffix, or a hostile encoding, and
/// [`ApiError::LimitExceeded`] when the decoded identity exceeds
/// [`ANALYSIS_RUN_ID_MAX_LEN`].
pub(crate) fn parse_analysis_run_live_route(path: &str) -> Result<AnalysisRunLiveRoute, ApiError> {
    let remainder = path
        .strip_prefix(ANALYSIS_RUN_STATUS_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    match encoded.split_once('/') {
        None => {
            let run_id = decode_path_segment(encoded)?;
            require_run_id_length(&run_id)?;
            Ok(AnalysisRunLiveRoute::Status { run_id })
        }
        Some((encoded_id, "running")) => {
            let run_id = decode_path_segment(encoded_id)?;
            require_run_id_length(&run_id)?;
            Ok(AnalysisRunLiveRoute::Running { run_id })
        }
        Some((encoded_id, "terminal")) => {
            let run_id = decode_path_segment(encoded_id)?;
            require_run_id_length(&run_id)?;
            Ok(AnalysisRunLiveRoute::Terminal { run_id })
        }
        Some(_) => Err(ApiError::InvalidWirePayload),
    }
}

/// Extract the opaque run identity from `GET /v1/analysis-runs/{run_id}`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a collection path, extra
/// segments, or a hostile encoding, and [`ApiError::LimitExceeded`] when the
/// decoded identity exceeds [`ANALYSIS_RUN_ID_MAX_LEN`].
pub(crate) fn analysis_run_status_path_run_id(path: &str) -> Result<String, ApiError> {
    match parse_analysis_run_live_route(path)? {
        AnalysisRunLiveRoute::Status { run_id } => Ok(run_id),
        AnalysisRunLiveRoute::Running { .. } | AnalysisRunLiveRoute::Terminal { .. } => {
            Err(ApiError::InvalidWirePayload)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_get_exchange_for_valid_origin_and_run_id() {
        let exchange = naruon_analysis_run_status_exchange(
            "https://tepp.example.com",
            "run-abc-123",
            "idem-key-1",
        )
        .expect("valid origin and run id");
        assert_eq!(exchange.method, "GET");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.com/v1/analysis-runs/run-abc-123"
        );
        assert!(exchange.body.is_empty());
        assert!(!exchange.headers.is_empty());
    }

    #[test]
    fn percent_encodes_unsafe_characters_in_run_id() {
        let exchange =
            naruon_analysis_run_status_exchange("https://tepp.example.com", "run/../../etc", "key")
                .expect("unsafe chars are encoded not rejected");
        assert!(exchange.target_url.contains("run%2F..%2F..%2Fetc"));
    }

    #[test]
    fn refuses_http_origin() {
        let result = naruon_analysis_run_status_exchange("http://tepp.example.com", "run-1", "k");
        assert_eq!(result.unwrap_err(), ApiError::InvalidWirePayload);
    }

    #[test]
    fn refuses_empty_run_id() {
        let result = naruon_analysis_run_status_exchange("https://t.example.com", "", "k");
        assert_eq!(result.unwrap_err(), ApiError::InvalidWirePayload);
    }

    #[test]
    fn refuses_oversized_run_id() {
        let big = "a".repeat(ANALYSIS_RUN_ID_MAX_LEN + 1);
        let result = naruon_analysis_run_status_exchange("https://t.example.com", &big, "k");
        assert_eq!(result.unwrap_err(), ApiError::LimitExceeded);
    }

    #[test]
    fn decodes_status_path_identities_and_refuses_hostile_segments() {
        assert_eq!(
            analysis_run_status_path_run_id("/v1/analysis-runs/tepp-run-1").expect("plain"),
            "tepp-run-1"
        );
        assert_eq!(
            decode_path_segment("run%2dabc").expect("lower hex"),
            "run-abc"
        );
        assert_eq!(
            decode_path_segment("run%2Dabc").expect("upper hex"),
            "run-abc"
        );
        assert_eq!(
            analysis_run_status_path_run_id("/v1/analysis-runs"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_status_path_run_id("/v1/other/tepp-run-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_status_path_run_id("/v1/analysis-runs/"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_status_path_run_id("/v1/analysis-runs/a/b"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_status_path_run_id("/v1/analysis-runs/%2F"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(decode_path_segment("%"), Err(ApiError::InvalidWirePayload));
        assert_eq!(decode_path_segment("%2"), Err(ApiError::InvalidWirePayload));
        assert_eq!(
            decode_path_segment("%ZZ"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            decode_path_segment("run id"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            decode_path_segment("%00"),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized = "a".repeat(ANALYSIS_RUN_ID_MAX_LEN + 1);
        assert_eq!(
            analysis_run_status_path_run_id(&format!("/v1/analysis-runs/{oversized}")),
            Err(ApiError::LimitExceeded)
        );
        let invalid_utf8 = "%FF";
        assert_eq!(
            decode_path_segment(invalid_utf8),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            analysis_run_status_path_run_id("/v1/analysis-runs/tepp-run-1/running"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_analysis_run_live_route("/v1/analysis-runs/tepp-run-1/running").expect("running"),
            AnalysisRunLiveRoute::Running {
                run_id: "tepp-run-1".into()
            }
        );
        assert_eq!(
            parse_analysis_run_live_route("/v1/analysis-runs/tepp-run-1/terminal")
                .expect("terminal"),
            AnalysisRunLiveRoute::Terminal {
                run_id: "tepp-run-1".into()
            }
        );
        assert_eq!(
            parse_analysis_run_live_route("/v1/analysis-runs/tepp-run-1/running/extra"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_analysis_run_live_route(&format!(
                "/v1/analysis-runs/{}/running",
                "a".repeat(ANALYSIS_RUN_ID_MAX_LEN + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            parse_analysis_run_live_route(&format!(
                "/v1/analysis-runs/{}/terminal",
                "a".repeat(ANALYSIS_RUN_ID_MAX_LEN + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            parse_analysis_run_live_route("/v1/analysis-runs/%2F/running"),
            Err(ApiError::InvalidWirePayload)
        );
        let _ = encoded_run_path("tepp-run-1", Some("running")).expect("suffix path");
    }
}
