//! Consumer-neutral live analysis-run ingress for modular CWL services.
//!
//! This module keeps the Naruon compatibility listener intact while providing
//! shared `/v1/analysis-runs` and cutoff-safe `/v1/temporal-context` boundaries
//! needed by Naruon and `LineageWeave`. Analysis-run responses remain transport
//! acknowledgements; completed psychometric results remain outside this crate.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};

use crate::lineageweave_http::consumer_is_supported;
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, header_is_credential};
use crate::naruon_live::host_is_loopback;
use crate::{
    AnalysisRunAccepted, AnalysisRunRequest, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
    ErrorEnvelope, NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT,
    NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse, TEMPORAL_CONTEXT_PATH, TemporalContextRequest,
    build_temporal_context, requests_are_idempotent_matches,
};

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
    accepted_runs: HashMap<String, (AnalysisRunRequest, AnalysisRunAccepted)>,
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
        let response = match read_http_request(&mut stream) {
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
        let (header_block, body) = split_request(request)?;
        let mut lines = header_block.split("\r\n");
        let path = require_request_line(lines.next().unwrap_or(""))?;
        let headers = parse_headers(&mut lines)?;
        let consumer = require_headers(&headers, self.bound_addr)?;
        if path == TEMPORAL_CONTEXT_PATH {
            let context_request = TemporalContextRequest::from_json(body)?;
            let response = build_temporal_context(&context_request)?;
            return Ok(json_response(200, "OK", response.to_json()?));
        }
        self.accept_analysis_run(consumer, &headers, body)
    }

    fn accept_analysis_run(
        &mut self,
        consumer: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
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
        if let Some((stored_request, stored_accepted)) = self.accepted_runs.get(&replay_key) {
            if requests_are_idempotent_matches(stored_request, &request) {
                return Ok(json_response(202, "Accepted", stored_accepted.to_json()?));
            }
            return Err(ApiError::InvalidWirePayload);
        }
        let run_id = format!("tepp-run-{}", self.next_run_serial);
        self.next_run_serial += 1;
        let accepted =
            AnalysisRunAccepted::new(run_id, "accepted", request.idempotency_key.clone())?;
        let response_body = accepted.to_json()?;
        self.accepted_runs.insert(replay_key, (request, accepted));
        Ok(json_response(202, "Accepted", response_body))
    }

    fn response_from_error(&mut self, error: ApiError) -> NaruonLiveResponse {
        let request_id = format!("analysis-run-live-{}", self.next_request_serial);
        self.next_request_serial += 1;
        let (status_code, reason_phrase) = status_for(error);
        json_response(
            status_code,
            reason_phrase,
            error_envelope_json(error, request_id),
        )
    }
}

fn read_http_request(reader: &mut dyn Read) -> Result<String, ApiError> {
    let mut header_bytes = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        if header_bytes.len() >= NARUON_LIVE_HEADER_BYTE_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        let read = reader
            .read(&mut byte)
            .map_err(|error| map_io_error(&error))?;
        if read == 0 {
            return Err(ApiError::InvalidWirePayload);
        }
        header_bytes.push(byte[0]);
        if header_bytes.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let header_text =
        std::str::from_utf8(&header_bytes).map_err(|_| ApiError::InvalidWirePayload)?;
    let content_length = declared_content_length(header_text)?;
    if content_length > DEFAULT_ANALYSIS_RUN_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
    let mut body = vec![0_u8; content_length];
    reader
        .read_exact(&mut body)
        .map_err(|error| map_io_error(&error))?;
    let body_text = std::str::from_utf8(&body).map_err(|_| ApiError::InvalidWirePayload)?;
    Ok(format!("{header_text}{body_text}"))
}

fn split_request(request: &str) -> Result<(&str, &str), ApiError> {
    let Some(index) = request.find("\r\n\r\n") else {
        if request.len() >= NARUON_LIVE_HEADER_BYTE_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        return Err(ApiError::InvalidWirePayload);
    };
    if index > NARUON_LIVE_HEADER_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
    let header_block = &request[..index];
    let body = &request[index + 4..];
    let declared = declared_content_length(&format!("{header_block}\r\n\r\n"))?;
    if declared != body.len() {
        return Err(ApiError::InvalidWirePayload);
    }
    if declared > DEFAULT_ANALYSIS_RUN_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
    Ok((header_block, body))
}

fn declared_content_length(header_text: &str) -> Result<usize, ApiError> {
    let header_block = header_text
        .strip_suffix("\r\n\r\n")
        .ok_or(ApiError::InvalidWirePayload)?;
    let mut found = None;
    for line in header_block.split("\r\n").skip(1) {
        let (name, value) = split_header_line(line)?;
        if name.eq_ignore_ascii_case("content-length") {
            if found.is_some()
                || value.is_empty()
                || !value.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(ApiError::InvalidWirePayload);
            }
            found = Some(value.parse().map_err(|_| ApiError::InvalidWirePayload)?);
        }
    }
    found.ok_or(ApiError::InvalidWirePayload)
}

fn require_request_line(line: &str) -> Result<&str, ApiError> {
    let mut parts = line.split(' ');
    let method = parts.next();
    let path = parts.next();
    if method != Some("POST")
        || !matches!(path, Some(NARUON_ANALYSIS_RUN_PATH | TEMPORAL_CONTEXT_PATH))
        || parts.next() != Some("HTTP/1.1")
        || parts.next().is_some()
    {
        return Err(ApiError::InvalidWirePayload);
    }
    path.ok_or(ApiError::InvalidWirePayload)
}

fn parse_headers(
    lines: &mut dyn Iterator<Item = &str>,
) -> Result<HashMap<String, String>, ApiError> {
    let mut headers = HashMap::new();
    for (index, line) in lines.enumerate() {
        if index >= NARUON_LIVE_HEADER_COUNT_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        let (name, value) = split_header_line(line)?;
        let key = name.to_ascii_lowercase();
        if headers.insert(key, value.to_owned()).is_some() {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    Ok(headers)
}

fn split_header_line(line: &str) -> Result<(&str, &str), ApiError> {
    let (name, value) = line.split_once(':').ok_or(ApiError::InvalidWirePayload)?;
    if name.is_empty() || name.chars().any(|ch| ch.is_whitespace() || ch.is_control()) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok((name, value.trim()))
}

fn require_headers(
    headers: &HashMap<String, String>,
    bound_addr: Option<SocketAddr>,
) -> Result<&str, ApiError> {
    for name in headers.keys() {
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
    }
    if headers.contains_key("transfer-encoding") {
        return Err(ApiError::InvalidWirePayload);
    }
    let host = header_value(headers, "host")?;
    if host_implies_table_access(host) {
        return Err(ApiError::InvalidWirePayload);
    }
    if !host_is_loopback(host, bound_addr) {
        return Err(ApiError::AuthorizationDenied);
    }
    if header_value(headers, "content-type")? != "application/json"
        || header_value(headers, "tepp-contract-version")? != "1"
    {
        return Err(ApiError::InvalidWirePayload);
    }
    let consumer = header_value(headers, "tepp-consumer")?;
    if !consumer_is_supported(consumer) {
        return Err(ApiError::InvalidWirePayload);
    }
    let _idempotency_key = header_value(headers, "idempotency-key")?;
    Ok(consumer)
}

fn header_value<'a>(headers: &'a HashMap<String, String>, name: &str) -> Result<&'a str, ApiError> {
    let value = headers.get(name).ok_or(ApiError::InvalidWirePayload)?;
    if value.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(value.as_str())
}

fn host_implies_table_access(host: &str) -> bool {
    let lowered = host.to_ascii_lowercase();
    lowered.contains("postgres")
        || lowered.contains("jdbc")
        || lowered.contains("/sql")
        || lowered.contains("/tables/")
        || lowered.contains('\'')
        || lowered.contains(';')
        || lowered.contains('\\')
        || lowered.contains(' ')
        || lowered.chars().any(char::is_control)
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

fn map_io_error(error: &std::io::Error) -> ApiError {
    match error.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => ApiError::LimitExceeded,
        _ => ApiError::InvalidWirePayload,
    }
}

fn error_envelope_json(error: ApiError, request_id: String) -> String {
    let result =
        ErrorEnvelope::from_api_error(error, request_id).and_then(|envelope| envelope.to_json());
    error_envelope_or_fallback(result)
}

fn error_envelope_or_fallback(result: Result<String, ApiError>) -> String {
    result.unwrap_or_else(|_| {
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
    use std::io::{Cursor, Read, Write};
    use std::net::{Shutdown, TcpStream};
    use std::thread;

    use super::{
        AnalysisRunLiveService, consumer_tenant_idempotency_key, declared_content_length,
        error_envelope_or_fallback, header_value, host_implies_table_access, map_io_error,
        parse_headers, read_http_request, require_headers, require_request_line, split_header_line,
        split_request, status_for,
    };
    use crate::naruon_live::host_is_loopback;
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunRequest, ApiError,
        DEFAULT_ANALYSIS_RUN_BYTE_LIMIT, LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH,
        NARUON_CONSUMER_CODE, NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT,
        TEMPORAL_CONTEXT_CONTRACT_VERSION, TEMPORAL_CONTEXT_PATH, TemporalContextEvent,
        TemporalContextRequest,
    };

    fn sample_request() -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "live-idempotency-key".into(),
            tenant_workspace_id: "live-tenant-workspace".into(),
            snapshot_id: "live-snapshot".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "tepp-analysis-run-v1".into(),
            output_profile: "temporal-measurement".into(),
        }
    }

    fn http_request(host: &str, consumer: &str, run: &AnalysisRunRequest) -> String {
        let body = run.to_json().expect("run json");
        format!(
            "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
            run.idempotency_key,
            body.len()
        )
    }

    fn valid_headers(consumer: &str) -> HashMap<String, String> {
        HashMap::from([
            ("host".into(), "127.0.0.1".into()),
            ("content-type".into(), "application/json".into()),
            ("tepp-contract-version".into(), "1".into()),
            ("tepp-consumer".into(), consumer.into()),
            ("idempotency-key".into(), "idem".into()),
        ])
    }

    struct FailingReader {
        bytes: Vec<u8>,
        position: usize,
        kind: std::io::ErrorKind,
    }

    impl Read for FailingReader {
        fn read(&mut self, target: &mut [u8]) -> std::io::Result<usize> {
            if self.position < self.bytes.len() {
                let count = (self.bytes.len() - self.position).min(target.len());
                target[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
                self.position += count;
                return Ok(count);
            }
            Err(std::io::Error::from(self.kind))
        }
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
    fn default_and_loopback_binding_expose_real_listener_address() {
        let mut service = AnalysisRunLiveService::default();
        assert_eq!(service.local_addr(), Err(ApiError::InvalidWirePayload));
        let bound = AnalysisRunLiveService::bind_loopback().expect("loopback bind");
        let address = bound.local_addr().expect("bound address");
        assert!(address.ip().is_loopback());
        assert!(address.port() > 0);
        assert_eq!(service.serve_one(), Err(ApiError::InvalidWirePayload));
    }

    #[test]
    fn serve_one_reads_and_writes_a_real_http_exchange() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("loopback bind");
        let address = service.local_addr().expect("address");
        let request = http_request(
            &address.to_string(),
            NARUON_CONSUMER_CODE,
            &sample_request(),
        );
        let worker = thread::spawn(move || service.serve_one());
        let mut client = TcpStream::connect(address).expect("connect");
        client.write_all(request.as_bytes()).expect("request");
        client.shutdown(Shutdown::Write).expect("shutdown write");
        let mut response_bytes = Vec::new();
        client.read_to_end(&mut response_bytes).expect("response");
        let response = worker.join().expect("server thread").expect("serve");
        assert_eq!(response.status_code, 202);
        assert!(
            String::from_utf8(response_bytes)
                .expect("HTTP response")
                .starts_with("HTTP/1.1 202 Accepted\r\n")
        );

        let mut service = AnalysisRunLiveService::bind_loopback().expect("loopback bind");
        let address = service.local_addr().expect("address");
        let worker = thread::spawn(move || service.serve_one());
        let mut client = TcpStream::connect(address).expect("connect");
        client.write_all(b"malformed").expect("request");
        client.shutdown(Shutdown::Write).expect("shutdown write");
        let mut response_bytes = Vec::new();
        client.read_to_end(&mut response_bytes).expect("response");
        let response = worker.join().expect("server thread").expect("serve");
        assert_eq!(response.status_code, 400);
        assert!(
            String::from_utf8(response_bytes)
                .expect("HTTP response")
                .starts_with("HTTP/1.1 400 Bad Request\r\n")
        );

        let mut service = AnalysisRunLiveService::bind_loopback().expect("loopback bind");
        let address = service.local_addr().expect("address");
        let worker = thread::spawn(move || service.serve_one());
        let mut client = TcpStream::connect(address).expect("connect");
        client
            .write_all(
                b"POST /v1/analysis-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: empty-body\r\ncontent-length: 0\r\n\r\n",
            )
            .expect("request");
        client.shutdown(Shutdown::Write).expect("shutdown write");
        let mut response_bytes = Vec::new();
        client.read_to_end(&mut response_bytes).expect("response");
        let response = worker.join().expect("server thread").expect("serve");
        assert_eq!(response.status_code, 400);
    }

    #[test]
    fn analysis_run_rejects_header_key_mismatch_and_replay_conflict() {
        let run = sample_request();
        let mut service = AnalysisRunLiveService::new();
        let mismatched = http_request("127.0.0.1", NARUON_CONSUMER_CODE, &run).replace(
            "idempotency-key: live-idempotency-key",
            "idempotency-key: other-key",
        );
        assert_eq!(service.handle_http_request(&mismatched).status_code, 400);

        let accepted =
            service.handle_http_request(&http_request("127.0.0.1", NARUON_CONSUMER_CODE, &run));
        assert_eq!(accepted.status_code, 202);
        assert_eq!(
            service
                .handle_http_request(&http_request("127.0.0.1", NARUON_CONSUMER_CODE, &run))
                .body,
            accepted.body
        );
        let mut changed = run;
        changed.snapshot_id = "different-snapshot".into();
        assert_eq!(
            service
                .handle_http_request(&http_request("127.0.0.1", NARUON_CONSUMER_CODE, &changed,))
                .status_code,
            400
        );
    }

    #[test]
    fn temporal_context_live_path_returns_a_cutoff_safe_response() {
        let request = TemporalContextRequest {
            contract_version: TEMPORAL_CONTEXT_CONTRACT_VERSION,
            consumer_code: LINEAGEWEAVE_CONSUMER_CODE.into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            subject_post_id: None,
            events: vec![TemporalContextEvent {
                event_id: "context-event".into(),
                source_post_id: "context-post".into(),
                event_type_code: "received".into(),
                event_label: "Received".into(),
                event_time: "2026-07-01T00:00:00Z".into(),
                available_time: "2026-07-01T01:00:00Z".into(),
                project_reference: None,
                actor_references: vec!["actor-1".into()],
            }],
        };
        let body = request.to_json().expect("context json");
        let wire = format!(
            "POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: context-idem\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        );
        let response = AnalysisRunLiveService::new().handle_http_request(&wire);
        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("timeline_events"));
    }

    #[test]
    fn request_reader_covers_eof_limits_encoding_and_body_errors() {
        assert_eq!(
            read_http_request(&mut Cursor::new(Vec::<u8>::new())),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            read_http_request(&mut Cursor::new(vec![b'x'; NARUON_LIVE_HEADER_BYTE_LIMIT])),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            read_http_request(&mut Cursor::new(b"\xff\r\n\r\n".to_vec())),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized = format!(
            "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\ncontent-length: {}\r\n\r\n",
            DEFAULT_ANALYSIS_RUN_BYTE_LIMIT + 1
        );
        assert_eq!(
            read_http_request(&mut Cursor::new(oversized.into_bytes())),
            Err(ApiError::LimitExceeded)
        );
        let invalid_body = b"POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: 1\r\n\r\n\xff";
        assert_eq!(
            read_http_request(&mut Cursor::new(invalid_body.to_vec())),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            read_http_request(&mut Cursor::new(
                b"POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: 0\r\n\r\n".to_vec(),
            )),
            Ok("POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: 0\r\n\r\n".into())
        );
        let header = b"POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: 1\r\n\r\n".to_vec();
        for kind in [std::io::ErrorKind::TimedOut, std::io::ErrorKind::Other] {
            assert_eq!(
                read_http_request(&mut FailingReader {
                    bytes: header.clone(),
                    position: 0,
                    kind,
                }),
                Err(if kind == std::io::ErrorKind::TimedOut {
                    ApiError::LimitExceeded
                } else {
                    ApiError::InvalidWirePayload
                })
            );
        }
    }

    #[test]
    fn request_split_and_content_length_validation_fail_closed() {
        assert_eq!(split_request("short"), Err(ApiError::InvalidWirePayload));
        assert_eq!(
            split_request(&"x".repeat(NARUON_LIVE_HEADER_BYTE_LIMIT)),
            Err(ApiError::LimitExceeded)
        );
        let late_delimiter = format!("{}\r\n\r\n", "x".repeat(NARUON_LIVE_HEADER_BYTE_LIMIT + 1));
        assert_eq!(split_request(&late_delimiter), Err(ApiError::LimitExceeded));
        assert_eq!(
            split_request("POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: 1\r\n\r\n"),
            Err(ApiError::InvalidWirePayload)
        );
        let oversized_body = "x".repeat(DEFAULT_ANALYSIS_RUN_BYTE_LIMIT + 1);
        let oversized_request = format!(
            "POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: {}\r\n\r\n{oversized_body}",
            oversized_body.len()
        );
        assert_eq!(
            split_request(&oversized_request),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            declared_content_length("POST /v1/analysis-runs HTTP/1.1"),
            Err(ApiError::InvalidWirePayload)
        );
        for header in [
            "POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: 1\r\ncontent-length: 1\r\n\r\n",
            "POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: \r\n\r\n",
            "POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: nope\r\n\r\n",
            "POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: 999999999999999999999999\r\n\r\n",
        ] {
            assert_eq!(
                declared_content_length(header),
                Err(ApiError::InvalidWirePayload)
            );
        }
    }

    #[test]
    fn header_and_request_line_parsers_reject_malformed_input() {
        assert_eq!(
            require_request_line("GET /v1/analysis-runs HTTP/1.1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            require_request_line("POST /v1/unknown HTTP/1.1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            require_request_line("POST /v1/analysis-runs HTTP/1.0"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            require_request_line("POST /v1/analysis-runs HTTP/1.1 extra"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            require_request_line("POST /v1/analysis-runs HTTP/1.1"),
            Ok(NARUON_ANALYSIS_RUN_PATH)
        );
        assert_eq!(
            require_request_line("POST /v1/temporal-context HTTP/1.1"),
            Ok(TEMPORAL_CONTEXT_PATH)
        );
        assert_eq!(
            split_header_line("missing delimiter"),
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
            split_header_line(": value"),
            Err(ApiError::InvalidWirePayload)
        );
        let too_many: Vec<_> = (0..=NARUON_LIVE_HEADER_COUNT_LIMIT)
            .map(|index| format!("x-header-{index}: value"))
            .collect();
        let mut too_many_lines = too_many.iter().map(String::as_str);
        assert_eq!(
            parse_headers(&mut too_many_lines),
            Err(ApiError::LimitExceeded)
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
        let mut single_header = ["X-Header: one"].into_iter();
        assert_eq!(
            parse_headers(&mut single_header).expect("header"),
            HashMap::from([(String::from("x-header"), String::from("one"))])
        );
    }

    #[test]
    fn required_headers_enforce_loopback_identity_and_transport_contract() {
        let mut headers = valid_headers(NARUON_CONSUMER_CODE);
        assert_eq!(
            require_headers(&headers, None).expect("valid headers"),
            NARUON_CONSUMER_CODE
        );
        headers.insert("authorization".into(), "secret".into());
        assert_eq!(
            require_headers(&headers, None),
            Err(ApiError::AuthorizationDenied)
        );

        let mut headers = valid_headers(NARUON_CONSUMER_CODE);
        headers.insert("transfer-encoding".into(), "chunked".into());
        assert_eq!(
            require_headers(&headers, None),
            Err(ApiError::InvalidWirePayload)
        );
        let mut headers = valid_headers(NARUON_CONSUMER_CODE);
        headers.insert("host".into(), "postgres.internal".into());
        assert_eq!(
            require_headers(&headers, None),
            Err(ApiError::InvalidWirePayload)
        );
        let mut headers = valid_headers(NARUON_CONSUMER_CODE);
        headers.insert("host".into(), "example.test".into());
        assert_eq!(
            require_headers(&headers, None),
            Err(ApiError::AuthorizationDenied)
        );
        let mut headers = valid_headers(NARUON_CONSUMER_CODE);
        headers.insert("content-type".into(), "text/plain".into());
        assert_eq!(
            require_headers(&headers, None),
            Err(ApiError::InvalidWirePayload)
        );
        let mut headers = valid_headers(NARUON_CONSUMER_CODE);
        headers.insert("tepp-contract-version".into(), "2".into());
        assert_eq!(
            require_headers(&headers, None),
            Err(ApiError::InvalidWirePayload)
        );
        let mut headers = valid_headers("unpublished");
        assert_eq!(
            require_headers(&headers, None),
            Err(ApiError::InvalidWirePayload)
        );
        headers.insert("host".into(), String::new());
        assert_eq!(
            header_value(&headers, "host"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            header_value(&HashMap::new(), "host"),
            Err(ApiError::InvalidWirePayload)
        );
        assert!(host_implies_table_access("jdbc://database"));
        assert!(host_implies_table_access("/sql/query"));
        assert!(host_implies_table_access("/tables/users"));
        assert!(host_implies_table_access("quoted'value"));
        assert!(host_implies_table_access("semicolon;value"));
        assert!(host_implies_table_access("back\\slash"));
        assert!(host_implies_table_access("space value"));
        assert!(host_implies_table_access("control\nvalue"));
        assert!(!host_implies_table_access("127.0.0.1"));
    }

    #[test]
    fn status_io_and_error_fallback_mappings_are_stable() {
        for (error, expected) in [
            (ApiError::InvalidWirePayload, (400, "Bad Request")),
            (ApiError::AuthorizationDenied, (403, "Forbidden")),
            (ApiError::LimitExceeded, (413, "Payload Too Large")),
            (
                ApiError::UnsupportedContractVersion,
                (422, "Unprocessable Entity"),
            ),
        ] {
            assert_eq!(status_for(error), expected);
        }
        for kind in [
            std::io::ErrorKind::TimedOut,
            std::io::ErrorKind::WouldBlock,
            std::io::ErrorKind::Other,
        ] {
            let mapped = map_io_error(&std::io::Error::from(kind));
            assert_eq!(
                mapped,
                if matches!(
                    kind,
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) {
                    ApiError::LimitExceeded
                } else {
                    ApiError::InvalidWirePayload
                }
            );
        }
        let fallback = error_envelope_or_fallback(Err(ApiError::InvalidWirePayload));
        assert!(fallback.contains("analysis-run-live-fallback"));
        assert_eq!(
            error_envelope_or_fallback(Ok("valid-envelope".into())),
            "valid-envelope"
        );
    }
}
