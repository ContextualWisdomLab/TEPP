//! Execute scientific-acceptance analysis on the loopback lifecycle path.
//!
//! GAP-003A engine-on-loopback slice: `POST /v1/analysis-runs/{run_id}/execute`
//! runs [`submit_validation_run`] and [`complete_validation_run`] against an
//! already accepted loopback run and records running then terminal status with
//! the produced `tepp.scientific_acceptance.v1` bytes. The execute body carries
//! corpus, recovery vectors, seed, and the pre-registered SE-gate multiplier.
//! It must not carry `scientific_acceptance_json`. GET then returns the artifact
//! without a caller-supplied terminal payload. Persistence remains GAP-003B.

use crate::{
    AnalysisCorpus, AnalysisEngineError, AnalysisEvidenceUnit, RecoveryObservation,
    SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE, SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION,
    VALIDATION_CPU_F64_MODEL, complete_validation_run, submit_validation_run,
};
use serde::Deserialize;
use std::net::SocketAddr;
use temporal_core::{AvailableTime, EventTime};
use tepp_api::{
    ANALYSIS_RUN_STATUS_PATH, AnalysisResultSummary, AnalysisRunLiveService, AnalysisRunStatus,
    AnalysisRunStatusState, AnalysisRunTerminalResult, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
    DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, ErrorEnvelope, NaruonLiveResponse,
    SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
    analysis_run_execute_path_run_id, parse_loopback_http_parts,
};

/// Result-metric keys that must not appear on an execute request object.
const EXECUTE_FORBIDDEN_RESULT_KEYS: [&str; 11] = [
    "rmse",
    "rmse_standard_error",
    "mean_bias",
    "bias_standard_error",
    "interval_coverage",
    "coverage_wilson_lower",
    "coverage_wilson_upper",
    "temporal_order_accuracy",
    "se_gate_accepted",
    "scientific_acceptance",
    "report",
];

/// Contract version for the loopback execute body.
pub const ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION: u16 = 1;
/// Loopback path suffix owned by this engine glue.
pub const ANALYSIS_RUN_EXECUTE_PATH_SUFFIX: &str = "execute";

/// Loopback analysis-run service that can execute scientific acceptance.
///
/// Create, GET, running, and terminal stay on [`tepp_api::AnalysisRunLiveService`].
/// This wrapper intercepts `POST /v1/analysis-runs/{run_id}/execute` so the
/// engine produces the artifact. `tepp_api` cannot depend on this crate.
#[derive(Debug)]
pub struct ScientificAcceptanceLoopbackService {
    live: AnalysisRunLiveService,
    next_request_serial: u64,
}

impl Default for ScientificAcceptanceLoopbackService {
    fn default() -> Self {
        Self::new()
    }
}

impl ScientificAcceptanceLoopbackService {
    /// Construct an in-memory handler with no bound socket.
    #[must_use]
    pub fn new() -> Self {
        Self {
            live: AnalysisRunLiveService::new(),
            next_request_serial: 1,
        }
    }

    /// Wrap an existing loopback listener.
    #[must_use]
    pub fn from_live(live: AnalysisRunLiveService) -> Self {
        Self {
            live,
            next_request_serial: 1,
        }
    }

    /// Bind a caller-supplied loopback address.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback address
    /// and [`ApiError::InvalidWirePayload`] when the socket cannot be opened.
    pub fn bind(addr: SocketAddr) -> Result<Self, ApiError> {
        Ok(Self::from_live(AnalysisRunLiveService::bind(addr)?))
    }

    /// Bind an ephemeral IPv4 loopback port.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when the operating system
    /// refuses the loopback bind.
    pub fn bind_loopback() -> Result<Self, ApiError> {
        Ok(Self::from_live(AnalysisRunLiveService::bind_loopback()?))
    }

    /// Return the bound loopback address.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when no socket is bound.
    pub fn local_addr(&self) -> Result<SocketAddr, ApiError> {
        self.live.local_addr()
    }

    /// Accept and serve one HTTP/1.1 request, including `/execute`.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed API error when no socket is bound or socket I/O
    /// fails. Protocol errors are returned as redacted HTTP responses.
    pub fn serve_one(&mut self) -> Result<NaruonLiveResponse, ApiError> {
        let (mut stream, request) = self.live.accept_loopback_request()?;
        let response = match request {
            Ok(request) => self.handle_http_request(&request),
            Err(error) => self.response_from_error(error),
        };
        AnalysisRunLiveService::write_loopback_response(&mut stream, &response)?;
        Ok(response)
    }

    /// Return the inner loopback service.
    #[must_use]
    pub fn live(&self) -> &AnalysisRunLiveService {
        &self.live
    }

    /// Return the inner loopback service mutably.
    pub fn live_mut(&mut self) -> &mut AnalysisRunLiveService {
        &mut self.live
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
        let Ok(parts) = parse_loopback_http_parts(request, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)
        else {
            return Ok(self.live.handle_http_request(request));
        };
        if parts.method == "POST" && is_execute_attempt(parts.path) {
            let run_id = analysis_run_execute_path_run_id(parts.path)?;
            return self.execute_scientific_acceptance(&run_id, &parts.headers, parts.body);
        }
        Ok(self.live.handle_http_request(request))
    }

    fn execute_scientific_acceptance(
        &mut self,
        path_run_id: &str,
        headers: &std::collections::HashMap<String, String>,
        body: &str,
    ) -> Result<NaruonLiveResponse, ApiError> {
        refuse_result_metrics_on_execute(body)?;
        let consumer = self.live.authorize_loopback_headers(headers)?;
        let idempotency_key = header_required(headers, "idempotency-key")?;
        let execute = parse_execute_body(body)?;
        if execute.run_id != path_run_id || execute.idempotency_key != idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        let stored = self.live.loopback_run(path_run_id)?;
        if stored.consumer != consumer || stored.accepted.idempotency_key != idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        if stored.status.run_state != AnalysisRunStatusState::Accepted {
            return Err(ApiError::InvalidWirePayload);
        }
        if stored.request.output_profile != SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE
            || stored.request.output_profile != SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE
            || stored.request.model_contract_version != VALIDATION_CPU_F64_MODEL
        {
            return Err(ApiError::InvalidWirePayload);
        }
        let corpus = execute.corpus.to_corpus().map_err(map_engine_error)?;
        let receipt =
            submit_validation_run(&stored.request, &corpus, execute.seed, execute.se_gate_k)
                .map_err(map_engine_error)?;
        let observation = RecoveryObservation::new(
            &receipt,
            execute.study_label.clone(),
            execute.truth.clone(),
            execute.recovered.clone(),
            execute.interval_lower.clone(),
            execute.interval_upper.clone(),
            execute.truth_times.clone(),
            execute.recovered_times.clone(),
            execute.se_gate_k,
            execute.authored_by_llm,
        )
        .map_err(map_engine_error)?;
        let evidence = complete_validation_run(&receipt, &stored.request, &corpus, &observation)
            .map_err(map_engine_error)?;
        let artifact_json = evidence.to_json().map_err(map_engine_error)?;
        let digest = evidence.sha256().map_err(map_engine_error)?;
        if evidence.schema_version() != SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION
            || evidence.schema_version() != SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA
            || evidence.output_profile() != SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE
        {
            return Err(ApiError::InvalidWirePayload);
        }
        let summary = AnalysisResultSummary::new(
            "scientific_acceptance",
            evidence.eligible_evidence_count(),
            4,
            "validated",
        )?;
        let terminal = AnalysisRunTerminalResult::succeeded(
            &stored.request,
            &stored.accepted,
            evidence.run_id(),
            digest,
            SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
            execute.completed_at.clone(),
            summary,
        )?;
        let running = AnalysisRunStatus::running(&stored.accepted)?;
        let terminal_status =
            AnalysisRunStatus::terminal(&stored.request, &stored.accepted, terminal)?;
        self.live
            .record_loopback_status(path_run_id, running, None)?;
        self.live
            .record_loopback_status(path_run_id, terminal_status, Some(artifact_json))?;
        let response_body = self.live.loopback_status_json(path_run_id)?;
        Ok(NaruonLiveResponse {
            status_code: 200,
            reason_phrase: "OK",
            body: response_body,
        })
    }

    fn response_from_error(&mut self, error: ApiError) -> NaruonLiveResponse {
        let request_id = format!("scientific-acceptance-execute-{}", self.next_request_serial);
        self.next_request_serial += 1;
        let (status_code, reason_phrase) = match error {
            ApiError::AuthorizationDenied => (403, "Forbidden"),
            ApiError::LimitExceeded => (413, "Payload Too Large"),
            ApiError::UnsupportedContractVersion => (422, "Unprocessable Entity"),
            _ => (400, "Bad Request"),
        };
        let body = ErrorEnvelope::from_api_error(error, request_id)
            .and_then(|envelope| envelope.to_json())
            .unwrap_or_else(|_| {
                "{\"error_code\":\"invalid_wire_payload\",\"message\":\"invalid API wire payload\",\"request_id\":\"scientific-acceptance-execute-fallback\",\"retryable\":false}".to_owned()
            });
        NaruonLiveResponse {
            status_code,
            reason_phrase,
            body,
        }
    }
}

/// Supported execute-body contract version.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ScientificAcceptanceExecuteRequest {
    contract_version: u16,
    run_id: String,
    idempotency_key: String,
    seed: u64,
    se_gate_k: f64,
    completed_at: String,
    study_label: String,
    authored_by_llm: bool,
    corpus: ExecuteCorpus,
    truth: Vec<f64>,
    recovered: Vec<f64>,
    interval_lower: Vec<f64>,
    interval_upper: Vec<f64>,
    truth_times: Vec<f64>,
    recovered_times: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ExecuteCorpus {
    snapshot_id: String,
    evidence_units: Vec<ExecuteEvidenceUnit>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ExecuteEvidenceUnit {
    evidence_id: String,
    event_time: String,
    available_time: String,
    membership_count: u32,
}

impl ExecuteCorpus {
    fn to_corpus(&self) -> Result<AnalysisCorpus, AnalysisEngineError> {
        let mut units = Vec::with_capacity(self.evidence_units.len());
        for unit in &self.evidence_units {
            let event_time = EventTime::parse_rfc3339(&unit.event_time)
                .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
            let available_time = AvailableTime::parse_rfc3339(&unit.available_time)
                .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
            units.push(AnalysisEvidenceUnit::new(
                unit.evidence_id.clone(),
                event_time,
                available_time,
                unit.membership_count,
            )?);
        }
        AnalysisCorpus::new(self.snapshot_id.clone(), units)
    }
}

fn parse_execute_body(body: &str) -> Result<ScientificAcceptanceExecuteRequest, ApiError> {
    if body.len() > DEFAULT_ANALYSIS_RUN_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
    refuse_result_metrics_on_execute(body)?;
    let execute: ScientificAcceptanceExecuteRequest =
        serde_json::from_str(body).map_err(|_| ApiError::InvalidWirePayload)?;
    if execute.contract_version != ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION {
        return Err(ApiError::UnsupportedContractVersion);
    }
    if execute.run_id.is_empty()
        || execute.idempotency_key.is_empty()
        || execute.completed_at.is_empty()
        || execute.study_label.is_empty()
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(execute)
}

fn refuse_result_metrics_on_execute(payload: &str) -> Result<(), ApiError> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) else {
        return Ok(());
    };
    let Some(object) = value.as_object() else {
        return Err(ApiError::InvalidWirePayload);
    };
    if EXECUTE_FORBIDDEN_RESULT_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

fn is_execute_attempt(path: &str) -> bool {
    path.rsplit('/').next() == Some(ANALYSIS_RUN_EXECUTE_PATH_SUFFIX)
        && path.starts_with(ANALYSIS_RUN_STATUS_PATH)
}

fn header_required<'a>(
    headers: &'a std::collections::HashMap<String, String>,
    name: &str,
) -> Result<&'a str, ApiError> {
    let value = headers.get(name).ok_or(ApiError::InvalidWirePayload)?;
    if value.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(value.as_str())
}

fn map_engine_error(error: AnalysisEngineError) -> ApiError {
    match error {
        AnalysisEngineError::Api(api) => api,
        AnalysisEngineError::LimitExceeded => ApiError::LimitExceeded,
        _ => ApiError::InvalidWirePayload,
    }
}
