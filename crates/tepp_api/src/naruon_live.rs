//! Loopback-only live HTTP/1.1 listener for naruon modular POSTs (ADR 0011).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use crate::analysis_run_lifecycle_http::AnalysisRunLifecycleTransition;
use crate::analysis_run_status_http::{AnalysisRunLiveRoute, parse_analysis_run_live_route};
use crate::authorization::{
    AnalyticalPurpose, ExportAuthorizationRequest, authorize_export, require_export_allowed,
};
use crate::lineageweave_http::NARUON_CONSUMER_CODE;
use crate::live_http::{
    header_value, map_io_error, parse_headers, parse_request_line, read_http_request,
    split_request, validate_common_headers,
};
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, NARUON_EXPORT_PATH};
use crate::scientific_acceptance_http::{refuse_metrics_on_receipt, status_http_json};
use crate::wire::{from_json, to_json};
use crate::{
    AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunStatus, AnalysisRunStatusState, ApiError,
    ErrorEnvelope, requests_are_idempotent_matches, require_status_binding,
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
/// `POST /v1/analysis-runs/{run_id}/running` and
/// `POST /v1/analysis-runs/{run_id}/terminal` for metric-free lifecycle.
#[derive(Debug)]
pub struct NaruonLiveService {
    listener: Option<TcpListener>,
    bound_addr: Option<SocketAddr>,
    next_run_serial: u64,
    next_request_serial: u64,
    accepted_runs: HashMap<String, NaruonLiveRun>,
    runs_by_id: HashMap<String, String>,
}

/// One naruon-only accepted run and its current lifecycle status.
#[derive(Clone, Debug, Eq, PartialEq)]
struct NaruonLiveRun {
    request: AnalysisRunRequest,
    accepted: AnalysisRunAccepted,
    status: AnalysisRunStatus,
    scientific_acceptance_json: Option<String>,
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
        match parse_analysis_run_live_route(path) {
            Ok(AnalysisRunLiveRoute::Running { run_id }) => {
                return self.post_running_status(&run_id, &headers, body);
            }
            Ok(AnalysisRunLiveRoute::Terminal { run_id }) => {
                return self.post_terminal_status(&run_id, &headers, body);
            }
            Ok(AnalysisRunLiveRoute::Status { .. }) => {
                return Err(ApiError::InvalidWirePayload);
            }
            Err(ApiError::LimitExceeded) => return Err(ApiError::LimitExceeded),
            Err(_) => {}
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
        let status = AnalysisRunStatus::accepted(&accepted)?;
        self.runs_by_id
            .insert(accepted.run_id.clone(), replay_key.clone());
        self.accepted_runs.insert(
            replay_key,
            NaruonLiveRun {
                request,
                accepted,
                status,
                scientific_acceptance_json: None,
            },
        );
        Ok(NaruonLiveResponse::json(202, "Accepted", body))
    }

    fn post_running_status(
        &mut self,
        run_id: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        refuse_metrics_on_receipt(body)?;
        let transition = AnalysisRunLifecycleTransition::from_json(body)?;
        if transition.run_state != AnalysisRunStatusState::Running {
            return Err(ApiError::InvalidWirePayload);
        }
        self.commit_lifecycle_transition(run_id, headers, &transition)
    }

    fn post_terminal_status(
        &mut self,
        run_id: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let transition = AnalysisRunLifecycleTransition::from_json(body)?;
        if !matches!(
            transition.run_state,
            AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed
        ) {
            return Err(ApiError::InvalidWirePayload);
        }
        self.commit_lifecycle_transition(run_id, headers, &transition)
    }

    fn commit_lifecycle_transition(
        &mut self,
        path_run_id: &str,
        headers: &HashMap<String, String>,
        transition: &AnalysisRunLifecycleTransition,
    ) -> Result<NaruonLiveResponse, ApiError> {
        if transition.run_id != path_run_id {
            return Err(ApiError::InvalidWirePayload);
        }
        let idempotency_key = header_value(headers, "idempotency-key")?;
        if idempotency_key != transition.idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        let replay_key = self
            .runs_by_id
            .get(path_run_id)
            .cloned()
            .ok_or(ApiError::InvalidWirePayload)?;
        let stored = self
            .accepted_runs
            .get(&replay_key)
            .ok_or(ApiError::InvalidWirePayload)?;
        if stored.accepted.idempotency_key != idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        let status = match transition.run_state {
            AnalysisRunStatusState::Running => AnalysisRunStatus::running(&stored.accepted)?,
            AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed => {
                let result = transition
                    .terminal_result
                    .clone()
                    .ok_or(ApiError::InvalidWirePayload)?;
                AnalysisRunStatus::terminal(&stored.request, &stored.accepted, result)?
            }
            AnalysisRunStatusState::Accepted => return Err(ApiError::InvalidWirePayload),
        };
        if stored.status == status
            && stored.scientific_acceptance_json == transition.scientific_acceptance_json
        {
            let response_body = status_http_json(
                &stored.status,
                &stored.request,
                stored.scientific_acceptance_json.as_deref(),
            )?;
            return Ok(NaruonLiveResponse::json(200, "OK", response_body));
        }
        match stored.status.run_state {
            AnalysisRunStatusState::Accepted => {}
            AnalysisRunStatusState::Running
                if matches!(
                    transition.run_state,
                    AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed
                ) => {}
            AnalysisRunStatusState::Running
            | AnalysisRunStatusState::Succeeded
            | AnalysisRunStatusState::Failed => {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        let stored = self
            .accepted_runs
            .get_mut(&replay_key)
            .ok_or(ApiError::InvalidWirePayload)?;
        require_status_binding(&stored.request, &stored.accepted, &status)?;
        let _ = status_http_json(
            &status,
            &stored.request,
            transition.scientific_acceptance_json.as_deref(),
        )?;
        stored.status = status;
        stored
            .scientific_acceptance_json
            .clone_from(&transition.scientific_acceptance_json);
        let response_body = status_http_json(
            &stored.status,
            &stored.request,
            stored.scientific_acceptance_json.as_deref(),
        )?;
        Ok(NaruonLiveResponse::json(200, "OK", response_body))
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

    #[test]
    #[allow(clippy::too_many_lines)]
    fn naruon_compatibility_listener_records_running_and_terminal_lifecycle() {
        use crate::{
            ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunLifecycleTransition,
            AnalysisRunRequest, AnalysisRunTerminalResult,
        };
        let run = AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "naruon-lifecycle-idem".into(),
            tenant_workspace_id: "naruon-lifecycle-tenant".into(),
            snapshot_id: "naruon-lifecycle-snapshot".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "topic-measurement-v1".into(),
            output_profile: "naruon-consumer-validation-report".into(),
        };
        let body = run.to_json().expect("json");
        let create = format!(
            "POST /v1/analysis-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
            run.idempotency_key,
            body.len()
        );
        let mut service = NaruonLiveService::new();
        let accepted = service.handle_http_request(&create);
        assert_eq!(accepted.status_code, 202);
        let run_id = AnalysisRunAccepted::from_json(&accepted.body)
            .expect("accepted")
            .run_id;
        let first_run_id = run_id.clone();
        let running = AnalysisRunLifecycleTransition::running(&run_id, run.idempotency_key.clone())
            .expect("running")
            .to_json()
            .expect("running json");
        let running_post = format!(
            "POST /v1/analysis-runs/{run_id}/running HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{running}",
            run.idempotency_key,
            running.len()
        );
        let recorded = service.handle_http_request(&running_post);
        assert_eq!(recorded.status_code, 200);
        assert!(recorded.body.contains("\"run_state\":\"running\""));
        assert!(!recorded.body.contains("rmse"));
        assert!(!recorded.body.contains("scientific_acceptance"));
        let replay = service.handle_http_request(&running_post);
        assert_eq!(replay.body, recorded.body);

        let accepted_dto =
            AnalysisRunAccepted::new(run_id.clone(), "accepted", run.idempotency_key.clone())
                .expect("accepted dto");
        let failed = AnalysisRunTerminalResult::failed(
            &run,
            &accepted_dto,
            "2026-08-02T03:04:05Z",
            "estimation_failed",
        )
        .expect("failed");
        let terminal = AnalysisRunLifecycleTransition::terminal(
            run_id.clone(),
            run.idempotency_key.clone(),
            failed,
            None,
        )
        .expect("terminal")
        .to_json()
        .expect("terminal json");
        let terminal_post = format!(
            "POST /v1/analysis-runs/{run_id}/terminal HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{terminal}",
            run.idempotency_key,
            terminal.len()
        );
        let finished = service.handle_http_request(&terminal_post);
        assert_eq!(finished.status_code, 200);
        assert!(finished.body.contains("\"run_state\":\"failed\""));
        assert!(!finished.body.contains("rmse"));
        let replay_terminal = service.handle_http_request(&terminal_post);
        assert_eq!(replay_terminal.body, finished.body);
        assert_eq!(service.handle_http_request(&running_post).status_code, 400);

        let lineageweave = format!(
            "POST /v1/analysis-runs/{first_run_id}/running HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{running}",
            run.idempotency_key,
            running.len()
        );
        assert_eq!(service.handle_http_request(&lineageweave).status_code, 400);
        let get = format!(
            "GET /v1/analysis-runs/{first_run_id}/running HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: 0\r\n\r\n",
            run.idempotency_key
        );
        assert_eq!(service.handle_http_request(&get).status_code, 400);
        let metrics = format!(
            "POST /v1/analysis-runs/{first_run_id}/running HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: 12\r\n\r\n{{\"rmse\":0.1}}",
            run.idempotency_key
        );
        assert_eq!(service.handle_http_request(&metrics).status_code, 400);
        assert_eq!(
            service.handle_http_request(
                "POST /v1/analysis-runs/missing/running HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: naruon-lifecycle-idem\r\ncontent-length: 0\r\n\r\n"
            ).status_code,
            400
        );
        let oversized = format!(
            "POST /v1/analysis-runs/{}/running HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 0\r\n\r\n",
            "a".repeat(129)
        );
        assert_eq!(service.handle_http_request(&oversized).status_code, 413);
        let mut dangling = NaruonLiveService::new();
        dangling
            .runs_by_id
            .insert("ghost".into(), "missing-replay".into());
        let ghost_running =
            AnalysisRunLifecycleTransition::running("ghost", "naruon-lifecycle-idem")
                .expect("ghost")
                .to_json()
                .expect("ghost json");
        let ghost = format!(
            "POST /v1/analysis-runs/ghost/running HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: naruon-lifecycle-idem\r\ncontent-length: {}\r\n\r\n{ghost_running}",
            ghost_running.len()
        );
        assert_eq!(dangling.handle_http_request(&ghost).status_code, 400);
    }
}
