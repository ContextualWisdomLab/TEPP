"""Expose the TEPP project-history projection through the shared live service."""

from __future__ import annotations

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    """Replace one exact source anchor or accept an already-applied edit."""
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    if new in text:
        return
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one anchor, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def append_once(path: str, marker: str, addition: str) -> None:
    """Append a test block once after confirming its source marker remains."""
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    if addition in text:
        return
    if marker not in text:
        raise SystemExit(f"{path}: append marker is missing")
    target.write_text(f"{text.rstrip()}\n\n{addition.rstrip()}\n", encoding="utf-8")


def main() -> None:
    """Patch routing, bounds, response generation, and live contract tests."""
    source = "crates/tepp_api/src/analysis_run_live.rs"
    contract_test = "crates/tepp_api/tests/lineageweave_project_history_contract.rs"

    replace_once(
        source,
        """//! Consumer-neutral live analysis-run ingress for modular CWL services.
//!
//! This module keeps the Naruon compatibility listener intact while providing
//! the shared `/v1/analysis-runs` boundary needed by Naruon and `LineageWeave`.
//! It accepts transport acknowledgements only; completed psychometric results
//! remain outside this crate.
""",
        """//! Consumer-neutral live TEPP ingress for modular CWL services.
//!
//! This module keeps the Naruon compatibility listener intact while providing
//! shared `/v1/analysis-runs` and `/v1/project-histories` boundaries. Analysis
//! runs return transport acknowledgements only. Project histories return a
//! deterministic projection over authorized evidence supplied by `LineageWeave`;
//! neither path claims a completed psychometric result or causal conclusion.
""",
    )
    replace_once(
        source,
        "use crate::lineageweave_http::consumer_is_supported;\n",
        "use crate::lineageweave_http::{LINEAGEWEAVE_CONSUMER_CODE, consumer_is_supported};\n",
    )
    replace_once(
        source,
        "use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, header_is_credential};\n",
        "use crate::naruon_http::header_is_credential;\n",
    )
    replace_once(
        source,
        """use crate::{
    AnalysisRunAccepted, AnalysisRunRequest, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
    ErrorEnvelope, NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT,
    NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse, requests_are_idempotent_matches,
};
""",
        """use crate::{
    AnalysisRunAccepted, AnalysisRunRequest, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
    DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, ErrorEnvelope, NARUON_ANALYSIS_RUN_PATH,
    NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT, NARUON_LIVE_IO_TIMEOUT,
    NaruonLiveResponse, PROJECT_HISTORY_PATH, ProjectHistoryRequest, project_history_projection,
    requests_are_idempotent_matches,
};

const LIVE_BODY_BYTE_LIMIT: usize = DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;
""",
    )
    replace_once(
        source,
        """/// Loopback HTTP/1.1 analysis-run service shared by published CWL consumers.
///
/// The service accepts only Naruon and `LineageWeave` consumer identities. Its
/// idempotency namespace includes consumer, tenant, and caller key so one
/// product cannot replay or conflict with another product's accepted run.
""",
        """/// Loopback HTTP/1.1 TEPP service shared by published CWL consumers.
///
/// The analysis-run path accepts Naruon and `LineageWeave` and scopes mutable
/// acknowledgement idempotency by consumer, tenant, and caller key. The
/// project-history path accepts `LineageWeave` only and computes a stateless,
/// cutoff-safe projection from the bounded request body.
""",
    )
    replace_once(
        source,
        """        let mut lines = header_block.split("\r\n");
        require_request_line(lines.next().unwrap_or(""))?;
        let headers = parse_headers(lines)?;
        let consumer = require_headers(&headers, self.bound_addr)?;
        self.accept_analysis_run(consumer, &headers, body)
""",
        """        let mut lines = header_block.split("\r\n");
        let request_path = require_request_line(lines.next().unwrap_or(""))?;
        let headers = parse_headers(lines)?;
        let consumer = require_headers(&headers, self.bound_addr)?;
        if request_path == NARUON_ANALYSIS_RUN_PATH {
            self.accept_analysis_run(consumer, &headers, body)
        } else {
            Self::project_history(consumer, &headers, body)
        }
""",
    )
    replace_once(
        source,
        """    fn response_from_error(&mut self, error: ApiError) -> NaruonLiveResponse {
""",
        """    fn project_history(
        consumer: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        if consumer != LINEAGEWEAVE_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        let request = ProjectHistoryRequest::from_json(body)?;
        if header_value(headers, "idempotency-key")? != request.idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        let projection = project_history_projection(&request)?;
        Ok(json_response(200, "OK", projection.to_json()?))
    }

    fn response_from_error(&mut self, error: ApiError) -> NaruonLiveResponse {
""",
    )
    replace_once(
        source,
        """    if content_length > DEFAULT_ANALYSIS_RUN_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
""",
        """    if content_length > LIVE_BODY_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
""",
    )
    replace_once(
        source,
        """    if declared > DEFAULT_ANALYSIS_RUN_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
""",
        """    if declared > LIVE_BODY_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
""",
    )
    replace_once(
        source,
        """fn require_request_line(line: &str) -> Result<(), ApiError> {
    let mut parts = line.split(' ');
    if parts.next() != Some("POST")
        || parts.next() != Some(NARUON_ANALYSIS_RUN_PATH)
        || parts.next() != Some("HTTP/1.1")
        || parts.next().is_some()
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}
""",
        """fn require_request_line(line: &str) -> Result<&str, ApiError> {
    let mut parts = line.split(' ');
    if parts.next() != Some("POST") {
        return Err(ApiError::InvalidWirePayload);
    }
    let path = parts.next().ok_or(ApiError::InvalidWirePayload)?;
    if (path != NARUON_ANALYSIS_RUN_PATH && path != PROJECT_HISTORY_PATH)
        || parts.next() != Some("HTTP/1.1")
        || parts.next().is_some()
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(path)
}
""",
    )

    replace_once(
        contract_test,
        """use tepp_api::{
    ApiError, LINEAGEWEAVE_CONSUMER_CODE, PROJECT_HISTORY_CONTRACT_VERSION, PROJECT_HISTORY_PATH,
    ProjectHistoryEvent, ProjectHistoryRequest, lineageweave_project_history_exchange,
    project_history_projection,
};
""",
        """use tepp_api::{
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE,
    PROJECT_HISTORY_CONTRACT_VERSION, PROJECT_HISTORY_PATH, ProjectHistoryEvent,
    ProjectHistoryProjection, ProjectHistoryRequest, lineageweave_project_history_exchange,
    project_history_projection,
};
""",
    )
    append_once(
        contract_test,
        "fn lineageweave_exchange_uses_the_versioned_credential_free_tepp_path()",
        r'''#[test]
fn shared_live_service_returns_the_project_history_and_rejects_other_consumers() {
    let request = sample_request();
    let body = request.to_json().expect("request json");
    let raw = format!(
        "POST {PROJECT_HISTORY_PATH} HTTP/1.1\r\nHost: localhost\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        request.idempotency_key,
        body.len(),
    );

    let mut service = AnalysisRunLiveService::new();
    let response = service.handle_http_request(&raw);
    assert_eq!(response.status_code, 200);
    let projection = ProjectHistoryProjection::from_json(&response.body).expect("projection");
    assert_eq!(projection.focus_event_id, request.focus_event_id);
    assert_eq!(projection.inference_status, "temporal_association_only");

    let naruon = raw.replace(
        &format!("tepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}"),
        "tepp-consumer: naruon",
    );
    assert_eq!(service.handle_http_request(&naruon).status_code, 400);

    let mismatched = raw.replace(
        &format!("idempotency-key: {}", request.idempotency_key),
        "idempotency-key: another-key",
    );
    assert_eq!(service.handle_http_request(&mismatched).status_code, 400);
}''',
    )


if __name__ == "__main__":
    main()
