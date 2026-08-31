//! Published modular-consumer identity and `LineageWeave` TEPP exchanges.

use crate::naruon_http::compose_https_target;
use crate::project_history::build_project_history_exchange;
use crate::{
    AnalysisRunRequest, AnalysisRunRetryRequest, ApiError, NaruonHttpExchange,
    ProjectHistoryHttpExchange, ProjectHistoryRequest, TEMPORAL_CONTEXT_CONTRACT_VERSION,
    TEMPORAL_CONTEXT_PATH, TemporalContextRequest, naruon_analysis_run_exchange,
    naruon_analysis_run_retry_exchange,
};

/// Stable consumer identity used by the Naruon adapter.
pub const NARUON_CONSUMER_CODE: &str = "naruon";

/// Stable consumer identity used by the `LineageWeave` adapter.
pub const LINEAGEWEAVE_CONSUMER_CODE: &str = "lineageweave";

/// Build a `LineageWeave` → TEPP analysis-run exchange without provider credentials.
///
/// The function reuses TEPP's existing origin, body, and header validation,
/// then replaces only the published modular-consumer identity. The accepted
/// response remains an asynchronous transport acknowledgement, not a completed
/// psychometric result.
///
/// # Errors
///
/// Returns the same fail-closed errors as [`naruon_analysis_run_exchange`].
pub fn lineageweave_analysis_run_exchange(
    origin: &str,
    request: &AnalysisRunRequest,
) -> Result<NaruonHttpExchange, ApiError> {
    let mut exchange = naruon_analysis_run_exchange(origin, request)?;
    swap_consumer_header(&mut exchange)?;
    Ok(exchange)
}

/// Build a `LineageWeave` → TEPP analysis-run retry exchange without credentials.
///
/// The function reuses TEPP's existing origin, body, and header validation,
/// then replaces only the published modular-consumer identity. Retry remains a
/// metric-free clone of a failed or cancelled run, not a measurement result.
///
/// # Errors
///
/// Returns the same fail-closed errors as [`naruon_analysis_run_retry_exchange`].
pub fn lineageweave_analysis_run_retry_exchange(
    origin: &str,
    request: &AnalysisRunRetryRequest,
) -> Result<NaruonHttpExchange, ApiError> {
    let mut exchange = naruon_analysis_run_retry_exchange(origin, request)?;
    swap_consumer_header(&mut exchange)?;
    Ok(exchange)
}

/// Build a credential-free `LineageWeave` temporal-context exchange.
///
/// # Errors
///
/// Returns a fail-closed error for a hostile origin or invalid temporal-context
/// request.
pub fn lineageweave_temporal_context_exchange(
    origin: &str,
    request: &TemporalContextRequest,
) -> Result<NaruonHttpExchange, ApiError> {
    let target_url = compose_https_target(origin, TEMPORAL_CONTEXT_PATH)?;
    let body = request.to_json()?;
    let headers = vec![
        ("content-type".into(), "application/json".into()),
        ("tepp-consumer".into(), LINEAGEWEAVE_CONSUMER_CODE.into()),
        (
            "tepp-contract-version".into(),
            TEMPORAL_CONTEXT_CONTRACT_VERSION.to_string(),
        ),
    ];
    Ok(NaruonHttpExchange {
        method: "POST",
        target_url,
        headers,
        body,
    })
}

/// Build a `LineageWeave` → TEPP project-history exchange without credentials.
///
/// The request contains only bounded source evidence selected after
/// `LineageWeave` authorization. TEPP validates the cutoff and returns a
/// deterministic temporal-association projection, never a causal score.
///
/// # Errors
///
/// Returns a fail-closed origin, request, version, size, or timestamp error.
pub fn lineageweave_project_history_exchange(
    origin: &str,
    request: &ProjectHistoryRequest,
) -> Result<ProjectHistoryHttpExchange, ApiError> {
    build_project_history_exchange(origin, LINEAGEWEAVE_CONSUMER_CODE, request)
}

/// Return whether a modular analysis-run consumer is published by TEPP.
pub(crate) fn consumer_is_supported(consumer_code: &str) -> bool {
    matches!(
        consumer_code,
        NARUON_CONSUMER_CODE | LINEAGEWEAVE_CONSUMER_CODE
    )
}

fn swap_consumer_header(exchange: &mut NaruonHttpExchange) -> Result<(), ApiError> {
    let consumer_header = exchange
        .headers
        .iter_mut()
        .find(|(name, _)| name.eq_ignore_ascii_case("tepp-consumer"))
        .ok_or(ApiError::InvalidWirePayload)?;
    LINEAGEWEAVE_CONSUMER_CODE.clone_into(&mut consumer_header.1);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, consumer_is_supported,
        lineageweave_analysis_run_exchange, lineageweave_analysis_run_retry_exchange,
    };
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunRequest, AnalysisRunRetryRequest, ApiError,
    };

    fn sample_run() -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "idem-1".into(),
            tenant_workspace_id: "tenant-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "tepp-analysis-run-v1".into(),
            output_profile: "calibrated_event_measurement".into(),
        }
    }

    #[test]
    fn supported_consumer_set_is_closed() {
        assert!(consumer_is_supported(NARUON_CONSUMER_CODE));
        assert!(consumer_is_supported(LINEAGEWEAVE_CONSUMER_CODE));
        assert!(!consumer_is_supported("unknown"));
    }

    #[test]
    fn lineageweave_exchange_preserves_existing_fail_closed_validation() {
        let run = sample_run();
        let exchange = lineageweave_analysis_run_exchange("https://tepp.example.test", &run)
            .expect("exchange");
        assert!(
            exchange
                .headers
                .contains(&("tepp-consumer".into(), LINEAGEWEAVE_CONSUMER_CODE.into()))
        );
        assert_eq!(
            lineageweave_analysis_run_exchange("http://tepp.example.test", &run),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn lineageweave_retry_exchange_swaps_only_the_consumer_header() {
        let request = AnalysisRunRetryRequest::new("tepp-run-1", "idem-retry-1").expect("retry");
        let exchange =
            lineageweave_analysis_run_retry_exchange("https://tepp.example.test", &request)
                .expect("retry exchange");
        assert_eq!(exchange.method, "POST");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.test/v1/analysis-runs/tepp-run-1/retry"
        );
        assert!(
            exchange
                .headers
                .contains(&("tepp-consumer".into(), LINEAGEWEAVE_CONSUMER_CODE.into()))
        );
        assert!(
            exchange
                .headers
                .contains(&("idempotency-key".into(), "idem-retry-1".into()))
        );
        assert!(exchange.headers.iter().all(|(name, _)| {
            !matches!(
                name.to_ascii_lowercase().as_str(),
                "authorization" | "proxy-authorization" | "cookie" | "x-api-key"
            )
        }));
        assert_eq!(
            lineageweave_analysis_run_retry_exchange("http://tepp.example.test", &request),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
