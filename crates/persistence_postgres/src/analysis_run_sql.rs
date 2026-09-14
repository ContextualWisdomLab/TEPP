//! SQL contracts for durable, idempotent analysis-run receipts and state events.

use crate::{ModelArtifactRecord, PersistenceError};
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, SystemTime};
use tepp_api::{
    AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunStatus, AnalysisRunStatusState,
    AnalysisRunTerminalResult, AnalysisRunTerminalState, require_status_binding,
};
use uuid::Uuid;

/// An immutable, canonical analysis-run submission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunRequestRecord {
    /// Server-assigned run identity.
    pub analysis_run_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Validated request contract.
    pub request: AnalysisRunRequest,
    /// Canonical request JSON.
    pub request_payload: String,
    /// Lowercase SHA-256 of `request_payload`.
    pub request_payload_sha256: String,
    /// System time at durable acceptance.
    pub system_time: SystemTime,
    /// Availability time of the receipt.
    pub available_time: AvailableTime,
}

/// Fully validated durable request and its latest lifecycle status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunWorkerSnapshot {
    /// Canonical persisted request reconstructed from its stored payload.
    pub request_record: AnalysisRunRequestRecord,
    /// Latest validated lifecycle status for the request.
    pub status: AnalysisRunStatus,
}

/// Build the content-addressed model-artifact row referenced by a succeeded result.
///
/// The caller persists this row before appending the succeeded lifecycle event.
/// The database then enforces both tenant ownership and exact digest equality.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidAnalysisRun`] unless `result` is a valid
/// succeeded result with a UUID artifact identity, digest, and schema version.
pub fn model_artifact_from_analysis_result(
    tenant_record_id: Uuid,
    model_run_id: Uuid,
    result: &AnalysisRunTerminalResult,
    protected_object_ref: Option<String>,
    system_time: SystemTime,
    available_time: AvailableTime,
) -> Result<ModelArtifactRecord, PersistenceError> {
    let (
        AnalysisRunTerminalState::Succeeded,
        Some(result_artifact_id),
        Some(result_sha256),
        Some(result_schema_version),
    ) = (
        result.run_state,
        result.result_artifact_id.as_deref(),
        result.result_sha256.as_deref(),
        result.result_schema_version.as_deref(),
    )
    else {
        return Err(PersistenceError::InvalidAnalysisRun);
    };
    result
        .to_json()
        .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
    let record = ModelArtifactRecord {
        model_artifact_id: Uuid::parse_str(result_artifact_id)
            .map_err(|_| PersistenceError::InvalidAnalysisRun)?,
        tenant_record_id,
        model_run_id,
        artifact_type_code: result_schema_version.to_owned(),
        artifact_content_digest: result_sha256.to_owned(),
        protected_object_ref,
        system_time,
        available_time,
    };
    record.validate()?;
    Ok(record)
}

impl AnalysisRunRequestRecord {
    /// Validate and canonicalize a public request for persistence.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidAnalysisRun`] for an invalid request.
    pub fn from_request(
        tenant_record_id: Uuid,
        request: &AnalysisRunRequest,
        system_time: SystemTime,
        available_time: AvailableTime,
    ) -> Result<Self, PersistenceError> {
        let request_payload = request
            .to_json()
            .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
        let request_payload_sha256 = format!("{:x}", Sha256::digest(request_payload.as_bytes()));
        let analysis_run_id = derived_uuid(tenant_record_id, "run", &request_payload);
        Ok(Self {
            analysis_run_id,
            tenant_record_id,
            request: request.clone(),
            request_payload,
            request_payload_sha256,
            system_time,
            available_time,
        })
    }

    /// Reconstruct the stable accepted receipt for this durable request.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed API-shape error mapped to persistence authority.
    pub fn accepted(&self) -> Result<AnalysisRunAccepted, PersistenceError> {
        AnalysisRunAccepted::new(
            self.analysis_run_id.to_string(),
            "accepted",
            self.request.idempotency_key.clone(),
        )
        .map_err(|_| PersistenceError::InvalidAnalysisRun)
    }
}

/// Closed lifecycle vocabulary for persisted run events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRunState {
    /// Durably accepted.
    Accepted,
    /// Execution started.
    Running,
    /// Completed with a bound artifact.
    Succeeded,
    /// Completed without an artifact.
    Failed,
}

impl AnalysisRunState {
    fn code(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

/// One immutable analysis-run lifecycle event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunStateEventRecord {
    /// Event identity.
    pub analysis_run_state_event_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Owning analysis run.
    pub analysis_run_id: Uuid,
    /// Monotonic one-based lifecycle sequence.
    pub state_sequence: u64,
    /// Lifecycle state.
    pub run_state: AnalysisRunState,
    /// Validated terminal status, absent for nonterminal states.
    pub terminal_status: Option<AnalysisRunStatus>,
    /// System time of this event.
    pub system_time: SystemTime,
    /// Availability time of this event.
    pub available_time: AvailableTime,
}

impl AnalysisRunStateEventRecord {
    fn validate(&self) -> Result<(), PersistenceError> {
        if self.state_sequence == 0 || self.state_sequence > i64::MAX as u64 {
            return Err(PersistenceError::InvalidAnalysisRun);
        }
        match (self.run_state, &self.terminal_status) {
            (AnalysisRunState::Accepted | AnalysisRunState::Running, None) => Ok(()),
            (AnalysisRunState::Succeeded | AnalysisRunState::Failed, Some(status)) => {
                status
                    .to_json()
                    .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
                let state_matches = matches!(
                    (self.run_state, status.run_state),
                    (
                        AnalysisRunState::Succeeded,
                        AnalysisRunStatusState::Succeeded
                    ) | (AnalysisRunState::Failed, AnalysisRunStatusState::Failed)
                );
                if state_matches {
                    Ok(())
                } else {
                    Err(PersistenceError::InvalidAnalysisRun)
                }
            }
            _ => Err(PersistenceError::InvalidAnalysisRun),
        }
    }
}

/// Render an atomic idempotent insert for a canonical request.
///
/// # Errors
///
/// Returns a fail-closed validation error before SQL generation.
pub fn insert_analysis_run_request_sql(
    record: &AnalysisRunRequestRecord,
) -> Result<String, PersistenceError> {
    let expected = AnalysisRunRequestRecord::from_request(
        record.tenant_record_id,
        &record.request,
        record.system_time,
        record.available_time,
    )?;
    if expected.analysis_run_id != record.analysis_run_id
        || expected.request_payload != record.request_payload
        || expected.request_payload_sha256 != record.request_payload_sha256
        || record.request_payload.contains("$tepp$")
    {
        return Err(PersistenceError::InvalidAnalysisRun);
    }
    Ok(format!(
        "DO $tepp$ DECLARE stored_run_id uuid; stored_digest text; stored_payload text; inserted_count bigint; BEGIN INSERT INTO analysis_run_request (analysis_run_id, tenant_record_id, tenant_workspace_id, idempotency_key, request_contract_version, snapshot_id, knowledge_cutoff, model_contract_version, output_profile, request_payload_sha256, request_payload, system_time, available_time) VALUES ('{}'::uuid, '{}'::uuid, E'{}', E'{}', {}, E'{}', E'{}'::timestamptz, E'{}', E'{}', E'{}', E'{}', '{}'::timestamptz, '{}'::timestamptz) ON CONFLICT (tenant_record_id, idempotency_key) DO NOTHING; GET DIAGNOSTICS inserted_count = ROW_COUNT; IF inserted_count = 1 THEN INSERT INTO analysis_run_state_event (analysis_run_state_event_id, tenant_record_id, analysis_run_id, state_sequence, run_state_code, system_time, available_time) VALUES ('{}'::uuid, '{}'::uuid, '{}'::uuid, 1, 'accepted', '{}'::timestamptz, '{}'::timestamptz); END IF; SELECT analysis_run_id, request_payload_sha256, request_payload INTO stored_run_id, stored_digest, stored_payload FROM analysis_run_request WHERE tenant_record_id = '{}'::uuid AND idempotency_key = E'{}'; IF stored_run_id IS DISTINCT FROM '{}'::uuid OR stored_digest IS DISTINCT FROM E'{}' OR stored_payload IS DISTINCT FROM E'{}' THEN RAISE EXCEPTION 'analysis-run idempotency conflict' USING ERRCODE = 'unique_violation'; END IF; END $tepp$",
        record.analysis_run_id,
        record.tenant_record_id,
        escape(&record.request.tenant_workspace_id),
        escape(&record.request.idempotency_key),
        record.request.contract_version,
        escape(&record.request.snapshot_id),
        escape(&record.request.knowledge_cutoff),
        escape(&record.request.model_contract_version),
        escape(&record.request.output_profile),
        record.request_payload_sha256,
        escape(&record.request_payload),
        record.system_time.to_rfc3339(),
        record.available_time.to_rfc3339(),
        derived_uuid(record.tenant_record_id, "accepted", &record.request_payload),
        record.tenant_record_id,
        record.analysis_run_id,
        record.system_time.to_rfc3339(),
        record.available_time.to_rfc3339(),
        record.tenant_record_id,
        escape(&record.request.idempotency_key),
        record.analysis_run_id,
        record.request_payload_sha256,
        escape(&record.request_payload),
    ))
}

/// Render an append-only lifecycle-event insert.
///
/// # Errors
///
/// Returns a fail-closed shape error before SQL generation.
pub fn insert_analysis_run_state_event_sql(
    request: &AnalysisRunRequestRecord,
    record: &AnalysisRunStateEventRecord,
) -> Result<String, PersistenceError> {
    record.validate()?;
    if record.tenant_record_id != request.tenant_record_id
        || record.analysis_run_id != request.analysis_run_id
    {
        return Err(PersistenceError::InvalidAnalysisRun);
    }
    if let Some(status) = &record.terminal_status {
        require_status_binding(&request.request, &request.accepted()?, status)
            .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
    }
    let (terminal, artifact_id, digest, schema, failure) = match &record.terminal_status {
        None => (
            "NULL".to_owned(),
            "NULL".to_owned(),
            "NULL".to_owned(),
            "NULL".to_owned(),
            "NULL".to_owned(),
        ),
        Some(status) => {
            let payload = status
                .to_json()
                .map_err(|_| PersistenceError::InvalidAnalysisRun)?;
            let result = status
                .terminal_result
                .as_ref()
                .ok_or(PersistenceError::InvalidAnalysisRun)?;
            let artifact_id = match &result.result_artifact_id {
                Some(value) => format!(
                    "'{}'::uuid",
                    Uuid::parse_str(value).map_err(|_| PersistenceError::InvalidAnalysisRun)?
                ),
                None => "NULL".to_owned(),
            };
            (
                format!("E'{}'", escape(&payload)),
                artifact_id,
                optional_literal(result.result_sha256.as_deref()),
                optional_literal(result.result_schema_version.as_deref()),
                optional_literal(result.failure_code.as_deref()),
            )
        }
    };
    Ok(format!(
        "INSERT INTO analysis_run_state_event (analysis_run_state_event_id, tenant_record_id, analysis_run_id, state_sequence, run_state_code, model_artifact_id, result_sha256, result_schema_version, failure_code, terminal_payload, system_time, available_time) VALUES ('{}'::uuid, '{}'::uuid, '{}'::uuid, {}, '{}', {}, {}, {}, {}, {}, '{}'::timestamptz, '{}'::timestamptz)",
        record.analysis_run_state_event_id,
        record.tenant_record_id,
        record.analysis_run_id,
        record.state_sequence,
        record.run_state.code(),
        artifact_id,
        digest,
        schema,
        failure,
        terminal,
        record.system_time.to_rfc3339(),
        record.available_time.to_rfc3339(),
    ))
}

/// Select the latest tenant-bound state needed to reconstruct the public status.
#[must_use]
pub fn select_analysis_run_status_sql(tenant_record_id: Uuid, analysis_run_id: Uuid) -> String {
    format!(
        "SELECT r.analysis_run_id, r.tenant_record_id, r.tenant_workspace_id, r.idempotency_key, r.request_contract_version, r.snapshot_id, r.knowledge_cutoff, r.model_contract_version, r.output_profile, r.request_payload_sha256, r.request_payload, r.system_time AS request_system_time, r.available_time AS request_available_time, e.analysis_run_state_event_id, e.state_sequence, e.run_state_code, e.model_artifact_id, e.result_sha256, e.result_schema_version, e.failure_code, e.terminal_payload, e.system_time AS event_system_time, e.available_time AS event_available_time FROM analysis_run_request AS r JOIN analysis_run_state_event AS e USING (tenant_record_id, analysis_run_id) WHERE tenant_record_id = '{tenant_record_id}'::uuid AND analysis_run_id = '{analysis_run_id}'::uuid ORDER BY state_sequence DESC LIMIT 1"
    )
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "''")
}

fn optional_literal(value: Option<&str>) -> String {
    value.map_or_else(
        || "NULL".to_owned(),
        |value| format!("E'{}'", escape(value)),
    )
}

fn derived_uuid(tenant_record_id: Uuid, purpose: &str, payload: &str) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(tenant_record_id.as_bytes());
    hasher.update([0]);
    hasher.update(purpose.as_bytes());
    hasher.update([0]);
    hasher.update(payload.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}
