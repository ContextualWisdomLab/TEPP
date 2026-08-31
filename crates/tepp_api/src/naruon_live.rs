//! Loopback-only live HTTP/1.1 listener for naruon modular POSTs (ADR 0011).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use crate::analysis_run_retry_http::analysis_run_retry_path_run_id;
use crate::authorization::{
    AnalyticalPurpose, ExportAuthorizationRequest, authorize_export, require_export_allowed,
};
use crate::lineageweave_http::NARUON_CONSUMER_CODE;
use crate::live_http::{
    header_value, map_io_error, parse_headers, parse_request_line, read_http_request,
    split_request, validate_common_headers,
};
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, NARUON_EXPORT_PATH};
use crate::wire::{from_json, to_json};
use crate::{
    AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunRetryRequest, AnalysisRunStatusState,
    ApiError, ErrorEnvelope, refuse_metrics_on_retry_payload, requests_are_idempotent_matches,
};

#[cfg(test)]
use crate::DEFAULT_ANALYSIS_RUN_BYTE_LIMIT;
#[cfg(test)]
use crate::live_http::{
    declared_content_length, host_implies_table_access, host_is_loopback, split_header_line,
};

/// Maximum live HTTP header-block bytes.
pub use crate::live_http::NARUON_LIVE_HEADER_BYTE_LIMIT;
/// Maximum live HTTP header count.
pub use crate::live_http::NARUON_LIVE_HEADER_COUNT_LIMIT;

/// Read and write deadline installed on every accepted stream.
pub const NARUON_LIVE_IO_TIMEOUT: Duration = Duration::from_secs(1);

/// HTTP/1.1 response produced by the naruon live listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NaruonLiveResponse {
    /// Numeric status code.
    pub status_code: u16,
    /// RFC 9110 reason phrase paired with [`Self::status_code`].
    pub reason_phrase: &'static str,
    /// JSON accepted-run, export-decision, or redacted error envelope.
    pub body: String,
}

/// Loopback live HTTP/1.1 service for naruon analysis-run and export POSTs.
///
/// Production interchange origins remain `https` only. This listener binds
/// loopback TCP so tests and local standalone operation can prove request
/// handling without claiming TLS termination or cross-service table access.
/// This port only accepts versioned naruon POSTs, including
/// `POST /v1/analysis-runs/{run_id}/retry` for metric-free retry of failed or
/// cancelled runs. `LineageWeave` remains refused here and uses
/// `AnalysisRunLiveService`. GET remains refused.
#[derive(Debug)]
pub struct NaruonLiveService {
    listener: Option<TcpListener>,
    bound_addr: Option<SocketAddr>,
    next_run_serial: u64,
    next_request_serial: u64,
    accepted_runs: HashMap<String, NaruonLiveRun>,
    runs_by_id: HashMap<String, String>,
}

/// One naruon-only accepted run and its current lifecycle state.
#[derive(Clone, Debug, Eq, PartialEq)]
struct NaruonLiveRun {
    request: AnalysisRunRequest,
    accepted: AnalysisRunAccepted,
    run_state: AnalysisRunStatusState,
}

impl Default for NaruonLiveService {
    fn default() -> Self {
        Self::new()
    }
}

impl NaruonLiveService {
    /// Construct an in-memory handler with no socket.
    #[must_use]
    pub fn new() -> Self {
        Self {
            listener: None,
            bound_addr: None,
            next_run_serial: 1,
            next_request_serial: 1,
            accepted_runs: HashMap::new(),
            runs_by_id: HashMap::new(),
        }
    }

    /// Bind `127.0.0.1:0`.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when the operating system
    /// refuses the loopback bind.
    pub fn bind_loopback() -> Result<Self, ApiError> {
        Self::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
    }

    /// Bind a caller-supplied address after refusing non-loopback IPs.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback bind
    /// address and [`ApiError::InvalidWirePayload`] when the socket cannot
    /// be opened.
    pub fn bind(addr: SocketAddr) -> Result<Self, ApiError> {
        if !addr.ip().is_loopback() {
            return Err(ApiError::AuthorizationDenied);
        }
        let listener = TcpListener::bind(addr).map_err(|error| map_io_error(&error))?;
        let bound_addr = listener
            .local_addr()
            .map_err(|error| map_io_error(&error))?;
        let mut service = Self::new();
        service.listener = Some(listener);
        service.bound_addr = Some(bound_addr);
        Ok(service)
    }

    /// Return the bound loopback address.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when no socket is bound.
    pub fn local_addr(&self) -> Result<SocketAddr, ApiError> {
        self.bound_addr.ok_or(ApiError::InvalidWirePayload)
    }

    /// Accept one TCP connection and serve one HTTP/1.1 request.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when no socket is bound or
    /// the accept/write path fails for a non-timeout reason. Timeouts map to
    /// [`ApiError::LimitExceeded`].
    pub fn serve_one(&mut self) -> Result<NaruonLiveResponse, ApiError> {
        let listener = self.listener.as_ref().ok_or(ApiError::InvalidWirePayload)?;
        self.serve_accepted(listener.accept().map(|(stream, _)| stream))
    }

    /// Serve one already-accepted stream, or map an accept failure.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`]
    /// when accept or response writing fails. Request-protocol failures become
    /// HTTP error responses and are returned as `Ok`.
    pub fn serve_accepted(
        &mut self,
        accepted: Result<TcpStream, std::io::Error>,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let mut stream = accepted.map_err(|error| map_io_error(&error))?;
        stream
            .set_read_timeout(Some(NARUON_LIVE_IO_TIMEOUT))
            .map_err(|error| map_io_error(&error))?;
        stream
            .set_write_timeout(Some(NARUON_LIVE_IO_TIMEOUT))
            .map_err(|error| map_io_error(&error))?;
        let response = match Self::read_http_request(&mut stream) {
            Ok(request) => self.handle_http_request(&request),
            Err(error) => self.response_from_error(error),
        };
        Self::write_response(&mut stream, &response)?;
        Ok(response)
    }

    /// Parse and handle a complete HTTP/1.1 request already in memory.
    #[must_use]
    pub fn handle_http_request(&mut self, request: &str) -> NaruonLiveResponse {
        match self.dispatch_http_request(request) {
            Ok(response) => response,
            Err(error) => self.response_from_error(error),
        }
    }

    /// Read one HTTP/1.1 request from `reader`, including the declared body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::LimitExceeded`] on timeout or when headers exceed
    /// [`NARUON_LIVE_HEADER_BYTE_LIMIT`]. Other read/framing failures are
    /// [`ApiError::InvalidWirePayload`].
    pub fn read_http_request<R: Read>(reader: &mut R) -> Result<String, ApiError> {
        read_http_request(reader)
    }

    /// Write one HTTP/1.1 response to `writer`.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when the write fails.
    pub fn write_response<W: Write>(
        writer: &mut W,
        response: &NaruonLiveResponse,
    ) -> Result<(), ApiError> {
        writer
            .write_all(&response.to_http_bytes())
            .map_err(|error| map_io_error(&error))?;
        writer.flush().map_err(|error| map_io_error(&error))
    }

    fn dispatch_http_request(&mut self, request: &str) -> Result<NaruonLiveResponse, ApiError> {
        let (header_block, body) = split_request(request)?;
        let mut lines = header_block.split("\r\n");
        let request_line = lines.next().unwrap_or("");
        let (method, path) = parse_request_line(request_line)?;
        if method != "POST" {
            return Err(ApiError::InvalidWirePayload);
        }
        let headers = parse_headers(lines)?;
        refuse_live_headers(&headers, self.bound_addr)?;
        if matches!(
            analysis_run_retry_path_run_id(path),
            Ok(_) | Err(ApiError::LimitExceeded)
        ) {
            return self.retry_analysis_run(path, &headers, body);
        }
        if path != NARUON_ANALYSIS_RUN_PATH && path != NARUON_EXPORT_PATH {
            return Err(ApiError::InvalidWirePayload);
        }
        self.dispatch_path(path, &headers, body)
    }

    fn dispatch_path(
        &mut self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        if path == NARUON_ANALYSIS_RUN_PATH {
            self.accept_analysis_run(headers, body)
        } else {
            Self::authorize_export(headers, body)
        }
    }

    fn accept_analysis_run(
        &mut self,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let request = AnalysisRunRequest::from_json(body)?;
        let idempotency_key = header_value(headers, "idempotency-key")?;
        if idempotency_key != request.idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        let replay_key = tenant_idempotency_key(&request.tenant_workspace_id, idempotency_key);
        if let Some(stored) = self.accepted_runs.get(&replay_key) {
            if requests_are_idempotent_matches(&stored.request, &request) {
                return Ok(NaruonLiveResponse::json(
                    202,
                    "Accepted",
                    stored.accepted.to_json()?,
                ));
            }
            return Err(ApiError::InvalidWirePayload);
        }
        let run_id = format!("naruon-run-{}", self.next_run_serial);
        self.next_run_serial += 1;
        let accepted =
            AnalysisRunAccepted::new(run_id, "accepted", request.idempotency_key.clone())?;
        let body = accepted.to_json()?;
        self.runs_by_id
            .insert(accepted.run_id.clone(), replay_key.clone());
        self.accepted_runs.insert(
            replay_key,
            NaruonLiveRun {
                request,
                accepted,
                run_state: AnalysisRunStatusState::Accepted,
            },
        );
        Ok(NaruonLiveResponse::json(202, "Accepted", body))
    }

    fn retry_analysis_run(
        &mut self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let run_id = analysis_run_retry_path_run_id(path)?;
        refuse_metrics_on_retry_payload(body)?;
        let new_idempotency_key = header_value(headers, "idempotency-key")?;
        if !body.trim().is_empty() {
            let retry = AnalysisRunRetryRequest::from_json(body)?;
            if retry.run_id != run_id || retry.idempotency_key != new_idempotency_key {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        let parent_replay_key = self
            .runs_by_id
            .get(&run_id)
            .cloned()
            .ok_or(ApiError::InvalidWirePayload)?;
        let (mut cloned_request, parent_idempotency_key, parent_state) = {
            let stored = self
                .accepted_runs
                .get(&parent_replay_key)
                .ok_or(ApiError::InvalidWirePayload)?;
            (
                stored.request.clone(),
                stored.accepted.idempotency_key.clone(),
                stored.run_state,
            )
        };
        if new_idempotency_key == parent_idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        match parent_state {
            AnalysisRunStatusState::Failed | AnalysisRunStatusState::Cancelled => {}
            AnalysisRunStatusState::Accepted
            | AnalysisRunStatusState::Running
            | AnalysisRunStatusState::Succeeded => {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        new_idempotency_key.clone_into(&mut cloned_request.idempotency_key);
        cloned_request.validate()?;
        let replay_key =
            tenant_idempotency_key(&cloned_request.tenant_workspace_id, new_idempotency_key);
        if let Some(stored) = self.accepted_runs.get(&replay_key) {
            if requests_are_idempotent_matches(&stored.request, &cloned_request) {
                let response_body = stored.accepted.to_json()?;
                refuse_metrics_on_retry_payload(&response_body)?;
                return Ok(NaruonLiveResponse::json(202, "Accepted", response_body));
            }
            return Err(ApiError::InvalidWirePayload);
        }
        let child_run_id = format!("naruon-run-{}", self.next_run_serial);
        self.next_run_serial += 1;
        let accepted = AnalysisRunAccepted::new(
            child_run_id.clone(),
            "accepted",
            cloned_request.idempotency_key.clone(),
        )?;
        let response_body = accepted.to_json()?;
        refuse_metrics_on_retry_payload(&response_body)?;
        self.runs_by_id.insert(child_run_id, replay_key.clone());
        self.accepted_runs.insert(
            replay_key,
            NaruonLiveRun {
                request: cloned_request,
                accepted,
                run_state: AnalysisRunStatusState::Accepted,
            },
        );
        Ok(NaruonLiveResponse::json(202, "Accepted", response_body))
    }

    /// Test-only seam that records a non-accepted loopback state.
    ///
    /// Used to prove retry of failed and cancelled Naruon runs without
    /// duplicating the live cancel HTTP slice.
    #[cfg(test)]
    fn force_naruon_run_state(
        &mut self,
        run_id: &str,
        run_state: AnalysisRunStatusState,
    ) -> Result<(), ApiError> {
        let replay_key = self
            .runs_by_id
            .get(run_id)
            .cloned()
            .ok_or(ApiError::InvalidWirePayload)?;
        let stored = self
            .accepted_runs
            .get_mut(&replay_key)
            .ok_or(ApiError::InvalidWirePayload)?;
        stored.run_state = run_state;
        Ok(())
    }

    fn authorize_export(
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let request: ExportAuthorizationRequest = from_json(body)?;
        let idempotency_key = header_value(headers, "idempotency-key")?;
        if idempotency_key == request.principal_id {
            return Err(ApiError::InvalidWirePayload);
        }
        if request.purpose != AnalyticalPurpose::ModularServiceConsumer {
            return Err(ApiError::AuthorizationDenied);
        }
        let decision = authorize_export(&request)?;
        require_export_allowed(&decision)?;
        Ok(NaruonLiveResponse::json(200, "OK", to_json(&decision)?))
    }

    fn response_from_error(&mut self, error: ApiError) -> NaruonLiveResponse {
        let request_id = format!("naruon-live-{}", self.next_request_serial);
        self.next_request_serial += 1;
        let (status_code, reason_phrase) = status_for(error);
        NaruonLiveResponse::json(status_code, reason_phrase, envelope_json(error, request_id))
    }
}

impl NaruonLiveResponse {
    fn json(status_code: u16, reason_phrase: &'static str, body: String) -> Self {
        Self {
            status_code,
            reason_phrase,
            body,
        }
    }

    /// Render the response as an HTTP/1.1 message.
    #[must_use]
    pub fn to_http_bytes(&self) -> Vec<u8> {
        format!(
            "HTTP/1.1 {} {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
            self.status_code,
            self.reason_phrase,
            self.body.len(),
            self.body
        )
        .into_bytes()
    }
}

fn tenant_idempotency_key(tenant_workspace_id: &str, idempotency_key: &str) -> String {
    format!("{tenant_workspace_id}\u{1f}{idempotency_key}")
}

fn envelope_json(error: ApiError, request_id: String) -> String {
    ErrorEnvelope::from_api_error(error, request_id)
        .and_then(|envelope| envelope.to_json())
        .unwrap_or_else(|_| fallback_envelope_json())
}

fn fallback_envelope_json() -> String {
    "{\"error_code\":\"invalid_wire_payload\",\"message\":\"invalid API wire payload\",\"request_id\":\"naruon-live-fallback\",\"retryable\":false}".to_owned()
}

fn status_for(error: ApiError) -> (u16, &'static str) {
    match error {
        ApiError::InvalidWirePayload => (400, "Bad Request"),
        ApiError::AuthorizationDenied => (403, "Forbidden"),
        ApiError::LimitExceeded => (413, "Payload Too Large"),
        ApiError::UnsupportedContractVersion => (422, "Unprocessable Entity"),
    }
}

fn refuse_live_headers(
    headers: &HashMap<String, String>,
    bound_addr: Option<SocketAddr>,
) -> Result<(), ApiError> {
    validate_common_headers(headers, bound_addr)?;
    if header_value(headers, "tepp-consumer")? != NARUON_CONSUMER_CODE {
        return Err(ApiError::InvalidWirePayload);
    }
    if header_value(headers, "tepp-contract-version")? != "1" {
        return Err(ApiError::InvalidWirePayload);
    }
    let _idempotency_key = header_value(headers, "idempotency-key")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        NaruonLiveService, declared_content_length, envelope_json, fallback_envelope_json,
        host_implies_table_access, host_is_loopback, map_io_error, parse_request_line,
        split_header_line, split_request, status_for, tenant_idempotency_key,
    };
    use crate::ApiError;
    use std::io::ErrorKind;
    use std::net::SocketAddr;

    #[test]
    fn helpers_cover_status_io_host_and_request_line_edges() {
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
            map_io_error(&std::io::Error::new(ErrorKind::TimedOut, "t")),
            ApiError::LimitExceeded
        );
        assert_eq!(
            map_io_error(&std::io::Error::new(ErrorKind::WouldBlock, "w")),
            ApiError::LimitExceeded
        );
        assert_eq!(
            map_io_error(&std::io::Error::other("x")),
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
        assert!(host_is_loopback("127.0.0.1", None));
        assert!(host_is_loopback("localhost", None));
        assert!(host_is_loopback("localhost:8080", None));
        assert!(!host_is_loopback("localhost:invalid", None));
        assert!(!host_is_loopback("localhost:", None));
        assert!(host_is_loopback("[::1]:9", None));
        assert!(host_is_loopback("::1", None));
        assert!(!host_is_loopback("8.8.8.8", None));
        assert!(!host_is_loopback("attacker.example.com", None));
        let bound: SocketAddr = "127.0.0.1:43789".parse().expect("bound");
        assert!(host_is_loopback("127.0.0.1:43789", Some(bound)));
        assert!(host_is_loopback("127.0.0.1", Some(bound)));
        assert!(!host_is_loopback("8.8.8.8", Some(bound)));
        assert_eq!(
            tenant_idempotency_key("tenant-a", "idem-1"),
            "tenant-a\u{1f}idem-1"
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn helpers_cover_request_line_headers_and_accept_failure() {
        assert_eq!(
            parse_request_line("POST /v1/analysis-runs HTTP/1.1 extra"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST https://tepp.example/v1/analysis-runs HTTP/1.1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST /v1/analysis-runs#x HTTP/1.1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST /only"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST /x HTTP/1.0"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST /x?query HTTP/1.1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(parse_request_line("POST /x HTTP/1.1"), Ok(("POST", "/x")));
        assert_eq!(
            split_header_line("NoColon"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line(": empty-name"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line("Bad Name: v"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line("Host: 127.0.0.1").expect("hdr"),
            ("Host", "127.0.0.1")
        );
        assert_eq!(
            declared_content_length(
                "POST /x HTTP/1.1\r\ncontent-length: 1\r\ncontent-length: 1\r\n\r\n"
            ),
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
                "POST /x HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: 0\r\n\r\n"
            ),
            Ok(0)
        );
        assert_eq!(
            declared_content_length("POST /x HTTP/1.1\r\ncontent-length: \r\n\r\n"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST /proxy://target HTTP/1.1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            split_request(&"x".repeat(super::NARUON_LIVE_HEADER_BYTE_LIMIT)),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(split_request("short"), Err(ApiError::InvalidWirePayload));
        assert_eq!(
            split_request("POST /x HTTP/1.1\r\ncontent-length: 0\r\n\r\n"),
            Ok(("POST /x HTTP/1.1\r\ncontent-length: 0", ""))
        );
        assert_eq!(
            split_request(&format!(
                "{}\r\n\r\n",
                "x".repeat(super::NARUON_LIVE_HEADER_BYTE_LIMIT + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            split_request("POST /x HTTP/1.1\r\ncontent-length: 1\r\n\r\n"),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized_body = "x".repeat(super::DEFAULT_ANALYSIS_RUN_BYTE_LIMIT + 1);
        assert_eq!(
            split_request(&format!(
                "POST /x HTTP/1.1\r\ncontent-length: {}\r\n\r\n{oversized_body}",
                oversized_body.len()
            )),
            Err(ApiError::LimitExceeded)
        );
        assert!(!fallback_envelope_json().is_empty());
        assert!(
            envelope_json(ApiError::InvalidWirePayload, String::new())
                .contains("naruon-live-fallback")
        );
        assert!(envelope_json(ApiError::LimitExceeded, "req-1".into()).contains("limit_exceeded"));
        assert_eq!(
            declared_content_length(
                "POST /x HTTP/1.1\r\ncontent-length: 999999999999999999999\r\n\r\n"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            NaruonLiveService::new()
                .serve_accepted(Err(std::io::Error::other("accept")))
                .expect_err("accept"),
            ApiError::InvalidWirePayload
        );
    }

    fn sample_run_json() -> String {
        r#"{"contract_version":1,"idempotency_key":"naruon-retry-parent","tenant_workspace_id":"naruon-retry-tenant","snapshot_id":"naruon-retry-snapshot","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"tepp-analysis-run-v1","output_profile":"calibrated_event_measurement"}"#.to_owned()
    }

    fn create_http(body: &str) -> String {
        format!(
            "POST /v1/analysis-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: naruon-retry-parent\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        )
    }

    fn retry_http(run_id: &str, body: &str, consumer: &str, idempotency_key: &str) -> String {
        format!(
            "POST /v1/analysis-runs/{run_id}/retry HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        )
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn handler_covers_metric_free_retry_of_failed_and_cancelled() {
        use crate::{AnalysisRunAccepted, AnalysisRunRetryRequest, AnalysisRunStatusState};

        let body = sample_run_json();
        let mut service = NaruonLiveService::new();
        let accepted = service.handle_http_request(&create_http(&body));
        assert_eq!(accepted.status_code, 202);
        let parent = AnalysisRunAccepted::from_json(&accepted.body).expect("parent");
        assert_eq!(parent.run_id, "naruon-run-1");
        service
            .force_naruon_run_state(&parent.run_id, AnalysisRunStatusState::Failed)
            .expect("force failed");

        let retry_key = "naruon-retry-child";
        let retry_body = AnalysisRunRetryRequest::new(&parent.run_id, retry_key)
            .expect("retry dto")
            .to_json()
            .expect("retry json");
        let retried = service.handle_http_request(&retry_http(
            &parent.run_id,
            &retry_body,
            "naruon",
            retry_key,
        ));
        assert_eq!(retried.status_code, 202);
        let child = AnalysisRunAccepted::from_json(&retried.body).expect("child");
        assert_eq!(child.run_id, "naruon-run-2");
        assert_eq!(child.idempotency_key, retry_key);
        assert_eq!(child.run_state, "accepted");
        assert!(!retried.body.contains("rmse"));
        assert!(!retried.body.contains("scientific_acceptance"));

        let replay = service.handle_http_request(&retry_http(
            &parent.run_id,
            &retry_body,
            "naruon",
            retry_key,
        ));
        assert_eq!(replay.status_code, 202);
        assert_eq!(replay.body, retried.body);

        let empty_body =
            service.handle_http_request(&retry_http(&parent.run_id, "", "naruon", retry_key));
        assert_eq!(empty_body.status_code, 202);
        assert_eq!(empty_body.body, retried.body);

        let mut cancelled_service = NaruonLiveService::new();
        let cancelled_parent = cancelled_service.handle_http_request(&create_http(&body));
        let cancelled_id = AnalysisRunAccepted::from_json(&cancelled_parent.body)
            .expect("cancelled parent")
            .run_id;
        cancelled_service
            .force_naruon_run_state(&cancelled_id, AnalysisRunStatusState::Cancelled)
            .expect("force cancelled");
        let cancelled_retry = cancelled_service.handle_http_request(&retry_http(
            &cancelled_id,
            "",
            "naruon",
            "naruon-retry-cancelled-child",
        ));
        assert_eq!(cancelled_retry.status_code, 202);
        assert!(!cancelled_retry.body.contains("rmse"));

        let mut accepted_only = NaruonLiveService::new();
        let still_accepted = accepted_only.handle_http_request(&create_http(&body));
        let accepted_id = AnalysisRunAccepted::from_json(&still_accepted.body)
            .expect("accepted")
            .run_id;
        assert_eq!(
            accepted_only
                .handle_http_request(&retry_http(
                    &accepted_id,
                    "",
                    "naruon",
                    "naruon-retry-too-early"
                ))
                .status_code,
            400
        );

        for state in [
            AnalysisRunStatusState::Running,
            AnalysisRunStatusState::Succeeded,
        ] {
            let mut blocked = NaruonLiveService::new();
            let created = blocked.handle_http_request(&create_http(&body));
            let blocked_id = AnalysisRunAccepted::from_json(&created.body)
                .expect("blocked")
                .run_id;
            blocked
                .force_naruon_run_state(&blocked_id, state)
                .expect("force");
            assert_eq!(
                blocked
                    .handle_http_request(&retry_http(
                        &blocked_id,
                        "",
                        "naruon",
                        "naruon-retry-blocked"
                    ))
                    .status_code,
                400
            );
        }

        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    &parent.run_id,
                    "",
                    "naruon",
                    "naruon-retry-parent"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&retry_http("missing", "", "naruon", retry_key))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    &parent.run_id,
                    "",
                    "lineageweave",
                    "naruon-retry-foreign"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET /v1/analysis-runs/{}/retry HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {retry_key}\r\ncontent-length: 0\r\n\r\n",
                    parent.run_id
                ))
                .status_code,
            400
        );
        let mismatched = AnalysisRunRetryRequest::new("other-run", retry_key)
            .expect("mismatch")
            .to_json()
            .expect("json");
        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    &parent.run_id,
                    &mismatched,
                    "naruon",
                    retry_key
                ))
                .status_code,
            400
        );
        let metric_body = format!(
            r#"{{"contract_version":1,"run_id":"{}","idempotency_key":"{retry_key}","rmse":0.1}}"#,
            parent.run_id
        );
        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    &parent.run_id,
                    &metric_body,
                    "naruon",
                    retry_key
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .force_naruon_run_state("ghost", AnalysisRunStatusState::Failed)
                .expect_err("unknown"),
            ApiError::InvalidWirePayload
        );
    }
}
