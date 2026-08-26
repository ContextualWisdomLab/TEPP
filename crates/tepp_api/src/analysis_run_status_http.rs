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
    require_nonempty(run_id)?;
    if run_id.len() > ANALYSIS_RUN_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_run_id = encode_path_segment(run_id);
    let target_path = format!("{ANALYSIS_RUN_STATUS_PATH}/{encoded_run_id}");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "GET",
        target_url,
        headers: standard_headers(idempotency_key),
        body: String::new(),
    })
}

/// Percent-encode one `URI` path segment without double-encoding safe chars.
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
        let exchange = naruon_analysis_run_status_exchange(
            "https://tepp.example.com",
            "run/../../etc",
            "key",
        )
        .expect("unsafe chars are encoded not rejected");
        assert!(exchange.target_url.contains("run%2F..%2F..%2Fetc"));
    }

    #[test]
    fn refuses_http_origin() {
        let result =
            naruon_analysis_run_status_exchange("http://tepp.example.com", "run-1", "k");
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
}
