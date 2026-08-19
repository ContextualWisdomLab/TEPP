//! Consumer-neutral live analysis-run ingress for modular CWL services.
//!
//! This module keeps the Naruon compatibility listener intact while providing
//! the shared `/v1/analysis-runs` boundary needed by Naruon and LineageWeave.
//! It accepts transport acknowledgements only; completed psychometric results
//! remain outside this crate.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use crate::lineageweave_http::{
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, consumer_is_supported,
};
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, header_is_credential};
use crate::{
    AnalysisRunAccepted, AnalysisRunRequest, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
    ErrorEnvelope, NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT,
    NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse, requests_are_idempotent_matches,
};

/// Loopback HTTP/1.1 analysis-run service shared by published CWL consumers.
///
/// The service accepts only Naruon and LineageWeave consumer identities. Its
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
        require_request_line(lines.next().unwrap_or(""))?;
        let headers = parse_headers(lines)?;
        let consumer = require_headers(&headers, self.bound_addr)?;
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

fn read_http_request<R: Read>(reader: &mut R) -> Result<String, ApiError> {
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
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|error| map_io_error(&error))?;
    }
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

fn require_request_line(line: &str) -> Result<(), ApiError> {
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

fn parse_headers<'a, I>(lines: I) -> Result<HashMap<String, String>, ApiError>
where
    I: Iterator<Item = &'a str>,
{
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
    let (name, value) = line
        .split_once(':')
        .ok_or(ApiError::InvalidWirePayload)?;
    if name.is_empty() || name.chars().any(|ch| ch.is_whitespace() || ch.is_control()) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok((name, value.trim()))
}

fn require_headers<'a>(
    headers: &'a HashMap<String, String>,
    bound_addr: Option<SocketAddr>,
) -> Result<&'a str, ApiError> {
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

fn host_is_loopback(host: &str, bound_addr: Option<SocketAddr>) -> bool {
    if let Some(bound) = bound_addr
        && (host == bound.to_string() || host == bound.ip().to_string())
    {
        return true;
    }
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if let Some(port) = host.strip_prefix("localhost:") {
        return !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit());
    }
    if let Ok(addr) = host.parse::<SocketAddr>() {
        return addr.ip().is_loopback();
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return ip.is_loopback();
    }
    false
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
    use super::{
        AnalysisRunLiveService, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
        consumer_tenant_idempotency_key, host_is_loopback,
    };
    use crate::ApiError;

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
            AnalysisRunLiveService::bind("0.0.0.0:0".parse().expect("addr"))
                .expect_err("denied"),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunLiveService::new().local_addr().expect_err("unbound"),
            ApiError::InvalidWirePayload
        );
    }
}
