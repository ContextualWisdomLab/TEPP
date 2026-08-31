//! Consumer-neutral live analysis-run ingress for modular CWL services.
//!
//! This module keeps the Naruon compatibility listener intact while providing
//! the shared `/v1/analysis-runs` and cutoff-safe `/v1/temporal-context`
//! boundaries needed by Naruon and `LineageWeave`. It accepts transport
//! acknowledgements and temporal evidence context only; completed psychometric
//! results remain outside this crate. `POST /v1/analysis-runs/{run_id}/cancel`
//! is the operator-visible cancel path: accepted and running runs become
//! metric-free `cancelled` status. `GET /v1/analysis-runs` enumerates those
//! runs without guessing identities. `POST /v1/analysis-runs/{run_id}/retry`
//! clones a failed or cancelled run into a new metric-free `202 Accepted`.
//! `GET /v1/analysis-runs/{run_id}/request` returns metric-free stored create
//! fields so operators can inspect snapshot, cutoff, model, and profile before
//! retry. GET-by-id and running/terminal POST transitions remain later slices.

use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpListener};

use crate::analysis_run_cancel_http::{
    AnalysisRunCancelRequest, analysis_run_cancel_path_run_id, refuse_metrics_on_cancel_payload,
};
use crate::analysis_run_collection_http::{
    AnalysisRunCollection, AnalysisRunCollectionItem, is_analysis_run_collection_path,
    parse_collection_page_cursor, parse_collection_page_limit,
    refuse_metrics_on_collection_payload,
};
use crate::analysis_run_retry_http::{
    AnalysisRunRetryRequest, analysis_run_retry_path_run_id, refuse_metrics_on_retry_payload,
};
use crate::analysis_run_stored_request_http::{
    AnalysisRunStoredRequest, analysis_run_stored_request_path_run_id,
    refuse_metrics_on_stored_request_payload,
};
use crate::lineageweave_http::{LINEAGEWEAVE_CONSUMER_CODE, consumer_is_supported};
use crate::live_http::{
    header_value, map_io_error, parse_headers, parse_request_line, read_http_request_with_limit,
    split_request_with_limit, validate_common_headers,
};
use crate::naruon_http::NARUON_ANALYSIS_RUN_PATH;
use crate::{
    AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunStatus, AnalysisRunStatusState, ApiError,
    DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, ErrorEnvelope, NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse,
    PROJECT_HISTORY_PATH, ProjectHistoryProjection, ProjectHistoryRequest, TEMPORAL_CONTEXT_PATH,
    TemporalContextRequest, build_temporal_context, project_history_projection,
    requests_are_idempotent_matches,
};

const MAX_LIVE_REQUEST_BODY_BYTES: usize = DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;

#[cfg(test)]
use crate::live_http::{declared_content_length, host_implies_table_access, split_header_line};

/// One accepted loopback analysis run and its current lifecycle state.
#[derive(Clone, Debug, Eq, PartialEq)]
struct LiveAnalysisRun {
    consumer: String,
    request: AnalysisRunRequest,
    accepted: AnalysisRunAccepted,
    run_state: AnalysisRunStatusState,
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
            if matches!(
                analysis_run_stored_request_path_run_id(path),
                Ok(_) | Err(ApiError::LimitExceeded)
            ) {
                return self.read_analysis_run_stored_request(path, &headers, body);
            }
            return self.list_analysis_runs(path, &headers, body);
        }
        if method != "POST" {
            return Err(ApiError::InvalidWirePayload);
        }
        if matches!(
            analysis_run_retry_path_run_id(path),
            Ok(_) | Err(ApiError::LimitExceeded)
        ) {
            return self.retry_analysis_run(path, &headers, body);
        }
        if matches!(
            analysis_run_cancel_path_run_id(path),
            Ok(_) | Err(ApiError::LimitExceeded)
        ) {
            return self.cancel_analysis_run(path, &headers, body);
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
        if let Some(stored) = self.accepted_runs.get(&replay_key) {
            if requests_are_idempotent_matches(&stored.request, &request) {
                return Ok(json_response(202, "Accepted", stored.accepted.to_json()?));
            }
            return Err(ApiError::InvalidWirePayload);
        }
        let run_id = format!("tepp-run-{}", self.next_run_serial);
        self.next_run_serial += 1;
        let accepted =
            AnalysisRunAccepted::new(run_id.clone(), "accepted", request.idempotency_key.clone())?;
        let response_body = accepted.to_json()?;
        refuse_metrics_on_cancel_payload(&response_body)?;
        self.runs_by_id.insert(run_id, replay_key.clone());
        self.accepted_runs.insert(
            replay_key,
            LiveAnalysisRun {
                consumer: consumer.to_owned(),
                request,
                accepted,
                run_state: AnalysisRunStatusState::Accepted,
            },
        );
        Ok(json_response(202, "Accepted", response_body))
    }

    fn cancel_analysis_run(
        &mut self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let run_id = analysis_run_cancel_path_run_id(path)?;
        let consumer = require_headers(headers, self.bound_addr, true)?;
        refuse_metrics_on_cancel_payload(body)?;
        let idempotency_key = header_value(headers, "idempotency-key")?;
        if !body.trim().is_empty() {
            let cancel = AnalysisRunCancelRequest::from_json(body)?;
            if cancel.run_id != run_id || cancel.idempotency_key != idempotency_key {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        let replay_key = self
            .runs_by_id
            .get(&run_id)
            .cloned()
            .ok_or(ApiError::InvalidWirePayload)?;
        let stored = self
            .accepted_runs
            .get_mut(&replay_key)
            .ok_or(ApiError::InvalidWirePayload)?;
        if stored.consumer != consumer || stored.accepted.idempotency_key != idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        match stored.run_state {
            AnalysisRunStatusState::Accepted | AnalysisRunStatusState::Running => {
                stored.run_state = AnalysisRunStatusState::Cancelled;
            }
            AnalysisRunStatusState::Cancelled => {}
            AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed => {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        let status = AnalysisRunStatus::cancelled(&stored.accepted)?;
        let response_body = status.to_json()?;
        refuse_metrics_on_cancel_payload(&response_body)?;
        Ok(json_response(200, "OK", response_body))
    }

    fn retry_analysis_run(
        &mut self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let run_id = analysis_run_retry_path_run_id(path)?;
        let consumer = require_headers(headers, self.bound_addr, true)?;
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
        let (parent_consumer, mut cloned_request, parent_idempotency_key, parent_state) = {
            let stored = self
                .accepted_runs
                .get(&parent_replay_key)
                .ok_or(ApiError::InvalidWirePayload)?;
            (
                stored.consumer.clone(),
                stored.request.clone(),
                stored.accepted.idempotency_key.clone(),
                stored.run_state,
            )
        };
        if parent_consumer != consumer {
            return Err(ApiError::InvalidWirePayload);
        }
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
        let replay_key = consumer_tenant_idempotency_key(
            consumer,
            &cloned_request.tenant_workspace_id,
            new_idempotency_key,
        );
        if let Some(stored) = self.accepted_runs.get(&replay_key) {
            if requests_are_idempotent_matches(&stored.request, &cloned_request) {
                let response_body = stored.accepted.to_json()?;
                refuse_metrics_on_retry_payload(&response_body)?;
                return Ok(json_response(202, "Accepted", response_body));
            }
            return Err(ApiError::InvalidWirePayload);
        }
        let child_run_id = format!("tepp-run-{}", self.next_run_serial);
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
            LiveAnalysisRun {
                consumer: consumer.to_owned(),
                request: cloned_request,
                accepted,
                run_state: AnalysisRunStatusState::Accepted,
            },
        );
        Ok(json_response(202, "Accepted", response_body))
    }

    fn read_analysis_run_stored_request(
        &self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        let run_id = analysis_run_stored_request_path_run_id(path)?;
        if !body.trim().is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        let consumer = require_headers(headers, self.bound_addr, false)?;
        refuse_metrics_on_stored_request_payload(body)?;
        let replay_key = self
            .runs_by_id
            .get(&run_id)
            .cloned()
            .ok_or(ApiError::InvalidWirePayload)?;
        let stored = self
            .accepted_runs
            .get(&replay_key)
            .ok_or(ApiError::InvalidWirePayload)?;
        if stored.consumer != consumer {
            return Err(ApiError::InvalidWirePayload);
        }
        let payload = AnalysisRunStoredRequest::new(
            stored.accepted.run_id.clone(),
            stored.run_state,
            stored.accepted.idempotency_key.clone(),
            stored.request.snapshot_id.clone(),
            stored.request.knowledge_cutoff.clone(),
            stored.request.model_contract_version.clone(),
            stored.request.output_profile.clone(),
        )?;
        let response_body = payload.to_json()?;
        refuse_metrics_on_stored_request_payload(&response_body)?;
        Ok(json_response(200, "OK", response_body))
    }

    fn list_analysis_runs(
        &self,
        path: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        if !is_analysis_run_collection_path(path) {
            return Err(ApiError::InvalidWirePayload);
        }
        if !body.trim().is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        let consumer = require_headers(headers, self.bound_addr, false)?;
        refuse_metrics_on_collection_payload(body)?;
        let limit =
            parse_collection_page_limit(headers.get("tepp-page-limit").map(String::as_str))?;
        let cursor =
            parse_collection_page_cursor(headers.get("tepp-page-cursor").map(String::as_str))?;
        let mut rows: Vec<&LiveAnalysisRun> = self
            .accepted_runs
            .values()
            .filter(|stored| stored.consumer == consumer)
            .collect();
        rows.sort_by(|left, right| left.accepted.run_id.cmp(&right.accepted.run_id));
        let start = match cursor {
            Some(cursor) => {
                let position = rows
                    .iter()
                    .position(|stored| stored.accepted.run_id == cursor)
                    .ok_or(ApiError::InvalidWirePayload)?;
                position + 1
            }
            None => 0,
        };
        let page = rows.get(start..).unwrap_or(&[]);
        let (visible, remainder) = if page.len() > limit {
            page.split_at(limit)
        } else {
            (page, &[] as &[&LiveAnalysisRun])
        };
        let mut items = Vec::with_capacity(visible.len());
        for stored in visible {
            items.push(AnalysisRunCollectionItem::new(
                stored.accepted.run_id.clone(),
                stored.run_state,
                stored.accepted.idempotency_key.clone(),
            )?);
        }
        let next_cursor = if remainder.is_empty() {
            None
        } else {
            visible.last().map(|stored| stored.accepted.run_id.clone())
        };
        let collection = AnalysisRunCollection::new(items, next_cursor)?;
        let response_body = collection.to_json()?;
        refuse_metrics_on_collection_payload(&response_body)?;
        Ok(json_response(200, "OK", response_body))
    }

    /// Test-only seam that records a non-accepted loopback state.
    ///
    /// Used to prove cancel, collection, retry, and stored-request inspect of
    /// running, succeeded, failed, and cancelled runs without duplicating the
    /// live POST running/terminal lifecycle slice.
    #[cfg(test)]
    fn force_loopback_run_state(
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
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunRequest, ApiError,
        DEFAULT_ANALYSIS_RUN_BYTE_LIMIT, ErrorEnvelope, LINEAGEWEAVE_CONSUMER_CODE,
        NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE, NARUON_LIVE_HEADER_BYTE_LIMIT,
        NARUON_LIVE_HEADER_COUNT_LIMIT, NARUON_LIVE_IO_TIMEOUT, TEMPORAL_CONTEXT_PATH,
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

    fn cancel_http(run_id: &str, body: &str, consumer: &str, idempotency_key: &str) -> String {
        let mut request = format!("POST {NARUON_ANALYSIS_RUN_PATH}/{run_id}/cancel HTTP/1.1\r\n");
        write!(
            request,
            "Host: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        )
        .expect("cancel request");
        request
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn handler_covers_metric_free_cancel_and_terminal_refusal() {
        use crate::{AnalysisRunCancelRequest, AnalysisRunStatus, AnalysisRunStatusState};

        let run = sample_run();
        let mut service = AnalysisRunLiveService::new();
        let accepted =
            service.handle_http_request(&valid_request(&run, NARUON_CONSUMER_CODE, "127.0.0.1"));
        assert_eq!(accepted.status_code, 202);
        let receipt: serde_json::Value =
            serde_json::from_str(&accepted.body).expect("accepted json");
        let run_id = receipt["run_id"].as_str().expect("run_id");
        assert_eq!(run_id, "tepp-run-1");

        let cancel_body = AnalysisRunCancelRequest::new(run_id, run.idempotency_key.as_str())
            .expect("cancel dto")
            .to_json()
            .expect("cancel json");
        let cancelled = service.handle_http_request(&cancel_http(
            run_id,
            &cancel_body,
            NARUON_CONSUMER_CODE,
            run.idempotency_key.as_str(),
        ));
        assert_eq!(cancelled.status_code, 200);
        let status = AnalysisRunStatus::from_json(&cancelled.body).expect("cancelled status");
        assert_eq!(status.run_state, AnalysisRunStatusState::Cancelled);
        assert_eq!(status.terminal_result, None);
        assert!(!cancelled.body.contains("rmse"));
        assert!(!cancelled.body.contains("scientific_acceptance"));

        let replay = service.handle_http_request(&cancel_http(
            run_id,
            &cancel_body,
            NARUON_CONSUMER_CODE,
            run.idempotency_key.as_str(),
        ));
        assert_eq!(replay.status_code, 200);
        assert_eq!(replay.body, cancelled.body);

        let empty_body = service.handle_http_request(&cancel_http(
            run_id,
            "",
            NARUON_CONSUMER_CODE,
            run.idempotency_key.as_str(),
        ));
        assert_eq!(empty_body.status_code, 200);

        let create_replay =
            service.handle_http_request(&valid_request(&run, NARUON_CONSUMER_CODE, "127.0.0.1"));
        assert_eq!(create_replay.status_code, 202);
        assert_eq!(create_replay.body, accepted.body);

        let mut second = run.clone();
        second.idempotency_key = "analysis-live-idem-002".into();
        let running_accepted =
            service.handle_http_request(&valid_request(&second, NARUON_CONSUMER_CODE, "127.0.0.1"));
        let running_id = serde_json::from_str::<serde_json::Value>(&running_accepted.body)
            .expect("running accepted")["run_id"]
            .as_str()
            .expect("id")
            .to_owned();
        service
            .force_loopback_run_state(&running_id, AnalysisRunStatusState::Running)
            .expect("force running");
        let running_cancel = service.handle_http_request(&cancel_http(
            &running_id,
            "",
            NARUON_CONSUMER_CODE,
            second.idempotency_key.as_str(),
        ));
        assert_eq!(running_cancel.status_code, 200);
        assert_eq!(
            AnalysisRunStatus::from_json(&running_cancel.body)
                .expect("running cancelled")
                .run_state,
            AnalysisRunStatusState::Cancelled
        );

        for (state, key_suffix) in [
            (AnalysisRunStatusState::Succeeded, "003"),
            (AnalysisRunStatusState::Failed, "004"),
        ] {
            let mut terminal = run.clone();
            terminal.idempotency_key = format!("analysis-live-idem-{key_suffix}");
            let terminal_accepted = service.handle_http_request(&valid_request(
                &terminal,
                NARUON_CONSUMER_CODE,
                "127.0.0.1",
            ));
            let terminal_id = serde_json::from_str::<serde_json::Value>(&terminal_accepted.body)
                .expect("terminal accepted")["run_id"]
                .as_str()
                .expect("id")
                .to_owned();
            service
                .force_loopback_run_state(&terminal_id, state)
                .expect("force terminal");
            assert_eq!(
                service
                    .handle_http_request(&cancel_http(
                        &terminal_id,
                        "",
                        NARUON_CONSUMER_CODE,
                        terminal.idempotency_key.as_str(),
                    ))
                    .status_code,
                400,
                "state={state:?}"
            );
        }

        assert_eq!(
            service
                .handle_http_request(&cancel_http(
                    "missing-run",
                    "",
                    NARUON_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&cancel_http(
                    run_id,
                    &cancel_body,
                    LINEAGEWEAVE_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&cancel_http(
                    run_id,
                    &cancel_body,
                    NARUON_CONSUMER_CODE,
                    "wrong-key",
                ))
                .status_code,
            400
        );
        let mismatched = AnalysisRunCancelRequest::new("other-run", run.idempotency_key.as_str())
            .expect("mismatch")
            .to_json()
            .expect("mismatch json");
        assert_eq!(
            service
                .handle_http_request(&cancel_http(
                    run_id,
                    &mismatched,
                    NARUON_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
        let key_mismatch = AnalysisRunCancelRequest::new(run_id, "wrong-key")
            .expect("key mismatch")
            .to_json()
            .expect("key json");
        assert_eq!(
            service
                .handle_http_request(&cancel_http(
                    run_id,
                    &key_mismatch,
                    NARUON_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
        let metric_body = r#"{"contract_version":1,"run_id":"tepp-run-1","idempotency_key":"analysis-live-idem-001","rmse":0.1}"#;
        assert_eq!(
            service
                .handle_http_request(&cancel_http(
                    run_id,
                    metric_body,
                    NARUON_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH}/{run_id}/cancel HTTP/1.1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "POST {NARUON_ANALYSIS_RUN_PATH}/{run_id} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: 0\r\n\r\n",
                    run.idempotency_key
                ))
                .status_code,
            400
        );
        let oversized = "a".repeat(129);
        assert_eq!(
            service
                .handle_http_request(&cancel_http(
                    &oversized,
                    "",
                    NARUON_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            413
        );
        assert_eq!(
            service
                .force_loopback_run_state("missing", AnalysisRunStatusState::Running)
                .expect_err("unknown force"),
            ApiError::InvalidWirePayload
        );
        service
            .runs_by_id
            .insert("dangling".into(), "missing-replay".into());
        assert_eq!(
            service
                .force_loopback_run_state("dangling", AnalysisRunStatusState::Running)
                .expect_err("dangling force"),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            service
                .handle_http_request(&cancel_http(
                    "dangling",
                    "",
                    NARUON_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
    }

    fn collection_http(consumer: &str, extra: &[(&str, &str)]) -> String {
        let mut request = format!("GET {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\n");
        write!(
            request,
            "Host: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\n"
        )
        .expect("collection headers");
        for (name, value) in extra {
            write!(request, "{name}: {value}\r\n").expect("extra header");
        }
        request.push_str("content-length: 0\r\n\r\n");
        request
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn handler_covers_metric_free_collection_get() {
        use crate::{AnalysisRunCollection, AnalysisRunStatusState};

        let run = sample_run();
        let mut service = AnalysisRunLiveService::new();
        let empty = service.handle_http_request(&collection_http(NARUON_CONSUMER_CODE, &[]));
        assert_eq!(empty.status_code, 200);
        let empty_page = AnalysisRunCollection::from_json(&empty.body).expect("empty");
        assert!(empty_page.runs.is_empty());
        assert_eq!(empty_page.next_cursor, None);
        assert!(!empty.body.contains("scientific_acceptance"));
        assert!(!empty.body.contains("rmse"));
        assert!(!empty.body.contains("terminal_result"));

        let accepted =
            service.handle_http_request(&valid_request(&run, NARUON_CONSUMER_CODE, "127.0.0.1"));
        assert_eq!(accepted.status_code, 202);
        let mut second = run.clone();
        second.idempotency_key = "analysis-live-idem-002".into();
        assert_eq!(
            service
                .handle_http_request(&valid_request(&second, NARUON_CONSUMER_CODE, "127.0.0.1"))
                .status_code,
            202
        );
        let mut third = run.clone();
        third.idempotency_key = "analysis-live-idem-003".into();
        assert_eq!(
            service
                .handle_http_request(&valid_request(&third, NARUON_CONSUMER_CODE, "127.0.0.1"))
                .status_code,
            202
        );
        service
            .force_loopback_run_state("tepp-run-2", AnalysisRunStatusState::Running)
            .expect("running");
        service
            .force_loopback_run_state("tepp-run-3", AnalysisRunStatusState::Cancelled)
            .expect("cancelled");
        let mut failed = run.clone();
        failed.idempotency_key = "analysis-live-idem-004".into();
        assert_eq!(
            service
                .handle_http_request(&valid_request(&failed, NARUON_CONSUMER_CODE, "127.0.0.1"))
                .status_code,
            202
        );
        service
            .force_loopback_run_state("tepp-run-4", AnalysisRunStatusState::Failed)
            .expect("failed");
        let mut succeeded = run.clone();
        succeeded.idempotency_key = "analysis-live-idem-005".into();
        assert_eq!(
            service
                .handle_http_request(&valid_request(
                    &succeeded,
                    NARUON_CONSUMER_CODE,
                    "127.0.0.1"
                ))
                .status_code,
            202
        );
        service
            .force_loopback_run_state("tepp-run-5", AnalysisRunStatusState::Succeeded)
            .expect("succeeded");
        assert_eq!(
            service
                .handle_http_request(&valid_request(
                    &run,
                    LINEAGEWEAVE_CONSUMER_CODE,
                    "127.0.0.1"
                ))
                .status_code,
            202
        );

        let listed = service.handle_http_request(&collection_http(NARUON_CONSUMER_CODE, &[]));
        assert_eq!(listed.status_code, 200);
        let page = AnalysisRunCollection::from_json(&listed.body).expect("page");
        assert_eq!(page.runs.len(), 5);
        assert_eq!(page.next_cursor, None);
        assert_eq!(page.runs[0].run_id, "tepp-run-1");
        assert_eq!(page.runs[0].run_state, AnalysisRunStatusState::Accepted);
        assert_eq!(page.runs[1].run_state, AnalysisRunStatusState::Running);
        assert_eq!(page.runs[2].run_state, AnalysisRunStatusState::Cancelled);
        assert_eq!(page.runs[3].run_state, AnalysisRunStatusState::Failed);
        assert_eq!(page.runs[4].run_state, AnalysisRunStatusState::Succeeded);
        assert!(!listed.body.contains("scientific_acceptance"));
        assert!(!listed.body.contains("rmse"));
        assert!(!listed.body.contains("terminal_result"));

        let lineage =
            service.handle_http_request(&collection_http(LINEAGEWEAVE_CONSUMER_CODE, &[]));
        let lineage_page = AnalysisRunCollection::from_json(&lineage.body).expect("lineage");
        assert_eq!(lineage_page.runs.len(), 1);
        assert_eq!(lineage_page.runs[0].run_id, "tepp-run-6");

        let first = service.handle_http_request(&collection_http(
            NARUON_CONSUMER_CODE,
            &[("tepp-page-limit", "2")],
        ));
        let first_page = AnalysisRunCollection::from_json(&first.body).expect("first");
        assert_eq!(first_page.runs.len(), 2);
        assert_eq!(first_page.next_cursor.as_deref(), Some("tepp-run-2"));
        let second_page_http = service.handle_http_request(&collection_http(
            NARUON_CONSUMER_CODE,
            &[("tepp-page-cursor", "tepp-run-2"), ("tepp-page-limit", "2")],
        ));
        let second_page = AnalysisRunCollection::from_json(&second_page_http.body).expect("second");
        assert_eq!(second_page.runs.len(), 2);
        assert_eq!(second_page.runs[0].run_id, "tepp-run-3");
        assert_eq!(second_page.next_cursor.as_deref(), Some("tepp-run-4"));
        let last_page = AnalysisRunCollection::from_json(
            &service
                .handle_http_request(&collection_http(
                    NARUON_CONSUMER_CODE,
                    &[("tepp-page-cursor", "tepp-run-4"), ("tepp-page-limit", "2")],
                ))
                .body,
        )
        .expect("last");
        assert_eq!(last_page.runs.len(), 1);
        assert_eq!(last_page.runs[0].run_id, "tepp-run-5");
        assert_eq!(last_page.next_cursor, None);

        assert_eq!(
            service
                .handle_http_request(&collection_http(
                    NARUON_CONSUMER_CODE,
                    &[("tepp-page-cursor", "missing")],
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&collection_http(
                    NARUON_CONSUMER_CODE,
                    &[("tepp-page-limit", "0")],
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&collection_http(
                    NARUON_CONSUMER_CODE,
                    &[("tepp-page-limit", "65")],
                ))
                .status_code,
            413
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH}/tepp-run-1 HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 2\r\n\r\n{{}}"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH}?cursor=tepp-run-1 HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
    }

    fn retry_http(run_id: &str, body: &str, consumer: &str, idempotency_key: &str) -> String {
        let mut request = format!("POST {NARUON_ANALYSIS_RUN_PATH}/{run_id}/retry HTTP/1.1\r\n");
        write!(
            request,
            "Host: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        )
        .expect("retry request");
        request
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn handler_covers_metric_free_retry_of_failed_and_cancelled() {
        use crate::{
            AnalysisRunAccepted, AnalysisRunCollection, AnalysisRunRetryRequest,
            AnalysisRunStatusState,
        };

        let run = sample_run();
        let mut service = AnalysisRunLiveService::new();
        let accepted =
            service.handle_http_request(&valid_request(&run, NARUON_CONSUMER_CODE, "127.0.0.1"));
        assert_eq!(accepted.status_code, 202);
        let parent_id = serde_json::from_str::<serde_json::Value>(&accepted.body)
            .expect("accepted json")["run_id"]
            .as_str()
            .expect("run_id")
            .to_owned();
        assert_eq!(parent_id, "tepp-run-1");
        service
            .force_loopback_run_state(&parent_id, AnalysisRunStatusState::Failed)
            .expect("force failed");

        let retry_key = "analysis-live-retry-001";
        let retry_body = AnalysisRunRetryRequest::new(&parent_id, retry_key)
            .expect("retry dto")
            .to_json()
            .expect("retry json");
        let retried = service.handle_http_request(&retry_http(
            &parent_id,
            &retry_body,
            NARUON_CONSUMER_CODE,
            retry_key,
        ));
        assert_eq!(retried.status_code, 202);
        let child = AnalysisRunAccepted::from_json(&retried.body).expect("child");
        assert_eq!(child.run_id, "tepp-run-2");
        assert_eq!(child.run_state, "accepted");
        assert_eq!(child.idempotency_key, retry_key);
        assert!(!retried.body.contains("rmse"));
        assert!(!retried.body.contains("scientific_acceptance"));
        assert_ne!(child.run_id, parent_id);

        let replay = service.handle_http_request(&retry_http(
            &parent_id,
            &retry_body,
            NARUON_CONSUMER_CODE,
            retry_key,
        ));
        assert_eq!(replay.status_code, 202);
        assert_eq!(replay.body, retried.body);

        let empty_body = service.handle_http_request(&retry_http(
            &parent_id,
            "",
            NARUON_CONSUMER_CODE,
            retry_key,
        ));
        assert_eq!(empty_body.status_code, 202);
        assert_eq!(empty_body.body, retried.body);

        let mut cancelled_run = run.clone();
        cancelled_run.idempotency_key = "analysis-live-idem-002".into();
        let cancelled_accepted = service.handle_http_request(&valid_request(
            &cancelled_run,
            NARUON_CONSUMER_CODE,
            "127.0.0.1",
        ));
        let cancelled_id = serde_json::from_str::<serde_json::Value>(&cancelled_accepted.body)
            .expect("cancelled accepted")["run_id"]
            .as_str()
            .expect("id")
            .to_owned();
        service
            .force_loopback_run_state(&cancelled_id, AnalysisRunStatusState::Cancelled)
            .expect("force cancelled");
        let cancelled_retry = service.handle_http_request(&retry_http(
            &cancelled_id,
            "",
            NARUON_CONSUMER_CODE,
            "analysis-live-retry-002",
        ));
        assert_eq!(cancelled_retry.status_code, 202);
        let cancelled_child =
            AnalysisRunAccepted::from_json(&cancelled_retry.body).expect("cancelled child");
        assert_eq!(cancelled_child.run_id, "tepp-run-4");
        assert_eq!(cancelled_child.idempotency_key, "analysis-live-retry-002");

        for (state, key_suffix) in [
            (AnalysisRunStatusState::Accepted, "003"),
            (AnalysisRunStatusState::Running, "004"),
            (AnalysisRunStatusState::Succeeded, "005"),
        ] {
            let mut blocked = run.clone();
            blocked.idempotency_key = format!("analysis-live-idem-{key_suffix}");
            let blocked_accepted = service.handle_http_request(&valid_request(
                &blocked,
                NARUON_CONSUMER_CODE,
                "127.0.0.1",
            ));
            let blocked_id = serde_json::from_str::<serde_json::Value>(&blocked_accepted.body)
                .expect("blocked accepted")["run_id"]
                .as_str()
                .expect("id")
                .to_owned();
            service
                .force_loopback_run_state(&blocked_id, state)
                .expect("force blocked");
            assert_eq!(
                service
                    .handle_http_request(&retry_http(
                        &blocked_id,
                        "",
                        NARUON_CONSUMER_CODE,
                        &format!("analysis-live-retry-{key_suffix}"),
                    ))
                    .status_code,
                400,
                "state={state:?}"
            );
        }

        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    "missing-run",
                    "",
                    NARUON_CONSUMER_CODE,
                    retry_key,
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    &parent_id,
                    &retry_body,
                    LINEAGEWEAVE_CONSUMER_CODE,
                    retry_key,
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    &parent_id,
                    "",
                    NARUON_CONSUMER_CODE,
                    run.idempotency_key.as_str(),
                ))
                .status_code,
            400
        );
        let mismatched = AnalysisRunRetryRequest::new("other-run", retry_key)
            .expect("mismatch")
            .to_json()
            .expect("mismatch json");
        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    &parent_id,
                    &mismatched,
                    NARUON_CONSUMER_CODE,
                    retry_key,
                ))
                .status_code,
            400
        );
        let key_mismatch = AnalysisRunRetryRequest::new(&parent_id, "wrong-retry-key")
            .expect("key mismatch")
            .to_json()
            .expect("key json");
        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    &parent_id,
                    &key_mismatch,
                    NARUON_CONSUMER_CODE,
                    retry_key,
                ))
                .status_code,
            400
        );
        let metric_body = r#"{"contract_version":1,"run_id":"tepp-run-1","idempotency_key":"analysis-live-retry-001","rmse":0.1}"#;
        assert_eq!(
            service
                .handle_http_request(&retry_http(
                    &parent_id,
                    metric_body,
                    NARUON_CONSUMER_CODE,
                    retry_key,
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH}/{parent_id} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "POST {NARUON_ANALYSIS_RUN_PATH}/{parent_id}/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {retry_key}\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
        let oversized = "a".repeat(129);
        assert_eq!(
            service
                .handle_http_request(&retry_http(&oversized, "", NARUON_CONSUMER_CODE, retry_key,))
                .status_code,
            413
        );

        let listed = service.handle_http_request(&collection_http(NARUON_CONSUMER_CODE, &[]));
        assert_eq!(listed.status_code, 200);
        let page = AnalysisRunCollection::from_json(&listed.body).expect("page");
        let parent_row = page
            .runs
            .iter()
            .find(|row| row.run_id == parent_id)
            .expect("parent row");
        assert_eq!(parent_row.run_state, AnalysisRunStatusState::Failed);
        let child_row = page
            .runs
            .iter()
            .find(|row| row.run_id == child.run_id)
            .expect("child row");
        assert_eq!(child_row.run_state, AnalysisRunStatusState::Accepted);
        assert_eq!(child_row.idempotency_key, retry_key);
        assert!(!listed.body.contains("scientific_acceptance"));
        assert!(!listed.body.contains("rmse"));
    }

    fn stored_request_http(run_id: &str, consumer: &str, extra: &[(&str, &str)]) -> String {
        let mut request = format!("GET {NARUON_ANALYSIS_RUN_PATH}/{run_id}/request HTTP/1.1\r\n");
        write!(
            request,
            "Host: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\n"
        )
        .expect("stored-request headers");
        for (name, value) in extra {
            write!(request, "{name}: {value}\r\n").expect("extra header");
        }
        request.push_str("content-length: 0\r\n\r\n");
        request
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn handler_covers_metric_free_stored_request_get() {
        use crate::{AnalysisRunStatusState, AnalysisRunStoredRequest};

        let run = sample_run();
        let mut service = AnalysisRunLiveService::new();
        let accepted =
            service.handle_http_request(&valid_request(&run, NARUON_CONSUMER_CODE, "127.0.0.1"));
        assert_eq!(accepted.status_code, 202);
        let run_id = serde_json::from_str::<serde_json::Value>(&accepted.body)
            .expect("accepted json")["run_id"]
            .as_str()
            .expect("run_id")
            .to_owned();

        let inspected =
            service.handle_http_request(&stored_request_http(&run_id, NARUON_CONSUMER_CODE, &[]));
        assert_eq!(inspected.status_code, 200);
        let stored = AnalysisRunStoredRequest::from_json(&inspected.body).expect("stored");
        assert_eq!(stored.run_id, run_id);
        assert_eq!(stored.run_state, AnalysisRunStatusState::Accepted);
        assert_eq!(stored.idempotency_key, run.idempotency_key);
        assert_eq!(stored.snapshot_id, run.snapshot_id);
        assert_eq!(stored.knowledge_cutoff, run.knowledge_cutoff);
        assert_eq!(stored.model_contract_version, run.model_contract_version);
        assert_eq!(stored.output_profile, run.output_profile);
        assert!(!inspected.body.contains("rmse"));
        assert!(!inspected.body.contains("scientific_acceptance"));
        assert!(!inspected.body.contains("terminal_result"));
        assert!(!inspected.body.contains("tenant_workspace_id"));

        service
            .force_loopback_run_state(&run_id, AnalysisRunStatusState::Failed)
            .expect("force failed");
        let failed =
            service.handle_http_request(&stored_request_http(&run_id, NARUON_CONSUMER_CODE, &[]));
        assert_eq!(failed.status_code, 200);
        assert_eq!(
            AnalysisRunStoredRequest::from_json(&failed.body)
                .expect("failed stored")
                .run_state,
            AnalysisRunStatusState::Failed
        );

        let mut cancelled_run = run.clone();
        cancelled_run.idempotency_key = "analysis-live-idem-002".into();
        let cancelled_accepted = service.handle_http_request(&valid_request(
            &cancelled_run,
            NARUON_CONSUMER_CODE,
            "127.0.0.1",
        ));
        let cancelled_id = serde_json::from_str::<serde_json::Value>(&cancelled_accepted.body)
            .expect("cancelled accepted")["run_id"]
            .as_str()
            .expect("id")
            .to_owned();
        service
            .force_loopback_run_state(&cancelled_id, AnalysisRunStatusState::Cancelled)
            .expect("force cancelled");
        let cancelled = service.handle_http_request(&stored_request_http(
            &cancelled_id,
            NARUON_CONSUMER_CODE,
            &[],
        ));
        assert_eq!(cancelled.status_code, 200);
        let cancelled_stored =
            AnalysisRunStoredRequest::from_json(&cancelled.body).expect("cancelled stored");
        assert_eq!(
            cancelled_stored.run_state,
            AnalysisRunStatusState::Cancelled
        );
        assert_eq!(cancelled_stored.snapshot_id, run.snapshot_id);
        assert_eq!(cancelled_stored.output_profile, run.output_profile);

        assert_eq!(
            service
                .handle_http_request(&stored_request_http(
                    &run_id,
                    LINEAGEWEAVE_CONSUMER_CODE,
                    &[],
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&stored_request_http(
                    "missing-run",
                    NARUON_CONSUMER_CODE,
                    &[],
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH}/{run_id} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "POST {NARUON_ANALYSIS_RUN_PATH}/{run_id}/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
                ))
                .status_code,
            400
        );
        assert_eq!(
            service
                .handle_http_request(&format!(
                    "GET {NARUON_ANALYSIS_RUN_PATH}/{run_id}/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 2\r\n\r\n{{}}"
                ))
                .status_code,
            400
        );
        let oversized = "a".repeat(129);
        assert_eq!(
            service
                .handle_http_request(&stored_request_http(&oversized, NARUON_CONSUMER_CODE, &[],))
                .status_code,
            413
        );
        let listed = service.handle_http_request(&collection_http(NARUON_CONSUMER_CODE, &[]));
        assert_eq!(listed.status_code, 200);
        assert!(!listed.body.contains("snapshot_id"));
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
