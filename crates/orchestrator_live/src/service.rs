//! Loopback-only live HTTP/1.1 listener for interpretation POSTs (ADR 0010/0011).

use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};

use crate::error::OrchestratorLiveError;
use crate::http::{
    OrchestratorLiveResponse, header_value, map_io_error, parse_headers, parse_request_line,
    read_http_request, refuse_collection_get_headers, refuse_live_headers, split_request,
    status_for, write_response,
};
use crate::interpretation_run_collection_http::{
    InterpretationRunCollection, InterpretationRunCollectionItem,
    is_interpretation_run_collection_path, page_interpretation_run_collection_items,
    parse_interpretation_run_collection_page_cursor,
    parse_interpretation_run_collection_page_limit,
};
use crate::request::{
    INTERPRETATION_RUN_PATH, InterpretationRunAccepted, InterpretationRunRequest, to_json,
};

/// Loopback live HTTP/1.1 service for contextual-orchestrator interpretation POSTs.
///
/// Production interchange remains optional and versioned. This listener binds
/// loopback TCP so tests and standalone operation can prove request handling
/// without TLS termination, table access, or scientific-authority promotion.
/// `GET /v1/interpretation-runs` enumerates accepted hypothetical runs as
/// metric-free identities.
#[derive(Debug)]
pub struct OrchestratorLiveService {
    listener: Option<TcpListener>,
    bound_addr: Option<SocketAddr>,
    next_run_serial: u64,
    next_request_serial: u64,
    accepted_runs: HashMap<String, (InterpretationRunRequest, InterpretationRunAccepted)>,
}

impl Default for OrchestratorLiveService {
    fn default() -> Self {
        Self::new()
    }
}

impl OrchestratorLiveService {
    /// Construct an in-memory handler with no socket.
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

    /// Bind `127.0.0.1:0`.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::InvalidWirePayload`] when the operating
    /// system refuses the loopback bind.
    pub fn bind_loopback() -> Result<Self, OrchestratorLiveError> {
        Self::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
    }

    /// Bind a caller-supplied address after refusing non-loopback IPs.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::AuthorizationDenied`] for a non-loopback
    /// bind address and [`OrchestratorLiveError::InvalidWirePayload`] when the
    /// socket cannot be opened.
    pub fn bind(addr: SocketAddr) -> Result<Self, OrchestratorLiveError> {
        if !addr.ip().is_loopback() {
            return Err(OrchestratorLiveError::AuthorizationDenied);
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
    /// Returns [`OrchestratorLiveError::InvalidWirePayload`] when no socket is bound.
    pub fn local_addr(&self) -> Result<SocketAddr, OrchestratorLiveError> {
        self.bound_addr
            .ok_or(OrchestratorLiveError::InvalidWirePayload)
    }

    /// Accept one TCP connection and serve one HTTP/1.1 request.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::InvalidWirePayload`] when no socket is
    /// bound or the accept/write path fails for a non-timeout reason. Timeouts
    /// map to [`OrchestratorLiveError::LimitExceeded`].
    pub fn serve_one(&mut self) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
        let listener = self
            .listener
            .as_ref()
            .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
        self.serve_accepted(listener.accept().map(|(stream, _)| stream))
    }

    /// Serve one already-accepted stream, or map an accept failure.
    ///
    /// # Errors
    ///
    /// Returns transport mapping errors. Request-protocol failures become HTTP
    /// error responses and are returned as `Ok`.
    pub fn serve_accepted(
        &mut self,
        accepted: Result<TcpStream, std::io::Error>,
    ) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
        let mut stream = accepted.map_err(|error| map_io_error(&error))?;
        let response = match read_http_request(&mut stream) {
            Ok(request) => self.handle_http_request(&request),
            Err(error) => self.response_from_error(error),
        };
        write_response(&mut stream, &response)?;
        Ok(response)
    }

    /// Parse and handle a complete HTTP/1.1 request already in memory.
    #[must_use]
    pub fn handle_http_request(&mut self, request: &str) -> OrchestratorLiveResponse {
        match self.dispatch_http_request(request) {
            Ok(response) => response,
            Err(error) => self.response_from_error(error),
        }
    }

    /// Read one HTTP/1.1 request from `reader`, including the declared body.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::LimitExceeded`] on timeout or when
    /// headers exceed the live header byte limit. Other read/framing failures
    /// are [`OrchestratorLiveError::InvalidWirePayload`].
    pub fn read_http_request<R: std::io::Read>(
        reader: &mut R,
    ) -> Result<String, OrchestratorLiveError> {
        read_http_request(reader)
    }

    /// Write one HTTP/1.1 response to `writer`.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::InvalidWirePayload`] when the write fails.
    pub fn write_response<W: std::io::Write>(
        writer: &mut W,
        response: &OrchestratorLiveResponse,
    ) -> Result<(), OrchestratorLiveError> {
        write_response(writer, response)
    }

    fn dispatch_http_request(
        &mut self,
        request: &str,
    ) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
        let (header_block, body) = split_request(request)?;
        let mut lines = header_block.split("\r\n");
        let request_line = lines.next().unwrap_or("");
        let (method, path) = parse_request_line(request_line)?;
        let headers = parse_headers(lines)?;
        if method == "GET" {
            return self.list_interpretation_runs(path, &headers, body);
        }
        if method != "POST" || path != INTERPRETATION_RUN_PATH {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        refuse_live_headers(&headers)?;
        self.accept_interpretation_run(&headers, body)
    }

    fn list_interpretation_runs(
        &self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
        if !is_interpretation_run_collection_path(path) {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        if !body.is_empty() {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        refuse_collection_get_headers(headers)?;
        let limit = parse_interpretation_run_collection_page_limit(
            headers.get("tepp-page-limit").map(String::as_str),
        )?;
        let cursor = parse_interpretation_run_collection_page_cursor(
            headers.get("tepp-page-cursor").map(String::as_str),
        )?;
        let items = self
            .accepted_runs
            .values()
            .map(|(_, accepted)| {
                InterpretationRunCollectionItem::new(
                    accepted.interpretation_run_id(),
                    accepted.idempotency_key(),
                    accepted.orchestration_mode(),
                    accepted.claim_status(),
                    accepted.scientific_authority(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (page, next_cursor) =
            page_interpretation_run_collection_items(items, cursor.as_deref(), limit);
        let collection = InterpretationRunCollection::new(page, next_cursor)?;
        Ok(OrchestratorLiveResponse::json(
            200,
            "OK",
            collection.to_json()?,
        ))
    }

    fn accept_interpretation_run(
        &mut self,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
        let request = InterpretationRunRequest::from_json(body)?;
        let idempotency_key = header_value(headers, "idempotency-key")?;
        if idempotency_key != request.idempotency_key() {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        if let Some((stored_request, stored_accepted)) = self.accepted_runs.get(idempotency_key) {
            if stored_request == &request {
                return Ok(OrchestratorLiveResponse::json(
                    202,
                    "Accepted",
                    stored_accepted.to_json()?,
                ));
            }
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        let run_id = format!("orch-run-{}", self.next_run_serial);
        self.next_run_serial += 1;
        let accepted = InterpretationRunAccepted::from_validated_request(run_id, &request);
        let body = accepted.to_json()?;
        self.accepted_runs
            .insert(idempotency_key.to_owned(), (request, accepted));
        Ok(OrchestratorLiveResponse::json(202, "Accepted", body))
    }

    fn response_from_error(&mut self, error: OrchestratorLiveError) -> OrchestratorLiveResponse {
        let request_id = format!("orch-live-{}", self.next_request_serial);
        self.next_request_serial += 1;
        let (status_code, reason_phrase) = status_for(error);
        OrchestratorLiveResponse::json(status_code, reason_phrase, envelope_json(error, request_id))
    }
}

fn envelope_json(error: OrchestratorLiveError, request_id: String) -> String {
    let payload = ErrorWire {
        error_code: error_code(error),
        message: error.to_string(),
        request_id,
        retryable: false,
    };
    to_json(&payload).unwrap_or_else(|_| fallback_envelope_json())
}

fn error_code(error: OrchestratorLiveError) -> &'static str {
    match error {
        OrchestratorLiveError::InvalidWirePayload => "invalid_wire_payload",
        OrchestratorLiveError::UnsupportedContractVersion => "unsupported_contract_version",
        OrchestratorLiveError::LimitExceeded => "limit_exceeded",
        OrchestratorLiveError::AuthorizationDenied => "authorization_denied",
        OrchestratorLiveError::ScientificAuthorityRefused => "scientific_authority_refused",
    }
}

fn fallback_envelope_json() -> String {
    "{\"error_code\":\"invalid_wire_payload\",\"message\":\"invalid orchestrator wire payload\",\"request_id\":\"orch-live-fallback\",\"retryable\":false}".to_owned()
}

#[derive(serde::Serialize)]
struct ErrorWire {
    error_code: &'static str,
    message: String,
    request_id: String,
    retryable: bool,
}

#[cfg(test)]
mod tests {
    use super::{OrchestratorLiveService, envelope_json, fallback_envelope_json};
    use crate::error::OrchestratorLiveError;

    #[test]
    fn helpers_cover_accept_failure_and_envelope_fallback() {
        assert!(!fallback_envelope_json().is_empty());
        assert!(
            envelope_json(OrchestratorLiveError::InvalidWirePayload, String::new())
                .contains("invalid_wire_payload")
        );
        assert!(
            envelope_json(OrchestratorLiveError::LimitExceeded, "req-1".into())
                .contains("limit_exceeded")
        );
        assert!(
            envelope_json(
                OrchestratorLiveError::ScientificAuthorityRefused,
                "req-2".into()
            )
            .contains("scientific_authority_refused")
        );
        assert_eq!(
            OrchestratorLiveService::new()
                .serve_accepted(Err(std::io::Error::other("accept")))
                .expect_err("accept"),
            OrchestratorLiveError::InvalidWirePayload
        );
    }
}
