//! Consumer-neutral live analysis-run ingress for modular CWL services.
//!
//! This module keeps the Naruon compatibility listener intact while providing
//! the shared `/v1/analysis-runs` and cutoff-safe `/v1/temporal-context`
//! boundaries needed by Naruon and `LineageWeave`. `POST /v1/analysis-runs`
//! accepts transport acknowledgements only. `GET /v1/analysis-runs/{run_id}`
//! returns metric-free accepted/running status, and may return
//! `tepp.scientific_acceptance.v1` only on a succeeded status whose request
//! profile is `scientific_acceptance_v1`. Completed psychometric estimation
//! remains outside this crate.

use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpListener};

use crate::analysis_run_status_http::analysis_run_status_path_run_id;
use crate::lineageweave_http::{LINEAGEWEAVE_CONSUMER_CODE, consumer_is_supported};
use crate::live_http::{
    header_value, map_io_error, parse_headers, parse_request_line, read_http_request_with_limit,
    split_request_with_limit, validate_common_headers,
};
use crate::naruon_http::NARUON_ANALYSIS_RUN_PATH;
use crate::scientific_acceptance_http::{refuse_metrics_on_receipt, status_http_json};
use crate::{
    AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunStatus, ApiError,
    DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, ErrorEnvelope, NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse,
    PROJECT_HISTORY_PATH, ProjectHistoryProjection, ProjectHistoryRequest, TEMPORAL_CONTEXT_PATH,
    TemporalContextRequest, build_temporal_context, project_history_projection,
    requests_are_idempotent_matches,
};

#[cfg(test)]
use crate::require_status_binding;

const MAX_LIVE_REQUEST_BODY_BYTES: usize = DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;

#[cfg(test)]
use crate::live_http::{declared_content_length, host_implies_table_access, split_header_line};

/// One accepted loopback analysis run and its current status/read body.
#[derive(Clone, Debug, Eq, PartialEq)]
struct LiveAnalysisRun {
    consumer: String,
    request: AnalysisRunRequest,
    accepted: AnalysisRunAccepted,
    status: AnalysisRunStatus,
    scientific_acceptance_json: Option<String>,
}

/// Loopback HTTP/1.1 analysis-run service shared by published CWL consumers.
///
/// The service accepts only Naruon and `LineageWeave` consumer identities. Its
/// idempotency namespace includes consumer, tenant, and caller key so one
/// product cannot replay or conflict with another product's accepted run.
#[derive(Debug)]
pub struct AnalysisRunLiveService {
    listener: Option<TcpListener>,
    bound_addr: Option<SocketAddr>,
    next_run_serial: u64,
    next_request_serial: u64,
    accepted_runs: HashMap<String, LiveAnalysisRun>,
    runs_by_id: HashMap<String, String>,
    accepted_project_histories: HashMap<String, (ProjectHistoryRequest, ProjectHistoryProjection)>,
}

impl Default for AnalysisRunLiveService {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisRunLiveService {
    /// Construct an in-memory handler with no bound socket.
    #[must_use]
    pub fn new() -> Self {
        Self {
            listener: None,
            bound_addr: None,
            next_run_serial: 1,
            next_request_serial: 1,
            accepted_runs: HashMap::new(),
            runs_by_id: HashMap::new(),
            accepted_project_histories: HashMap::new(),
        }
    }

    /// Bind an ephemeral IPv4 loopback port.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when the operating system
    /// refuses the loopback bind.
    pub fn bind_loopback() -> Result<Self, ApiError> {
        Self::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
    }

    /// Bind a caller-supplied loopback address.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback address
    /// and [`ApiError::InvalidWirePayload`] when the socket cannot be opened.
    pub fn bind(addr: SocketAddr) -> Result<Self, ApiError> {
        if !addr.ip().is_loopback() {
            return Err(ApiError::AuthorizationDenied);
        }
        let listener = TcpListener::bind(addr).map_err(|error| map_io_error(&error))?;
        let bound_addr = listener
            .local_addr()
            .map_err(|error| map_io_error(&error))?;
        Ok(Self {
            listener: Some(listener),
            bound_addr: Some(bound_addr),
            ..Self::new()
        })
    }

    /// Return the bound loopback address.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when no socket is bound.
    pub fn local_addr(&self) -> Result<SocketAddr, ApiError> {
        self.bound_addr.ok_or(ApiError::InvalidWirePayload)
    }

    /// Accept and serve one HTTP/1.1 request.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed API error when no socket is bound or socket I/O
    /// fails. Protocol errors are returned as redacted HTTP responses.
    pub fn serve_one(&mut self) -> Result<NaruonLiveResponse, ApiError> {
        let listener = self.listener.as_ref().ok_or(ApiError::InvalidWirePayload)?;
        let (mut stream, _) = listener.accept().map_err(|error| map_io_error(&error))?;
        stream
            .set_read_timeout(Some(NARUON_LIVE_IO_TIMEOUT))
            .map_err(|error| map_io_error(&error))?;
        stream
            .set_write_timeout(Some(NARUON_LIVE_IO_TIMEOUT))
            .map_err(|error| map_io_error(&error))?;
        let response = match read_http_request_with_limit(&mut stream, MAX_LIVE_REQUEST_BODY_BYTES)
        {
            Ok(request) => self.handle_http_request(&request),
            Err(error) => self.response_from_error(error),
        };
        stream
            .write_all(&response.to_http_bytes())
            .map_err(|error| map_io_error(&error))?;
        stream.flush().map_err(|error| map_io_error(&error))?;
        Ok(response)
    }

    /// Parse and handle one complete HTTP/1.1 request already in memory.
    #[must_use]
    pub fn handle_http_request(&mut self, request: &str) -> NaruonLiveResponse {
        match self.dispatch_http_request(request) {
            Ok(response) => response,
            Err(error) => self.response_from_error(error),
        }
    }

    fn dispatch_http_request(&mut self, request: &str) -> Result<NaruonLiveResponse, ApiError> {
        let (header_block, body) = split_request_with_limit(request, MAX_LIVE_REQUEST_BODY_BYTES)?;
        let mut lines = header_block.split("\r\n");
        let (method, path) = parse_request_line(lines.next().unwrap_or(""))?;
        let headers = parse_headers(&mut lines)?;
        if method == "GET" {
            return self.read_analysis_run_status(path, &headers, body);
        }
        if method != "POST"
            || (path != NARUON_ANALYSIS_RUN_PATH
                && path != TEMPORAL_CONTEXT_PATH
                && path != PROJECT_HISTORY_PATH)
        {
            return Err(ApiError::InvalidWirePayload);
        }
        let consumer = require_headers(
            &headers,
            self.bound_addr,
            path == NARUON_ANALYSIS_RUN_PATH || path == PROJECT_HISTORY_PATH,
        )?;
        if path == TEMPORAL_CONTEXT_PATH {
            if consumer != LINEAGEWEAVE_CONSUMER_CODE {
                return Err(ApiError::InvalidWirePayload);
            }
            let context_request = TemporalContextRequest::from_json(body)?;
            let response = build_temporal_context(&context_request)?;
            return Ok(json_response(200, "OK", response.to_json()?));
        }
        if path == PROJECT_HISTORY_PATH {
            return self.accept_project_history(consumer, &headers, body);
        }
        self.accept_analysis_run(consumer, &headers, body)
    }

    fn accept_analysis_run(
        &mut self,
        consumer: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        refuse_metrics_on_receipt(body)?;
        let request = AnalysisRunRequest::from_json(body)?;
        let idempotency_key = header_value(headers, "idempotency-key")?;
        if idempotency_key != request.idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        let replay_key = consumer_tenant_idempotency_key(
            consumer,
            &request.tenant_workspace_id,
            idempotency_key,
        );
        if let Some(stored) = self.accepted_runs.get(&replay_key) {
            if requests_are_idempotent_matches(&stored.request, &request) {
                let accepted_json = stored.accepted.to_json()?;
                refuse_metrics_on_receipt(&accepted_json)?;
                return Ok(json_response(202, "Accepted", accepted_json));
            }
            return Err(ApiError::InvalidWirePayload);
        }
        let run_id = format!("tepp-run-{}", self.next_run_serial);
        self.next_run_serial += 1;
        if self.runs_by_id.contains_key(&run_id) {
            return Err(ApiError::InvalidWirePayload);
        }
        let accepted =
            AnalysisRunAccepted::new(run_id.clone(), "accepted", request.idempotency_key.clone())?;
        let accepted_json = accepted.to_json()?;
        refuse_metrics_on_receipt(&accepted_json)?;
        let status = AnalysisRunStatus::accepted(&accepted)?;
        self.runs_by_id.insert(run_id, replay_key.clone());
        self.accepted_runs.insert(
            replay_key,
            LiveAnalysisRun {
                consumer: consumer.to_owned(),
                request,
                accepted,
                status,
                scientific_acceptance_json: None,
            },
        );
        Ok(json_response(202, "Accepted", accepted_json))
    }

    fn read_analysis_run_status(
        &mut self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        if !body.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        let run_id = analysis_run_status_path_run_id(path)?;
        let consumer = require_headers(headers, self.bound_addr, true)?;
        let idempotency_key = header_value(headers, "idempotency-key")?;
        let replay_key = self
            .runs_by_id
            .get(&run_id)
            .ok_or(ApiError::InvalidWirePayload)?;
        let stored = self
            .accepted_runs
            .get(replay_key)
            .ok_or(ApiError::InvalidWirePayload)?;
        if stored.consumer != consumer || stored.accepted.idempotency_key != idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        let response_body = status_http_json(
            &stored.status,
            &stored.request,
            stored.scientific_acceptance_json.as_deref(),
        )?;
        Ok(json_response(200, "OK", response_body))
    }

    /// Record a loopback lifecycle transition for an already accepted run.
    ///
    /// Tests and the in-memory listener use this helper because completed
    /// psychometric execution remains outside this crate. HTTP GET is the only
    /// operator-visible status path.
    #[cfg(test)]
    pub(crate) fn record_loopback_status(
        &mut self,
        run_id: &str,
        status: AnalysisRunStatus,
        scientific_acceptance_json: Option<String>,
    ) -> Result<(), ApiError> {
        let replay_key = self
            .runs_by_id
            .get(run_id)
            .ok_or(ApiError::InvalidWirePayload)?
            .clone();
        let stored = self
            .accepted_runs
            .get_mut(&replay_key)
            .ok_or(ApiError::InvalidWirePayload)?;
        if stored.accepted.run_id != run_id {
            return Err(ApiError::InvalidWirePayload);
        }
        require_status_binding(&stored.request, &stored.accepted, &status)?;
        let _ = status_http_json(
            &status,
            &stored.request,
            scientific_acceptance_json.as_deref(),
        )?;
        stored.status = status;
        stored.scientific_acceptance_json = scientific_acceptance_json;
        Ok(())
    }

    fn accept_project_history(
        &mut self,
        consumer: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        if consumer != LINEAGEWEAVE_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        let request = ProjectHistoryRequest::from_json(body)?;
        let idempotency_key = header_value(headers, "idempotency-key")?;
        if idempotency_key != request.idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        let replay_key = consumer_tenant_idempotency_key(
            consumer,
            &request.tenant_workspace_id,
            idempotency_key,
        );
        if let Some((stored_request, stored_projection)) =
            self.accepted_project_histories.get(&replay_key)
        {
            if stored_request == &request {
                return Ok(json_response(200, "OK", stored_projection.to_json()?));
            }
            return Err(ApiError::InvalidWirePayload);
        }
        let projection = project_history_projection(&request)?;
        let response_body = projection.to_json()?;
        self.accepted_project_histories
            .insert(replay_key, (request, projection));
        Ok(json_response(200, "OK", response_body))
    }

    fn response_from_error(&mut self, error: ApiError) -> NaruonLiveResponse {
        let request_id = format!("analysis-run-live-{}", self.next_request_serial);
        self.next_request_serial += 1;
        let (status_code, reason_phrase) = status_for(error);
        let body = error_envelope_json(error, request_id);
        json_response(status_code, reason_phrase, body)
    }
}

fn require_headers(
    headers: &HashMap<String, String>,
    bound_addr: Option<SocketAddr>,
    require_idempotency_key: bool,
) -> Result<&str, ApiError> {
    validate_common_headers(headers, bound_addr)?;
    if header_value(headers, "tepp-contract-version")? != "1" {
        return Err(ApiError::InvalidWirePayload);
    }
    let consumer = header_value(headers, "tepp-consumer")?;
    if !consumer_is_supported(consumer) {
        return Err(ApiError::InvalidWirePayload);
    }
    if require_idempotency_key {
        let _idempotency_key = header_value(headers, "idempotency-key")?;
    }
    Ok(consumer)
}

fn consumer_tenant_idempotency_key(
    consumer: &str,
    tenant_workspace_id: &str,
    idempotency_key: &str,
) -> String {
    format!("{consumer}\u{1f}{tenant_workspace_id}\u{1f}{idempotency_key}")
}

fn status_for(error: ApiError) -> (u16, &'static str) {
    match error {
        ApiError::InvalidWirePayload => (400, "Bad Request"),
        ApiError::AuthorizationDenied => (403, "Forbidden"),
        ApiError::LimitExceeded => (413, "Payload Too Large"),
        ApiError::UnsupportedContractVersion => (422, "Unprocessable Entity"),
    }
}

fn error_envelope_json(error: ApiError, request_id: String) -> String {
    ErrorEnvelope::from_api_error(error, request_id)
        .and_then(|envelope| envelope.to_json())
        .unwrap_or_else(|_| {
            "{\"error_code\":\"invalid_wire_payload\",\"message\":\"invalid API wire payload\",\"request_id\":\"analysis-run-live-fallback\",\"retryable\":false}".to_owned()
        })
}

fn json_response(
    status_code: u16,
    reason_phrase: &'static str,
    body: String,
) -> NaruonLiveResponse {
    NaruonLiveResponse {
        status_code,
        reason_phrase,
        body,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Write as _;
    use std::io::{Cursor, Read, Write};
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;

    use super::{
        AnalysisRunLiveService, consumer_tenant_idempotency_key, declared_content_length,
        error_envelope_json, host_implies_table_access, map_io_error, parse_headers,
        require_headers, split_header_line, status_for,
    };
    use crate::live_http::{host_is_loopback, read_http_request, split_request};
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisResultSummary, AnalysisRunAccepted,
        AnalysisRunRequest, AnalysisRunStatus, AnalysisRunTerminalResult, ApiError,
        DEFAULT_ANALYSIS_RUN_BYTE_LIMIT, ErrorEnvelope, LINEAGEWEAVE_CONSUMER_CODE,
        NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE, NARUON_LIVE_HEADER_BYTE_LIMIT,
        NARUON_LIVE_HEADER_COUNT_LIMIT, NARUON_LIVE_IO_TIMEOUT, SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
        SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA, TEMPORAL_CONTEXT_PATH,
    };
    use sha2::{Digest, Sha256};

    fn sample_run() -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "analysis-live-idem-001".into(),
            tenant_workspace_id: "analysis-live-tenant".into(),
            snapshot_id: "analysis-live-snapshot".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "tepp-analysis-run-v1".into(),
            output_profile: "calibrated_event_measurement".into(),
        }
    }

    fn http_request(body: &str, headers: &[(&str, &str)]) -> String {
        let mut request = format!("POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\n");
        for (name, value) in headers {
            write!(request, "{name}: {value}\r\n").expect("header");
        }
        write!(request, "content-length: {}\r\n\r\n{body}", body.len()).expect("body");
        request
    }

    fn http_get(path: &str, headers: &[(&str, &str)]) -> String {
        let mut request = format!("GET {path} HTTP/1.1\r\n");
        for (name, value) in headers {
            write!(request, "{name}: {value}\r\n").expect("header");
        }
        request.push_str("content-length: 0\r\n\r\n");
        request
    }

    fn status_get(run_id: &str, consumer: &str, idempotency_key: &str) -> String {
        http_get(
            &format!("{NARUON_ANALYSIS_RUN_PATH}/{run_id}"),
            &[
                ("Host", "127.0.0.1"),
                ("content-type", "application/json"),
                ("tepp-consumer", consumer),
                ("tepp-contract-version", "1"),
                ("idempotency-key", idempotency_key),
            ],
        )
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let digest = Sha256::digest(bytes);
        let mut encoded = String::with_capacity(64);
        for byte in digest {
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        encoded
    }

    fn valid_request(run: &AnalysisRunRequest, consumer: &str, host: &str) -> String {
        let body = run.to_json().expect("run json");
        http_request(
            &body,
            &[
                ("Host", host),
                ("content-type", "application/json"),
                ("tepp-consumer", consumer),
                ("tepp-contract-version", "1"),
                ("idempotency-key", run.idempotency_key.as_str()),
            ],
        )
    }

    fn envelope(body: &str) -> ErrorEnvelope {
        serde_json::from_str(body).expect("error envelope")
    }

    #[test]
    fn helper_contracts_cover_consumer_identity_and_loopback_ports() {
        assert_eq!(
            consumer_tenant_idempotency_key(LINEAGEWEAVE_CONSUMER_CODE, "tenant", "key"),
            "lineageweave\u{1f}tenant\u{1f}key"
        );
        assert_ne!(
            consumer_tenant_idempotency_key(LINEAGEWEAVE_CONSUMER_CODE, "tenant", "key"),
            consumer_tenant_idempotency_key(NARUON_CONSUMER_CODE, "tenant", "key")
        );
        assert!(host_is_loopback("localhost:8080", None));
        assert!(!host_is_loopback("localhost:not-a-port", None));
        assert_eq!(
            AnalysisRunLiveService::bind("0.0.0.0:0".parse().expect("addr")).expect_err("denied"),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunLiveService::new()
                .local_addr()
                .expect_err("unbound"),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn bind_and_error_helpers_cover_loopback_and_fail_closed_edges() {
        let default_service = AnalysisRunLiveService::default();
        assert_eq!(
            default_service
                .local_addr()
                .expect_err("default is unbound"),
            ApiError::InvalidWirePayload
        );
        let service = AnalysisRunLiveService::bind_loopback().expect("loopback bind");
        let addr = service.local_addr().expect("bound address");
        assert!(addr.ip().is_loopback());
        assert_eq!(
            AnalysisRunLiveService::bind(addr).expect_err("in-use address"),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunLiveService::new()
                .serve_one()
                .expect_err("unbound serve"),
            ApiError::InvalidWirePayload
        );

        assert_eq!(
            status_for(ApiError::InvalidWirePayload),
            (400, "Bad Request")
        );
        assert_eq!(
            status_for(ApiError::AuthorizationDenied),
            (403, "Forbidden")
        );
        assert_eq!(
            status_for(ApiError::LimitExceeded),
            (413, "Payload Too Large")
        );
        assert_eq!(
            status_for(ApiError::UnsupportedContractVersion),
            (422, "Unprocessable Entity")
        );
        assert_eq!(
            map_io_error(&std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "timeout"
            )),
            ApiError::LimitExceeded
        );
        assert_eq!(
            map_io_error(&std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "would block"
            )),
            ApiError::LimitExceeded
        );
        assert_eq!(
            map_io_error(&std::io::Error::other("broken")),
            ApiError::InvalidWirePayload
        );
        assert!(host_implies_table_access("db.postgres.local"));
        assert!(host_implies_table_access("jdbc.local"));
        assert!(host_implies_table_access("127.0.0.1/sql"));
        assert!(host_implies_table_access("127.0.0.1/tables/x"));
        assert!(host_implies_table_access("bad host"));
        assert!(host_implies_table_access("bad;host"));
        assert!(host_implies_table_access("bad'host"));
        assert!(host_implies_table_access("bad\\host"));
        assert!(host_implies_table_access("bad\u{0001}host"));
        assert!(!host_implies_table_access("127.0.0.1:43789"));
        assert!(
            error_envelope_json(ApiError::InvalidWirePayload, String::new())
                .contains("analysis-run-live-fallback")
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn handler_covers_acceptance_replay_and_header_security() {
        let run = sample_run();
        let body = run.to_json().expect("body");
        let mut service = AnalysisRunLiveService::new();
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
        for request_line in [
            "POST /wrong HTTP/1.1",
            "POST /v1/analysis-runs HTTP/1.0",
            "POST /v1/analysis-runs HTTP/1.1 extra",
        ] {
            assert_eq!(
                service
                    .handle_http_request(&format!("{request_line}\r\ncontent-length: 0\r\n\r\n"))
                    .status_code,
                400
            );
        }

        let naruon =
            service.handle_http_request(&valid_request(&run, NARUON_CONSUMER_CODE, "127.0.0.1"));
        let lineageweave = service.handle_http_request(&valid_request(
            &run,
            LINEAGEWEAVE_CONSUMER_CODE,
            "127.0.0.1",
        ));
        assert_eq!(naruon.status_code, 202);
        assert_eq!(lineageweave.status_code, 202);
        let replay = service.handle_http_request(&valid_request(
            &run,
            LINEAGEWEAVE_CONSUMER_CODE,
            "127.0.0.1",
        ));
        assert_eq!(replay.status_code, 202);
        assert_eq!(replay.body, lineageweave.body);

        let mut conflict = run.clone();
        conflict.snapshot_id = "different-snapshot".into();
        assert_eq!(
            service
                .handle_http_request(&valid_request(
                    &conflict,
                    LINEAGEWEAVE_CONSUMER_CODE,
                    "127.0.0.1",
                ))
                .status_code,
            400
        );

        let mismatch = http_request(
            &body,
            &[
                ("Host", "127.0.0.1"),
                ("content-type", "application/json"),
                ("tepp-consumer", NARUON_CONSUMER_CODE),
                ("tepp-contract-version", "1"),
                ("idempotency-key", "different-key"),
            ],
        );
        assert_eq!(service.handle_http_request(&mismatch).status_code, 400);

        let unsupported = body.replace("\"contract_version\":1", "\"contract_version\":9");
        let unsupported_response = service.handle_http_request(&http_request(
            &unsupported,
            &[
                ("Host", "127.0.0.1"),
                ("content-type", "application/json"),
                ("tepp-consumer", NARUON_CONSUMER_CODE),
                ("tepp-contract-version", "1"),
                ("idempotency-key", run.idempotency_key.as_str()),
            ],
        ));
        assert_eq!(unsupported_response.status_code, 422);
        assert_eq!(
            envelope(&unsupported_response.body).error_code(),
            "unsupported_contract_version"
        );

        for (name, value) in [
            ("authorization", "Bearer secret"),
            ("proxy-authorization", "Basic secret"),
            ("cookie", "session=secret"),
            ("x-api-key", "secret"),
        ] {
            let response = service.handle_http_request(&http_request(
                &body,
                &[
                    ("Host", "127.0.0.1"),
                    ("content-type", "application/json"),
                    ("tepp-consumer", NARUON_CONSUMER_CODE),
                    ("tepp-contract-version", "1"),
                    ("idempotency-key", run.idempotency_key.as_str()),
                    (name, value),
                ],
            ));
            assert_eq!(response.status_code, 403, "header={name}");
            assert!(!response.body.contains(value));
        }

        for (headers, status) in [
            (
                vec![
                    ("content-type", "application/json"),
                    ("tepp-consumer", NARUON_CONSUMER_CODE),
                    ("tepp-contract-version", "1"),
                    ("idempotency-key", run.idempotency_key.as_str()),
                ],
                400,
            ),
            (
                vec![
                    ("Host", ""),
                    ("content-type", "application/json"),
                    ("tepp-consumer", NARUON_CONSUMER_CODE),
                    ("tepp-contract-version", "1"),
                    ("idempotency-key", run.idempotency_key.as_str()),
                ],
                400,
            ),
            (
                vec![
                    ("Host", "127.0.0.1/sql"),
                    ("content-type", "application/json"),
                    ("tepp-consumer", NARUON_CONSUMER_CODE),
                    ("tepp-contract-version", "1"),
                    ("idempotency-key", run.idempotency_key.as_str()),
                ],
                400,
            ),
            (
                vec![
                    ("Host", "8.8.8.8"),
                    ("content-type", "application/json"),
                    ("tepp-consumer", NARUON_CONSUMER_CODE),
                    ("tepp-contract-version", "1"),
                    ("idempotency-key", run.idempotency_key.as_str()),
                ],
                403,
            ),
            (
                vec![
                    ("Host", "127.0.0.1"),
                    ("content-type", "text/plain"),
                    ("tepp-consumer", NARUON_CONSUMER_CODE),
                    ("tepp-contract-version", "1"),
                    ("idempotency-key", run.idempotency_key.as_str()),
                ],
                400,
            ),
            (
                vec![
                    ("Host", "127.0.0.1"),
                    ("content-type", "application/json"),
                    ("tepp-consumer", NARUON_CONSUMER_CODE),
                    ("tepp-contract-version", "2"),
                    ("idempotency-key", run.idempotency_key.as_str()),
                ],
                400,
            ),
            (
                vec![
                    ("Host", "127.0.0.1"),
                    ("content-type", "application/json"),
                    ("tepp-consumer", "unpublished"),
                    ("tepp-contract-version", "1"),
                    ("idempotency-key", run.idempotency_key.as_str()),
                ],
                400,
            ),
            (
                vec![
                    ("Host", "127.0.0.1"),
                    ("content-type", "application/json"),
                    ("tepp-consumer", NARUON_CONSUMER_CODE),
                    ("tepp-contract-version", "1"),
                    ("idempotency-key", ""),
                ],
                400,
            ),
        ] {
            assert_eq!(
                service
                    .handle_http_request(&http_request(&body, &headers))
                    .status_code,
                status
            );
        }

        let transfer = format!(
            "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: key\r\ntransfer-encoding: chunked\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        );
        assert_eq!(service.handle_http_request(&transfer).status_code, 400);
    }

    #[test]
    fn temporal_read_headers_and_defensive_write_edges_are_covered() {
        let run = sample_run();
        let body = run.to_json().expect("body");
        let mut service = AnalysisRunLiveService::new();

        for missing_header in ["tepp-contract-version", "tepp-consumer"] {
            let headers = [
                ("Host", "127.0.0.1"),
                ("content-type", "application/json"),
                ("tepp-consumer", NARUON_CONSUMER_CODE),
                ("tepp-contract-version", "1"),
                ("idempotency-key", run.idempotency_key.as_str()),
            ]
            .into_iter()
            .filter(|(name, _)| *name != missing_header)
            .collect::<Vec<_>>();
            assert_eq!(
                service
                    .handle_http_request(&http_request(&body, &headers))
                    .status_code,
                400,
                "missing={missing_header}"
            );
        }

        let temporal_body = r#"{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":null,"events":[{"event_id":"event-1","source_post_id":"post-1","event_type_code":"order_awarded","event_label":"Order awarded","event_time":"2026-08-01T09:00:00Z","available_time":"2026-08-01T10:00:00Z","project_reference":null,"actor_references":["actor-1"]}]}"#;
        let temporal_request = format!(
            "POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\ncontent-length: {}\r\n\r\n{temporal_body}",
            temporal_body.len()
        );
        assert_eq!(
            service.handle_http_request(&temporal_request).status_code,
            200
        );

        let mut headers = HashMap::from([
            ("host".to_owned(), "127.0.0.1".to_owned()),
            ("content-type".to_owned(), "application/json".to_owned()),
            ("tepp-consumer".to_owned(), NARUON_CONSUMER_CODE.to_owned()),
            ("tepp-contract-version".to_owned(), "1".to_owned()),
        ]);
        assert_eq!(
            service.accept_analysis_run(NARUON_CONSUMER_CODE, &headers, &body),
            Err(ApiError::InvalidWirePayload)
        );
        headers.insert("idempotency-key".to_owned(), run.idempotency_key.clone());
        assert_eq!(
            require_headers(&headers, None, true),
            Ok(NARUON_CONSUMER_CODE)
        );
        headers.insert("tepp-contract-version".to_owned(), "2".to_owned());
        assert_eq!(
            require_headers(&headers, None, true),
            Err(ApiError::InvalidWirePayload)
        );
        headers.insert("tepp-contract-version".to_owned(), "1".to_owned());
        headers.insert("tepp-consumer".to_owned(), "unpublished".to_owned());
        assert_eq!(
            require_headers(&headers, None, true),
            Err(ApiError::InvalidWirePayload)
        );
        headers.insert("tepp-consumer".to_owned(), NARUON_CONSUMER_CODE.to_owned());
        let accepted = service
            .accept_analysis_run(NARUON_CONSUMER_CODE, &headers, &body)
            .expect("accepted");
        assert_eq!(accepted.status_code, 202);
        let replay = service
            .accept_analysis_run(NARUON_CONSUMER_CODE, &headers, &body)
            .expect("replay");
        assert_eq!(replay.body, accepted.body);
    }

    #[test]
    fn parser_helpers_cover_framing_header_and_limit_edges() {
        assert_eq!(
            split_request("").expect_err("empty"),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            split_request(&"x".repeat(NARUON_LIVE_HEADER_BYTE_LIMIT)),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            split_request(&format!(
                "{}\r\n\r\n",
                "x".repeat(NARUON_LIVE_HEADER_BYTE_LIMIT + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
        let oversized_body = "x".repeat(DEFAULT_ANALYSIS_RUN_BYTE_LIMIT + 1);
        assert_eq!(
            split_request(&format!(
                "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\ncontent-length: {}\r\n\r\n{oversized_body}",
                oversized_body.len()
            )),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            split_header_line("NoColon"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line(": empty-name"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line("Bad Name: value"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line("Bad\u{0001}Name: value"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line("Host: 127.0.0.1").expect("header"),
            ("Host", "127.0.0.1")
        );
        assert_eq!(
            declared_content_length(
                "POST /x HTTP/1.1\r\ncontent-length: 1\r\ncontent-length: 1\r\n\r\n"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            declared_content_length("POST /x HTTP/1.1\r\ncontent-length: \r\n\r\n"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            declared_content_length("POST /x HTTP/1.1\r\ncontent-length: +1\r\n\r\n"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            declared_content_length("POST /x HTTP/1.1\r\nHost: 127.0.0.1\r\n"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            declared_content_length(
                "POST /x HTTP/1.1\r\ncontent-length: 999999999999999999999\r\n\r\n"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        let mut crowded = (0..=NARUON_LIVE_HEADER_COUNT_LIMIT)
            .map(|index| Box::leak(format!("x-{index}: value").into_boxed_str()) as &str);
        assert_eq!(parse_headers(&mut crowded), Err(ApiError::LimitExceeded));
        let mut duplicate = ["x-header: one", "X-HEADER: two"].into_iter();
        assert_eq!(
            parse_headers(&mut duplicate),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            split_request("POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: 2\r\n\r\na"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn read_http_request_covers_transport_utf8_and_body_limits() {
        assert_eq!(
            read_http_request(&mut Cursor::new(Vec::<u8>::new())).expect_err("eof"),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            read_http_request(&mut ScriptedRead::error(std::io::ErrorKind::TimedOut))
                .expect_err("timeout"),
            ApiError::LimitExceeded
        );
        assert_eq!(
            read_http_request(&mut ScriptedRead::error(std::io::ErrorKind::Other))
                .expect_err("other"),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            read_http_request(&mut Cursor::new(vec![
                b'x';
                NARUON_LIVE_HEADER_BYTE_LIMIT + 1
            ]))
            .expect_err("header limit"),
            ApiError::LimitExceeded
        );

        let run = sample_run();
        let request = valid_request(&run, NARUON_CONSUMER_CODE, "127.0.0.1");
        assert_eq!(
            read_http_request(&mut ScriptedRead::bytes(request.as_bytes())).expect("request"),
            request
        );
        let zero = request.replace(&run.to_json().expect("body"), "");
        let zero = zero.replace(
            &format!("content-length: {}", run.to_json().expect("body").len()),
            "content-length: 0",
        );
        assert!(read_http_request(&mut Cursor::new(zero.into_bytes())).is_ok());

        let mut invalid_header = b"POST /v1/analysis-runs HTTP/1.1\r\n".to_vec();
        invalid_header.push(0xff);
        invalid_header.extend_from_slice(b"\r\ncontent-length: 0\r\n\r\n");
        assert_eq!(
            read_http_request(&mut Cursor::new(invalid_header)).expect_err("header utf8"),
            ApiError::InvalidWirePayload
        );
        let header = format!(
            "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: 1\r\n\r\n"
        );
        let invalid_body = [header.as_bytes(), &[0xff]].concat();
        assert_eq!(
            read_http_request(&mut Cursor::new(invalid_body)).expect_err("body utf8"),
            ApiError::InvalidWirePayload
        );
        let truncated =
            format!("POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\ncontent-length: 4\r\n\r\nab");
        assert_eq!(
            read_http_request(&mut Cursor::new(truncated.into_bytes())).expect_err("short body"),
            ApiError::InvalidWirePayload
        );
        let huge = format!(
            "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\ncontent-length: {}\r\n\r\n",
            DEFAULT_ANALYSIS_RUN_BYTE_LIMIT + 1
        );
        assert_eq!(
            read_http_request(&mut Cursor::new(huge.into_bytes())).expect_err("body limit"),
            ApiError::LimitExceeded
        );
    }

    #[test]
    fn serve_one_covers_loopback_success_disconnect_and_timeout() {
        let run = sample_run();
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("address");
        let worker = thread::spawn(move || service.serve_one());
        let mut stream = TcpStream::connect(addr).expect("connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("read timeout");
        stream
            .write_all(valid_request(&run, NARUON_CONSUMER_CODE, &addr.to_string()).as_bytes())
            .expect("request");
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("response");
        assert!(response.starts_with("HTTP/1.1 202 Accepted"));
        assert_eq!(
            worker.join().expect("join").expect("served").status_code,
            202
        );

        let mut idle = AnalysisRunLiveService::bind_loopback().expect("idle bind");
        let idle_addr = idle.local_addr().expect("idle address");
        let idle_worker = thread::spawn(move || idle.serve_one());
        drop(TcpStream::connect(idle_addr).expect("idle connect"));
        assert_eq!(
            idle_worker
                .join()
                .expect("idle join")
                .expect("idle served")
                .status_code,
            400
        );

        let mut timeout = AnalysisRunLiveService::bind_loopback().expect("timeout bind");
        let timeout_addr = timeout.local_addr().expect("timeout address");
        let timeout_worker = thread::spawn(move || timeout.serve_one());
        let stream = TcpStream::connect(timeout_addr).expect("timeout connect");
        let started = std::time::Instant::now();
        let timeout_response = timeout_worker
            .join()
            .expect("timeout join")
            .expect("timeout served");
        drop(stream);
        assert!(started.elapsed() >= NARUON_LIVE_IO_TIMEOUT);
        assert_eq!(timeout_response.status_code, 413);
        assert_eq!(
            envelope(&timeout_response.body).error_code(),
            "limit_exceeded"
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn get_status_keeps_receipts_metric_free_and_gates_scientific_acceptance() {
        let mut run = sample_run();
        run.output_profile = SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE.into();
        let mut service = AnalysisRunLiveService::new();
        let accepted =
            service.handle_http_request(&valid_request(&run, NARUON_CONSUMER_CODE, "127.0.0.1"));
        assert_eq!(accepted.status_code, 202);
        assert!(!accepted.body.contains("scientific_acceptance"));
        assert!(!accepted.body.contains("rmse"));
        let accepted_dto = AnalysisRunAccepted::from_json(&accepted.body).expect("accepted");
        let get_accepted = service.handle_http_request(&status_get(
            &accepted_dto.run_id,
            NARUON_CONSUMER_CODE,
            run.idempotency_key.as_str(),
        ));
        assert_eq!(get_accepted.status_code, 200);
        assert!(get_accepted.body.contains("\"accepted\""));
        assert!(!get_accepted.body.contains("scientific_acceptance"));
        assert!(!get_accepted.body.contains("rmse"));

        let encoded = service.handle_http_request(&http_get(
            &format!("{NARUON_ANALYSIS_RUN_PATH}/tepp-run-%31"),
            &[
                ("Host", "127.0.0.1"),
                ("content-type", "application/json"),
                ("tepp-consumer", NARUON_CONSUMER_CODE),
                ("tepp-contract-version", "1"),
                ("idempotency-key", run.idempotency_key.as_str()),
            ],
        ));
        assert_eq!(encoded.status_code, 200);

        let running = AnalysisRunStatus::running(&accepted_dto).expect("running");
        service
            .record_loopback_status(&accepted_dto.run_id, running, None)
            .expect("running recorded");
        let get_running = service.handle_http_request(&status_get(
            &accepted_dto.run_id,
            NARUON_CONSUMER_CODE,
            run.idempotency_key.as_str(),
        ));
        assert_eq!(get_running.status_code, 200);
        assert!(get_running.body.contains("\"running\""));
        assert!(!get_running.body.contains("scientific_acceptance"));

        let artifact = format!(
            r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}","output_profile":"{SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE}","binding_sha256":"{}","run_id":"{}"}}"#,
            "ab".repeat(32),
            accepted_dto.run_id
        );
        let digest = sha256_hex(artifact.as_bytes());
        let terminal = AnalysisRunTerminalResult::succeeded(
            &run,
            &accepted_dto,
            "artifact-live-1",
            digest,
            SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
            "2026-08-02T03:04:05Z",
            AnalysisResultSummary::new("scientific_acceptance", 4, 8, "validated")
                .expect("summary"),
        )
        .expect("terminal");
        let succeeded =
            AnalysisRunStatus::terminal(&run, &accepted_dto, terminal).expect("succeeded");
        service
            .record_loopback_status(&accepted_dto.run_id, succeeded, Some(artifact.clone()))
            .expect("succeeded recorded");
        let get_succeeded = service.handle_http_request(&status_get(
            &accepted_dto.run_id,
            NARUON_CONSUMER_CODE,
            run.idempotency_key.as_str(),
        ));
        assert_eq!(get_succeeded.status_code, 200);
        assert!(
            get_succeeded
                .body
                .contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA)
        );
        assert!(get_succeeded.body.contains("scientific_acceptance"));

        let failed = AnalysisRunTerminalResult::failed(
            &run,
            &accepted_dto,
            "2026-08-02T03:04:05Z",
            "estimation_failed",
        )
        .expect("failed");
        let failed_status =
            AnalysisRunStatus::terminal(&run, &accepted_dto, failed).expect("failed status");
        assert_eq!(
            service.record_loopback_status(&accepted_dto.run_id, failed_status, Some(artifact),),
            Err(ApiError::InvalidWirePayload)
        );

        assert_eq!(
            service
                .handle_http_request(&status_get(
                    &accepted_dto.run_id,
                    LINEAGEWEAVE_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&status_get(
                    &accepted_dto.run_id,
                    NARUON_CONSUMER_CODE,
                    "wrong-key",
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&status_get(
                    "missing-run",
                    NARUON_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
        let mut with_body = status_get(
            &accepted_dto.run_id,
            NARUON_CONSUMER_CODE,
            run.idempotency_key.as_str(),
        );
        with_body = with_body.replace("content-length: 0", "content-length: 1");
        with_body.push('x');
        assert_eq!(service.handle_http_request(&with_body).status_code, 400);
        assert_eq!(
            service
                .handle_http_request(&http_get(
                    NARUON_ANALYSIS_RUN_PATH,
                    &[
                        ("Host", "127.0.0.1"),
                        ("content-type", "application/json"),
                        ("tepp-consumer", NARUON_CONSUMER_CODE),
                        ("tepp-contract-version", "1"),
                        ("idempotency-key", run.idempotency_key.as_str()),
                    ],
                ))
                .status_code,
            400
        );

        let metric_body = run
            .to_json()
            .expect("json")
            .replacen('{', r#"{"rmse":0.1,"#, 1);
        assert_eq!(
            service
                .handle_http_request(&http_request(
                    &metric_body,
                    &[
                        ("Host", "127.0.0.1"),
                        ("content-type", "application/json"),
                        ("tepp-consumer", NARUON_CONSUMER_CODE),
                        ("tepp-contract-version", "1"),
                        ("idempotency-key", run.idempotency_key.as_str()),
                    ],
                ))
                .status_code,
            400
        );

        assert_eq!(
            service.record_loopback_status("missing", failed_status_placeholder(), None),
            Err(ApiError::InvalidWirePayload)
        );
        service
            .runs_by_id
            .insert("ghost".into(), "missing-replay".into());
        assert_eq!(
            service
                .handle_http_request(&status_get(
                    "ghost",
                    NARUON_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
        assert_eq!(
            service.record_loopback_status("ghost", failed_status_placeholder(), None),
            Err(ApiError::InvalidWirePayload)
        );
        let replay = service
            .runs_by_id
            .get(&accepted_dto.run_id)
            .expect("replay")
            .clone();
        service.runs_by_id.insert("tepp-run-other".into(), replay);
        assert_eq!(
            service.record_loopback_status(
                "tepp-run-other",
                AnalysisRunStatus::accepted(&accepted_dto).expect("status"),
                None,
            ),
            Err(ApiError::InvalidWirePayload)
        );

        let mut collision = AnalysisRunLiveService::new();
        collision
            .runs_by_id
            .insert("tepp-run-1".into(), "preloaded".into());
        collision.next_run_serial = 1;
        assert_eq!(
            collision
                .handle_http_request(&valid_request(&run, NARUON_CONSUMER_CODE, "127.0.0.1"))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&http_get(
                    &format!("{NARUON_ANALYSIS_RUN_PATH}/{}", accepted_dto.run_id),
                    &[
                        ("Host", "127.0.0.1"),
                        ("content-type", "application/json"),
                        ("tepp-consumer", NARUON_CONSUMER_CODE),
                        ("tepp-contract-version", "1"),
                        ("idempotency-key", run.idempotency_key.as_str()),
                        ("authorization", "Bearer secret"),
                    ],
                ))
                .status_code,
            403
        );
    }

    fn failed_status_placeholder() -> AnalysisRunStatus {
        let request = sample_run();
        let accepted =
            AnalysisRunAccepted::new("missing", "accepted", request.idempotency_key.clone())
                .expect("accepted");
        AnalysisRunStatus::accepted(&accepted).expect("status")
    }

    struct ScriptedRead {
        reader: Cursor<Vec<u8>>,
        first_error: Option<std::io::ErrorKind>,
    }

    impl ScriptedRead {
        fn bytes(bytes: &[u8]) -> Self {
            Self {
                reader: Cursor::new(bytes.to_vec()),
                first_error: None,
            }
        }

        fn error(kind: std::io::ErrorKind) -> Self {
            Self {
                reader: Cursor::new(Vec::new()),
                first_error: Some(kind),
            }
        }
    }

    impl Read for ScriptedRead {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            if let Some(kind) = self.first_error.take() {
                return Err(std::io::Error::new(kind, "scripted error"));
            }
            self.reader.read(buffer)
        }
    }
}
