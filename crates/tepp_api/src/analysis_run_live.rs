//! Consumer-neutral live analysis-run ingress for modular CWL services.
//!
//! This module keeps the Naruon compatibility listener intact while providing
//! the shared `/v1/analysis-runs` and cutoff-safe `/v1/temporal-context`
//! boundaries needed by Naruon and `LineageWeave`. Naruon may also POST and
//! GET `/v1/exports/{export_id}` for metric-free purpose-bound retrieval
//! and `GET /v1/exports/by-idempotency/{idempotency_key}` for key lookup.
//! `GET /v1/exports/by-idempotency/{idempotency_key}/request` returns the stored
//! export-authorization request of that unique accepted export.
//! It accepts transport acknowledgements, temporal evidence context, and
//! export identities only; completed psychometric results remain outside this
//! crate.

use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpListener};

use crate::export_http::{export_retrieval_path_id, refuse_metrics_on_export_retrieval_payload};
use crate::export_idempotency_lookup_http::{
    ExportIdempotencyLookup, export_idempotency_lookup_path_key,
    refuse_metrics_on_export_idempotency_lookup_payload,
};
use crate::export_idempotency_lookup_stored_request_http::{
    export_idempotency_lookup_stored_request_path_key,
    refuse_metrics_on_export_lookup_stored_request_payload,
};
use crate::lineageweave_http::{
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, consumer_is_supported,
};
use crate::live_http::{
    header_value, map_io_error, parse_headers, parse_request_line, read_http_request_with_limit,
    split_request_with_limit, validate_common_headers,
};
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, NARUON_EXPORT_PATH};
use crate::{
    AnalysisRunAccepted, AnalysisRunRequest, AnalyticalPurpose, ApiError,
    DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, ErrorEnvelope, ExportAuthorizationRequest, ExportRetrieval,
    NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse, PROJECT_HISTORY_PATH, ProjectHistoryProjection,
    ProjectHistoryRequest, TEMPORAL_CONTEXT_PATH, TemporalContextRequest, authorize_export,
    build_temporal_context, project_history_projection, requests_are_idempotent_matches,
    require_export_allowed,
};

const MAX_LIVE_REQUEST_BODY_BYTES: usize = DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;

#[derive(Clone, Debug, Eq, PartialEq)]
struct StoredExport {
    request: ExportAuthorizationRequest,
    retrieval: ExportRetrieval,
}

#[cfg(test)]
use crate::live_http::{declared_content_length, host_implies_table_access, split_header_line};

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
    accepted_project_histories: HashMap<String, (ProjectHistoryRequest, ProjectHistoryProjection)>,
    authorized_exports: HashMap<String, StoredExport>,
    exports_by_id: HashMap<String, String>,
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
            accepted_project_histories: HashMap::new(),
            authorized_exports: HashMap::new(),
            exports_by_id: HashMap::new(),
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
            if matches!(
                export_idempotency_lookup_stored_request_path_key(path),
                Ok(_) | Err(ApiError::LimitExceeded)
            ) {
                return self.lookup_export_stored_request_by_idempotency(path, &headers, body);
            }
            if matches!(
                export_idempotency_lookup_path_key(path),
                Ok(_) | Err(ApiError::LimitExceeded)
            ) {
                return self.lookup_export_by_idempotency(path, &headers, body);
            }
            if matches!(
                export_retrieval_path_id(path),
                Ok(_) | Err(ApiError::LimitExceeded)
            ) {
                return self.read_export(path, &headers, body);
            }
            return Err(ApiError::InvalidWirePayload);
        }
        if method != "POST" {
            return Err(ApiError::InvalidWirePayload);
        }
        if path == NARUON_EXPORT_PATH {
            return self.accept_export(&headers, body);
        }
        if path != NARUON_ANALYSIS_RUN_PATH
            && path != TEMPORAL_CONTEXT_PATH
            && path != PROJECT_HISTORY_PATH
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

    fn accept_export(
        &mut self,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let consumer = require_headers(headers, self.bound_addr, true)?;
        if consumer != NARUON_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        let request: ExportAuthorizationRequest = crate::wire::from_json(body)?;
        let idempotency_key = header_value(headers, "idempotency-key")?;
        if idempotency_key == request.principal_id {
            return Err(ApiError::InvalidWirePayload);
        }
        if request.purpose != AnalyticalPurpose::ModularServiceConsumer {
            return Err(ApiError::AuthorizationDenied);
        }
        let decision = authorize_export(&request)?;
        require_export_allowed(&decision)?;
        let replay_key = consumer_tenant_idempotency_key(
            consumer,
            &request.tenant_workspace_id,
            idempotency_key,
        );
        if let Some(stored) = self.authorized_exports.get(&replay_key) {
            if stored.request == request {
                refuse_metrics_on_export_retrieval_payload(&stored.retrieval.to_json()?)?;
                return Ok(json_response(200, "OK", stored.retrieval.to_json()?));
            }
            return Err(ApiError::InvalidWirePayload);
        }
        let export_id = uuid::Uuid::now_v7().hyphenated().to_string();
        let retrieval = ExportRetrieval::new(
            export_id.clone(),
            request.artifact_id.clone(),
            decision.decision_code().to_owned(),
            request.purpose.wire_name(),
            idempotency_key,
        )?;
        let response_body = retrieval.to_json()?;
        refuse_metrics_on_export_retrieval_payload(&response_body)?;
        self.exports_by_id.insert(export_id, replay_key.clone());
        self.authorized_exports
            .insert(replay_key, StoredExport { request, retrieval });
        Ok(json_response(200, "OK", response_body))
    }

    fn read_export(
        &self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let export_id = export_retrieval_path_id(path)?;
        if !body.trim().is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        let consumer = require_headers(headers, self.bound_addr, false)?;
        if consumer != NARUON_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_metrics_on_export_retrieval_payload(body)?;
        let replay_key = self
            .exports_by_id
            .get(&export_id)
            .cloned()
            .ok_or(ApiError::InvalidWirePayload)?;
        let stored = self
            .authorized_exports
            .get(&replay_key)
            .ok_or(ApiError::InvalidWirePayload)?;
        let response_body = stored.retrieval.to_json()?;
        refuse_metrics_on_export_retrieval_payload(&response_body)?;
        Ok(json_response(200, "OK", response_body))
    }

    fn lookup_export_by_idempotency(
        &self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let idempotency_key = export_idempotency_lookup_path_key(path)?;
        if !body.trim().is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        let consumer = require_headers(headers, self.bound_addr, false)?;
        if consumer != NARUON_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_metrics_on_export_idempotency_lookup_payload(body)?;
        let prefix = format!("{consumer}\u{1f}");
        let mut matches: Vec<&StoredExport> = self
            .authorized_exports
            .iter()
            .filter(|(replay_key, stored)| {
                replay_key.starts_with(&prefix)
                    && stored.retrieval.idempotency_key == idempotency_key
            })
            .map(|(_, stored)| stored)
            .collect();
        if matches.len() != 1 {
            return Err(ApiError::InvalidWirePayload);
        }
        let stored = matches.remove(0);
        let payload = ExportIdempotencyLookup::new(
            stored.retrieval.export_id.clone(),
            stored.retrieval.decision_code.clone(),
            stored.retrieval.idempotency_key.clone(),
        )?;
        let response_body = payload.to_json()?;
        refuse_metrics_on_export_idempotency_lookup_payload(&response_body)?;
        Ok(json_response(200, "OK", response_body))
    }

    fn lookup_export_stored_request_by_idempotency(
        &self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let idempotency_key = export_idempotency_lookup_stored_request_path_key(path)?;
        if !body.trim().is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        let consumer = require_headers(headers, self.bound_addr, false)?;
        if consumer != NARUON_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_metrics_on_export_lookup_stored_request_payload(body)?;
        let prefix = format!("{consumer}\u{1f}");
        let mut matches: Vec<&StoredExport> = self
            .authorized_exports
            .iter()
            .filter(|(replay_key, stored)| {
                replay_key.starts_with(&prefix)
                    && stored.retrieval.idempotency_key == idempotency_key
            })
            .map(|(_, stored)| stored)
            .collect();
        if matches.len() != 1 {
            return Err(ApiError::InvalidWirePayload);
        }
        let stored = matches.remove(0);
        let response_body = crate::wire::to_json(&stored.request)?;
        refuse_metrics_on_export_lookup_stored_request_payload(&response_body)?;
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
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunRequest, ApiError,
        DEFAULT_ANALYSIS_RUN_BYTE_LIMIT, ErrorEnvelope, LINEAGEWEAVE_CONSUMER_CODE,
        NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE, NARUON_EXPORT_PATH,
        NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT, NARUON_LIVE_IO_TIMEOUT,
        TEMPORAL_CONTEXT_PATH,
    };

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
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "PUT {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\ncontent-length: 0\r\n\r\n"
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
    fn handler_covers_metric_free_export_retrieval_get() {
        use crate::{AnalyticalPurpose, ExportAuthorizationRequest, ExportRetrieval};

        let request = ExportAuthorizationRequest {
            tenant_workspace_id: "export-live-tenant".into(),
            principal_id: "principal-analyst-1".into(),
            purpose: AnalyticalPurpose::ModularServiceConsumer,
            artifact_id: "artifact-live-1".into(),
            includes_source_text: false,
        };
        let body = crate::wire::to_json(&request).expect("export json");
        let mut service = AnalysisRunLiveService::new();
        let posted = service.handle_http_request(&export_post_http(
            &body,
            NARUON_CONSUMER_CODE,
            "export-idem-1",
        ));
        assert_eq!(posted.status_code, 200);
        let retrieval = ExportRetrieval::from_json(&posted.body).expect("posted retrieval");
        let parsed_export_id =
            uuid::Uuid::parse_str(&retrieval.export_id).expect("UUID capability");
        assert_eq!(parsed_export_id.get_version_num(), 7);
        assert_eq!(retrieval.artifact_id, "artifact-live-1");
        assert_eq!(retrieval.purpose, "modular_service_consumer");
        assert_eq!(retrieval.decision_code, "purpose_bound_export_allowed");
        assert_eq!(retrieval.idempotency_key, "export-idem-1");
        assert!(!posted.body.contains("tenant_workspace_id"));
        assert!(!posted.body.contains("principal_id"));
        assert!(!posted.body.contains("includes_source_text"));
        assert!(!posted.body.contains("scientific_acceptance"));
        assert!(!posted.body.contains("rmse"));

        let replay = service.handle_http_request(&export_post_http(
            &body,
            NARUON_CONSUMER_CODE,
            "export-idem-1",
        ));
        assert_eq!(replay.status_code, 200);
        assert_eq!(replay.body, posted.body);

        let got = service
            .handle_http_request(&export_get_http(&retrieval.export_id, NARUON_CONSUMER_CODE));
        assert_eq!(got.status_code, 200);
        assert_eq!(
            ExportRetrieval::from_json(&got.body).expect("got"),
            retrieval
        );

        assert_eq!(
            service
                .handle_http_request(&export_get_http(
                    &retrieval.export_id,
                    LINEAGEWEAVE_CONSUMER_CODE
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&export_post_http(
                    &body,
                    LINEAGEWEAVE_CONSUMER_CODE,
                    "export-idem-2"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&export_get_http("missing-export", NARUON_CONSUMER_CODE))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_EXPORT_PATH}/{}/extra HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
                    retrieval.export_id
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_EXPORT_PATH}/{} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 2\r\n\r\n{{}}",
                    retrieval.export_id
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "POST {NARUON_EXPORT_PATH}/{} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: export-idem-1\r\ncontent-length: 0\r\n\r\n",
                    retrieval.export_id
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH}/tepp-run-1 HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );

        let mut denied = request.clone();
        denied.purpose = AnalyticalPurpose::OperationalMonitoring;
        denied.includes_source_text = true;
        let denied_body = crate::wire::to_json(&denied).expect("denied json");
        assert_eq!(
            service
                .handle_http_request(&export_post_http(
                    &denied_body,
                    NARUON_CONSUMER_CODE,
                    "export-idem-denied",
                ))
                .status_code,
            403
        );

        let mut conflict = request.clone();
        conflict.artifact_id = "artifact-live-2".into();
        let conflict_body = crate::wire::to_json(&conflict).expect("conflict json");
        assert_eq!(
            service
                .handle_http_request(&export_post_http(
                    &conflict_body,
                    NARUON_CONSUMER_CODE,
                    "export-idem-1",
                ))
                .status_code,
            400
        );

        let looked_up = service.handle_http_request(&export_lookup_http(
            "export-idem-1",
            NARUON_CONSUMER_CODE,
        ));
        assert_eq!(looked_up.status_code, 200);
        let lookup = crate::ExportIdempotencyLookup::from_json(&looked_up.body).expect("lookup");
        assert_eq!(lookup.export_id, retrieval.export_id);
        assert_eq!(lookup.idempotency_key, "export-idem-1");
        assert_eq!(lookup.decision_code, "purpose_bound_export_allowed");
        assert!(!looked_up.body.contains("tenant_workspace_id"));
        assert!(!looked_up.body.contains("principal_id"));
        assert!(!looked_up.body.contains("includes_source_text"));
        assert!(!looked_up.body.contains("scientific_acceptance"));
        assert!(!looked_up.body.contains("rmse"));
        assert_eq!(
            service
                .handle_http_request(&export_lookup_http(
                    "export-idem-1",
                    LINEAGEWEAVE_CONSUMER_CODE
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&export_lookup_http("missing-key", NARUON_CONSUMER_CODE))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&export_lookup_body_http(
                    "export-idem-1",
                    NARUON_CONSUMER_CODE,
                    "{}",
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&export_lookup_post_http(
                    "export-idem-1",
                    NARUON_CONSUMER_CODE,
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&export_get_http("by-idempotency", NARUON_CONSUMER_CODE))
                .status_code,
            400
        );

        let mut other_tenant = request.clone();
        other_tenant.tenant_workspace_id = "export-live-tenant-b".into();
        let other_body = crate::wire::to_json(&other_tenant).expect("other json");
        let other_posted = service.handle_http_request(&export_post_http(
            &other_body,
            NARUON_CONSUMER_CODE,
            "export-idem-1",
        ));
        assert_eq!(other_posted.status_code, 200);
        assert_eq!(
            service
                .handle_http_request(&export_lookup_http("export-idem-1", NARUON_CONSUMER_CODE))
                .status_code,
            400
        );

        let principal_as_key = service.handle_http_request(&export_post_http(
            &body,
            NARUON_CONSUMER_CODE,
            "principal-analyst-1",
        ));
        assert_eq!(principal_as_key.status_code, 400);
        assert_eq!(
            service
                .handle_http_request(&export_post_http(
                    &body,
                    NARUON_CONSUMER_CODE,
                    &"k".repeat(crate::EXPORT_RETRIEVAL_ID_MAX_LEN + 1),
                ))
                .status_code,
            413
        );
    }

    fn export_post_http(body: &str, consumer: &str, idempotency_key: &str) -> String {
        format!(
            "POST {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        )
    }

    fn export_get_http(export_id: &str, consumer: &str) -> String {
        format!(
            "GET {NARUON_EXPORT_PATH}/{export_id} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
        )
    }

    fn export_lookup_http(idempotency_key: &str, consumer: &str) -> String {
        export_lookup_body_http(idempotency_key, consumer, "")
    }

    fn export_lookup_body_http(idempotency_key: &str, consumer: &str, body: &str) -> String {
        format!(
            "GET {NARUON_EXPORT_PATH}/by-idempotency/{idempotency_key} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        )
    }

    fn export_lookup_post_http(idempotency_key: &str, consumer: &str) -> String {
        format!(
            "POST {NARUON_EXPORT_PATH}/by-idempotency/{idempotency_key} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: 0\r\n\r\n"
        )
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
