#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! One-shot execution for a durable TEPP analysis run.

use analysis_engine::{
    ANALYSIS_ARTIFACT_SCHEMA_VERSION, AnalysisArtifact, AnalysisCorpus, AnalysisEvidenceUnit,
    AnalysisExecution, execute_analysis_run,
};
use persistence_postgres::{
    AnalysisRunState, AnalysisRunStateEventRecord, AnalysisWorkerStore, ModelRunRecord,
    PersistenceError, ReproducibilityManifestRecord, insert_analysis_run_state_event_sql,
    insert_model_artifact_sql, insert_model_run_sql, model_artifact_from_analysis_result,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt};
use temporal_core::{AvailableTime, EventTime, SystemTime};
use tepp_api::{AnalysisRunStatus, AnalysisRunStatusState, AnalysisRunTerminalState};
use uuid::Uuid;

/// Version of the canonical worker-input envelope.
pub const WORKER_INPUT_CONTRACT_VERSION: u16 = 1;
/// Maximum accepted worker-input JSON size.
pub const MAX_WORKER_INPUT_BYTES: usize = 64 * 1024 * 1024;
/// Only model contract executed by this bounded worker.
pub const SUPPORTED_MODEL_CONTRACT: &str = "temporal-evidence-v1";
/// Only output profile executed by this bounded worker.
pub const SUPPORTED_OUTPUT_PROFILE: &str = "validation-report";

/// Identity-free evidence metadata in a worker input envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerEvidenceUnit {
    /// Opaque evidence identity.
    pub evidence_id: String,
    /// Event-valid instant.
    pub event_time: String,
    /// First availability instant.
    pub available_time: String,
    /// Number of simultaneous membership assignments.
    pub membership_count: u32,
}

/// Canonical, manifest-digest-bound input for one durable run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisWorkerInput {
    /// Semantic contract version.
    pub contract_version: u16,
    /// Reproducibility manifest authorizing these exact bytes.
    pub reproducibility_manifest_id: Uuid,
    /// Immutable snapshot identity matching the accepted request.
    pub snapshot_id: String,
    /// SHA-256 of the canonical snapshot/evidence payload.
    pub source_snapshot_sha256: String,
    /// Bounded identity-free evidence metadata.
    pub evidence_units: Vec<WorkerEvidenceUnit>,
}

impl AnalysisWorkerInput {
    /// Parse, bound, validate, and canonicalize untrusted JSON.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisWorkerError::InvalidInput`] for oversized, malformed,
    /// unknown-field, invalid-clock, or invalid-evidence input.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisWorkerError> {
        if payload.len() > MAX_WORKER_INPUT_BYTES {
            return Err(AnalysisWorkerError::InvalidInput);
        }
        let input: Self =
            serde_json::from_str(payload).map_err(|_| AnalysisWorkerError::InvalidInput)?;
        input.corpus()?;
        if input.contract_version != WORKER_INPUT_CONTRACT_VERSION {
            return Err(AnalysisWorkerError::InvalidInput);
        }
        Ok(input)
    }

    /// Return canonical JSON used by the reproducibility-manifest digest.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisWorkerError::InvalidInput`] if validation or
    /// serialization fails.
    pub fn to_json(&self) -> Result<String, AnalysisWorkerError> {
        self.corpus()?;
        if self.contract_version != WORKER_INPUT_CONTRACT_VERSION {
            return Err(AnalysisWorkerError::InvalidInput);
        }
        serde_json::to_string(self).map_err(|_| AnalysisWorkerError::InvalidInput)
    }

    /// Return the canonical source-snapshot digest bound by the manifest.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisWorkerError::InvalidInput`] when the evidence contract
    /// cannot be validated or serialized.
    pub fn evidence_digest(&self) -> Result<String, AnalysisWorkerError> {
        self.corpus()?;
        let payload = serde_json::to_string(&CanonicalEvidencePayload {
            contract_version: self.contract_version,
            snapshot_id: &self.snapshot_id,
            evidence_units: &self.evidence_units,
        })
        .map_err(|_| AnalysisWorkerError::InvalidInput)?;
        Ok(format!("{:x}", Sha256::digest(payload.as_bytes())))
    }

    fn corpus(&self) -> Result<AnalysisCorpus, AnalysisWorkerError> {
        let mut identities = BTreeSet::new();
        let units = self
            .evidence_units
            .iter()
            .map(|unit| {
                if !identities.insert(unit.evidence_id.as_str()) {
                    return Err(AnalysisWorkerError::InvalidInput);
                }
                AnalysisEvidenceUnit::new(
                    unit.evidence_id.clone(),
                    EventTime::parse_rfc3339(&unit.event_time)
                        .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                    AvailableTime::parse_rfc3339(&unit.available_time)
                        .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                    unit.membership_count,
                )
                .map_err(|_| AnalysisWorkerError::InvalidInput)
            })
            .collect::<Result<Vec<_>, _>>()?;
        AnalysisCorpus::new(self.snapshot_id.clone(), units)
            .map_err(|_| AnalysisWorkerError::InvalidInput)
    }
}

#[derive(Serialize)]
struct CanonicalEvidencePayload<'input> {
    contract_version: u16,
    snapshot_id: &'input str,
    evidence_units: &'input [WorkerEvidenceUnit],
}

/// Runtime build identities compared with the selected manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerRuntimeIdentity {
    /// Exact producing Git object identity.
    pub code_commit_sha: String,
    /// SHA-256 of the dependency lock used to build the worker.
    pub dependency_lock_digest: String,
}

/// Observable result of one worker attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisWorkerOutcome {
    /// Validated terminal status, including unchanged prior terminal state.
    pub status: AnalysisRunStatus,
    /// Canonical artifact JSON produced by this attempt, when successful.
    pub artifact_json: Option<String>,
}

/// Fail-closed one-shot worker failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisWorkerError {
    /// The input or its binding is invalid.
    InvalidInput,
    /// Another worker currently owns the run lock.
    AlreadyLocked,
    /// The requested model/output pair is not implemented by this worker.
    UnsupportedRequest,
    /// Persistence or engine execution failed; a running run remains retryable.
    ExecutionFailed,
}

impl fmt::Display for AnalysisWorkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "invalid analysis worker input",
            Self::AlreadyLocked => "analysis run is already locked",
            Self::UnsupportedRequest => "analysis request is unsupported",
            Self::ExecutionFailed => "analysis worker execution failed",
        })
    }
}

impl std::error::Error for AnalysisWorkerError {}

/// Execute or resume one durable analysis run while holding its session lock.
///
/// Infrastructure failures after the running event leave the run retryable;
/// only the analysis engine may author a scientific terminal failure.
///
/// # Errors
///
/// Returns a typed lock, trust-boundary, support, persistence, or engine error.
pub fn execute_one<S: AnalysisWorkerStore>(
    store: &mut S,
    tenant_record_id: Uuid,
    analysis_run_id: Uuid,
    input: &AnalysisWorkerInput,
    runtime_identity: &WorkerRuntimeIdentity,
    completed_at: &str,
) -> Result<AnalysisWorkerOutcome, AnalysisWorkerError> {
    store
        .bind_worker_tenant(tenant_record_id)
        .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    if !store
        .try_worker_lock(tenant_record_id, analysis_run_id)
        .map_err(|_| AnalysisWorkerError::ExecutionFailed)?
    {
        return Err(AnalysisWorkerError::AlreadyLocked);
    }
    let outcome = execute_locked(
        store,
        tenant_record_id,
        analysis_run_id,
        input,
        runtime_identity,
        completed_at,
    );
    let unlocked = store.unlock_worker_run(tenant_record_id, analysis_run_id);
    match (outcome, unlocked) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) => Err(error),
        (Ok(_), Err(_)) => Err(AnalysisWorkerError::ExecutionFailed),
    }
}

fn execute_locked<S: AnalysisWorkerStore>(
    store: &mut S,
    tenant_record_id: Uuid,
    analysis_run_id: Uuid,
    input: &AnalysisWorkerInput,
    runtime_identity: &WorkerRuntimeIdentity,
    completed_at: &str,
) -> Result<AnalysisWorkerOutcome, AnalysisWorkerError> {
    let snapshot = store
        .load_worker_run(tenant_record_id, analysis_run_id)
        .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    if matches!(
        snapshot.status.run_state,
        AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed
    ) {
        return Ok(AnalysisWorkerOutcome {
            status: snapshot.status,
            artifact_json: None,
        });
    }
    let request = &snapshot.request_record.request;
    if request.model_contract_version != SUPPORTED_MODEL_CONTRACT
        || request.output_profile != SUPPORTED_OUTPUT_PROFILE
    {
        return Err(AnalysisWorkerError::UnsupportedRequest);
    }
    let manifest = store
        .load_worker_manifest(tenant_record_id, input.reproducibility_manifest_id)
        .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    let input_digest = input.evidence_digest()?;
    let cutoff = AvailableTime::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisWorkerError::InvalidInput)?;
    if input.snapshot_id != request.snapshot_id
        || input_digest != manifest.evidence_digest
        || input.source_snapshot_sha256 != input_digest
        || cutoff.instant() != manifest.knowledge_cutoff.instant()
        || runtime_identity.code_commit_sha != manifest.code_commit_sha
        || runtime_identity.dependency_lock_digest != manifest.dependency_lock_digest
    {
        return Err(AnalysisWorkerError::InvalidInput);
    }
    let completed_system =
        SystemTime::parse_rfc3339(completed_at).map_err(|_| AnalysisWorkerError::InvalidInput)?;
    let completed_available = AvailableTime::parse_rfc3339(completed_at)
        .map_err(|_| AnalysisWorkerError::InvalidInput)?;
    if snapshot.status.run_state == AnalysisRunStatusState::Accepted {
        let running = AnalysisRunStateEventRecord {
            analysis_run_state_event_id: Uuid::now_v7(),
            tenant_record_id,
            analysis_run_id,
            state_sequence: 2,
            run_state: AnalysisRunState::Running,
            terminal_status: None,
            system_time: completed_system,
            available_time: completed_available,
        };
        let sql = insert_analysis_run_state_event_sql(&snapshot.request_record, &running)
            .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
        store
            .execute_worker_sql(&sql)
            .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    }
    let accepted = snapshot
        .request_record
        .accepted()
        .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    let execution = execute_analysis_run(
        request,
        &accepted,
        &input.corpus()?,
        completed_at.to_owned(),
    )
    .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    publish_execution(
        store,
        tenant_record_id,
        analysis_run_id,
        &snapshot.request_record,
        &manifest,
        cutoff,
        completed_system,
        completed_available,
        execution,
    )
}

#[allow(clippy::too_many_arguments)]
fn publish_execution<S: AnalysisWorkerStore>(
    store: &mut S,
    tenant_record_id: Uuid,
    analysis_run_id: Uuid,
    request_record: &persistence_postgres::AnalysisRunRequestRecord,
    manifest: &ReproducibilityManifestRecord,
    cutoff: AvailableTime,
    completed_system: SystemTime,
    completed_available: AvailableTime,
    execution: AnalysisExecution,
) -> Result<AnalysisWorkerOutcome, AnalysisWorkerError> {
    let request = &request_record.request;
    let accepted = request_record
        .accepted()
        .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    let status = AnalysisRunStatus::terminal(request, &accepted, execution.terminal_result.clone())
        .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    let terminal_state = match execution.terminal_result.run_state {
        AnalysisRunTerminalState::Succeeded => AnalysisRunState::Succeeded,
        AnalysisRunTerminalState::Failed => AnalysisRunState::Failed,
    };
    let terminal = AnalysisRunStateEventRecord {
        analysis_run_state_event_id: Uuid::now_v7(),
        tenant_record_id,
        analysis_run_id,
        state_sequence: 3,
        run_state: terminal_state,
        terminal_status: Some(status.clone()),
        system_time: completed_system,
        available_time: completed_available,
    };
    let terminal_sql = insert_analysis_run_state_event_sql(request_record, &terminal)
        .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    let artifact_json = execution
        .artifact
        .as_ref()
        .map(AnalysisArtifact::to_json)
        .transpose()
        .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    if let Some(artifact) = execution.artifact {
        let model_run = ModelRunRecord {
            model_run_id: analysis_run_id,
            tenant_record_id,
            reproducibility_manifest_id: manifest.reproducibility_manifest_id,
            corpus_split_manifest_id: None,
            configuration_digest: request_record.request_payload_sha256.clone(),
            random_seed_manifest_digest: format!(
                "{:x}",
                Sha256::digest(b"tepp.deterministic-no-random-seed.v1")
            ),
            engine_version_label: env!("CARGO_PKG_VERSION").to_owned(),
            compute_backend_code: "cpu_f64".to_owned(),
            knowledge_cutoff: cutoff,
            system_time: completed_system,
            available_time: completed_available,
        };
        let artifact_record = model_artifact_from_analysis_result(
            tenant_record_id,
            model_run.model_run_id,
            &execution.terminal_result,
            None,
            completed_system,
            completed_available,
        )?;
        debug_assert_eq!(artifact.schema_version, ANALYSIS_ARTIFACT_SCHEMA_VERSION);
        store
            .execute_worker_transaction(&[
                insert_model_run_sql(&model_run)?,
                insert_model_artifact_sql(&artifact_record)?,
                terminal_sql,
            ])
            .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    } else {
        store
            .execute_worker_transaction(&[terminal_sql])
            .map_err(|_| AnalysisWorkerError::ExecutionFailed)?;
    }
    Ok(AnalysisWorkerOutcome {
        status,
        artifact_json,
    })
}

impl From<PersistenceError> for AnalysisWorkerError {
    fn from(_: PersistenceError) -> Self {
        Self::ExecutionFailed
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AnalysisWorkerError, AnalysisWorkerInput, MAX_WORKER_INPUT_BYTES,
        WORKER_INPUT_CONTRACT_VERSION, WorkerEvidenceUnit, WorkerRuntimeIdentity, execute_one,
    };
    use persistence_postgres::{
        AnalysisRunRequestRecord, AnalysisRunWorkerSnapshot, PersistenceError,
        ReproducibilityManifestRecord,
    };
    use temporal_core::{AvailableTime, SystemTime};
    use tepp_api::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunRequest, AnalysisRunStatus,
        AnalysisRunTerminalResult,
    };
    use uuid::Uuid;

    fn input() -> AnalysisWorkerInput {
        let mut input = AnalysisWorkerInput {
            contract_version: WORKER_INPUT_CONTRACT_VERSION,
            reproducibility_manifest_id: Uuid::nil(),
            snapshot_id: "snapshot-1".into(),
            source_snapshot_sha256: String::new(),
            evidence_units: vec![WorkerEvidenceUnit {
                evidence_id: "evidence-1".into(),
                event_time: "2026-01-01T00:00:00Z".into(),
                available_time: "2026-01-02T00:00:00Z".into(),
                membership_count: 2,
            }],
        };
        input.source_snapshot_sha256 = input.evidence_digest().expect("digest");
        input
    }

    fn snapshot(state: &str) -> AnalysisRunWorkerSnapshot {
        let request = AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "worker-idempotency".into(),
            tenant_workspace_id: "workspace-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-01-03T00:00:00Z".into(),
            model_contract_version: super::SUPPORTED_MODEL_CONTRACT.into(),
            output_profile: super::SUPPORTED_OUTPUT_PROFILE.into(),
        };
        let request_record = AnalysisRunRequestRecord::from_request(
            Uuid::nil(),
            &request,
            SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
            AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
        )
        .expect("record");
        let accepted = request_record.accepted().expect("receipt");
        let status = match state {
            "accepted" => AnalysisRunStatus::accepted(&accepted).expect("accepted"),
            "running" => AnalysisRunStatus::running(&accepted).expect("running"),
            "failed" => AnalysisRunStatus::terminal(
                &request,
                &accepted,
                AnalysisRunTerminalResult::failed(
                    &request,
                    &accepted,
                    "2026-01-04T00:00:00Z",
                    "no_eligible_evidence",
                )
                .expect("failed result"),
            )
            .expect("failed status"),
            _ => panic!("unknown test state"),
        };
        AnalysisRunWorkerSnapshot {
            request_record,
            status,
        }
    }

    #[test]
    #[should_panic(expected = "unknown test state")]
    fn test_snapshot_fixture_rejects_unknown_state() {
        let _ = snapshot("unknown");
    }

    #[derive(Clone)]
    struct FakeStore {
        snapshot: AnalysisRunWorkerSnapshot,
        manifest: ReproducibilityManifestRecord,
        locked: bool,
        fail_on: Option<&'static str>,
        executed: Vec<String>,
        transactions: Vec<Vec<String>>,
    }

    impl FakeStore {
        fn new(state: &str, input: &AnalysisWorkerInput) -> Self {
            Self {
                snapshot: snapshot(state),
                manifest: ReproducibilityManifestRecord {
                    reproducibility_manifest_id: input.reproducibility_manifest_id,
                    tenant_record_id: Uuid::nil(),
                    knowledge_cutoff: AvailableTime::parse_rfc3339("2026-01-03T00:00:00Z")
                        .expect("cutoff"),
                    evidence_digest: input.evidence_digest().expect("digest"),
                    code_commit_sha: "a".repeat(40),
                    dependency_lock_digest: "b".repeat(64),
                    system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
                    available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z")
                        .expect("available"),
                },
                locked: false,
                fail_on: None,
                executed: Vec::new(),
                transactions: Vec::new(),
            }
        }
    }

    impl persistence_postgres::AnalysisWorkerStore for FakeStore {
        fn bind_worker_tenant(&mut self, _: Uuid) -> Result<(), PersistenceError> {
            fail(self.fail_on, "bind")
        }

        fn try_worker_lock(&mut self, _: Uuid, _: Uuid) -> Result<bool, PersistenceError> {
            fail(self.fail_on, "lock")?;
            if self.locked {
                Ok(false)
            } else {
                self.locked = true;
                Ok(true)
            }
        }

        fn unlock_worker_run(&mut self, _: Uuid, _: Uuid) -> Result<(), PersistenceError> {
            fail(self.fail_on, "unlock")?;
            self.locked = false;
            Ok(())
        }

        fn load_worker_run(
            &mut self,
            _: Uuid,
            _: Uuid,
        ) -> Result<AnalysisRunWorkerSnapshot, PersistenceError> {
            fail(self.fail_on, "run")?;
            Ok(self.snapshot.clone())
        }

        fn load_worker_manifest(
            &mut self,
            _: Uuid,
            _: Uuid,
        ) -> Result<ReproducibilityManifestRecord, PersistenceError> {
            fail(self.fail_on, "manifest")?;
            Ok(self.manifest.clone())
        }

        fn execute_worker_sql(&mut self, sql: &str) -> Result<(), PersistenceError> {
            fail(self.fail_on, "execute")?;
            self.executed.push(sql.into());
            Ok(())
        }

        fn execute_worker_transaction(
            &mut self,
            statements: &[String],
        ) -> Result<(), PersistenceError> {
            fail(self.fail_on, "transaction")?;
            self.transactions.push(statements.to_vec());
            Ok(())
        }
    }

    fn fail(selected: Option<&str>, stage: &str) -> Result<(), PersistenceError> {
        if selected == Some(stage) {
            Err(PersistenceError::SqlExecutionFailed)
        } else {
            Ok(())
        }
    }

    fn identity() -> WorkerRuntimeIdentity {
        WorkerRuntimeIdentity {
            code_commit_sha: "a".repeat(40),
            dependency_lock_digest: "b".repeat(64),
        }
    }

    fn run(
        store: &mut FakeStore,
        input: &AnalysisWorkerInput,
    ) -> Result<super::AnalysisWorkerOutcome, AnalysisWorkerError> {
        execute_one(
            store,
            Uuid::nil(),
            store.snapshot.request_record.analysis_run_id,
            input,
            &identity(),
            "2026-01-04T00:00:00Z",
        )
    }

    #[test]
    fn canonical_input_round_trips_and_digest_ignores_manifest_identity() {
        let input = input();
        let json = input.to_json().expect("json");
        assert_eq!(AnalysisWorkerInput::from_json(&json).expect("parse"), input);
        let mut another_manifest = input.clone();
        another_manifest.reproducibility_manifest_id = Uuid::from_u128(1);
        assert_eq!(another_manifest.evidence_digest(), input.evidence_digest());
        assert_eq!(
            WorkerRuntimeIdentity {
                code_commit_sha: "a".repeat(40),
                dependency_lock_digest: "b".repeat(64),
            }
            .code_commit_sha
            .len(),
            40
        );
    }

    #[test]
    fn untrusted_input_rejects_every_shape_and_bound_failure() {
        let valid = input().to_json().expect("valid");
        assert_eq!(
            AnalysisWorkerInput::from_json(&"x".repeat(MAX_WORKER_INPUT_BYTES + 1)),
            Err(AnalysisWorkerError::InvalidInput)
        );
        assert_eq!(
            AnalysisWorkerInput::from_json("{}"),
            Err(AnalysisWorkerError::InvalidInput)
        );
        assert_eq!(
            AnalysisWorkerInput::from_json(
                &valid.replace("\"contract_version\":1", "\"contract_version\":2")
            ),
            Err(AnalysisWorkerError::InvalidInput)
        );
        assert_eq!(
            AnalysisWorkerInput::from_json(
                &valid.replace("\"membership_count\":2", "\"membership_count\":0")
            ),
            Err(AnalysisWorkerError::InvalidInput)
        );
        assert_eq!(
            AnalysisWorkerInput::from_json(&valid.replace("2026-01-01T00:00:00Z", "not-a-clock")),
            Err(AnalysisWorkerError::InvalidInput)
        );
        assert_eq!(
            AnalysisWorkerInput::from_json(&valid.replacen('{', "{\"unknown\":1,", 1)),
            Err(AnalysisWorkerError::InvalidInput)
        );
        let mut duplicate = input();
        duplicate
            .evidence_units
            .push(duplicate.evidence_units[0].clone());
        assert_eq!(duplicate.to_json(), Err(AnalysisWorkerError::InvalidInput));
        let mut bad_version = input();
        bad_version.contract_version = 2;
        assert_eq!(
            bad_version.to_json(),
            Err(AnalysisWorkerError::InvalidInput)
        );
    }

    #[test]
    fn worker_errors_are_redacted_and_stable() {
        assert_eq!(
            AnalysisWorkerError::InvalidInput.to_string(),
            "invalid analysis worker input"
        );
        assert_eq!(
            AnalysisWorkerError::AlreadyLocked.to_string(),
            "analysis run is already locked"
        );
        assert_eq!(
            AnalysisWorkerError::UnsupportedRequest.to_string(),
            "analysis request is unsupported"
        );
        assert_eq!(
            AnalysisWorkerError::from(PersistenceError::SqlExecutionFailed),
            AnalysisWorkerError::ExecutionFailed
        );
    }

    #[test]
    fn accepted_and_running_runs_publish_once_while_terminal_is_unchanged() {
        let input = input();
        let mut accepted = FakeStore::new("accepted", &input);
        let outcome = run(&mut accepted, &input).expect("accepted execution");
        assert!(outcome.artifact_json.is_some());
        assert_eq!(accepted.executed.len(), 1);
        assert_eq!(accepted.transactions[0].len(), 3);
        assert!(!accepted.locked);

        let mut running = FakeStore::new("running", &input);
        run(&mut running, &input).expect("running recovery");
        assert!(running.executed.is_empty());
        assert_eq!(running.transactions[0].len(), 3);

        let mut terminal = FakeStore::new("failed", &input);
        let terminal_outcome = run(&mut terminal, &input).expect("terminal no-op");
        assert!(terminal_outcome.artifact_json.is_none());
        assert!(terminal.executed.is_empty());
        assert!(terminal.transactions.is_empty());
    }

    #[test]
    fn scientific_empty_corpus_publishes_only_the_failed_terminal_event() {
        let mut input = input();
        input.evidence_units.clear();
        input.source_snapshot_sha256 = input.evidence_digest().expect("digest");
        let mut store = FakeStore::new("accepted", &input);
        let outcome = run(&mut store, &input).expect("scientific failure");
        assert!(outcome.artifact_json.is_none());
        assert_eq!(store.transactions[0].len(), 1);
    }

    #[test]
    fn lock_support_and_every_binding_fail_closed_without_terminal_publication() {
        let input = input();
        assert_eq!(
            AnalysisWorkerError::ExecutionFailed.to_string(),
            "analysis worker execution failed"
        );
        assert_eq!(
            AnalysisWorkerError::from(PersistenceError::SqlExecutionFailed),
            AnalysisWorkerError::ExecutionFailed
        );
        let mut locked = FakeStore::new("accepted", &input);
        locked.locked = true;
        assert_eq!(
            run(&mut locked, &input),
            Err(AnalysisWorkerError::AlreadyLocked)
        );

        let mut unsupported = FakeStore::new("accepted", &input);
        unsupported.snapshot.request_record.request.output_profile = "other".into();
        assert_eq!(
            run(&mut unsupported, &input),
            Err(AnalysisWorkerError::UnsupportedRequest)
        );

        let mutations: [fn(&mut FakeStore); 5] = [
            |store| store.manifest.evidence_digest = "c".repeat(64),
            |store| store.manifest.code_commit_sha = "d".repeat(40),
            |store| store.manifest.dependency_lock_digest = "e".repeat(64),
            |store| {
                store.manifest.knowledge_cutoff =
                    AvailableTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("cutoff");
            },
            |store| store.snapshot.request_record.request.snapshot_id = "other".into(),
        ];
        for mutate in mutations {
            let mut store = FakeStore::new("accepted", &input);
            mutate(&mut store);
            assert_eq!(
                run(&mut store, &input),
                Err(AnalysisWorkerError::InvalidInput)
            );
            assert!(store.transactions.is_empty());
        }
    }

    #[test]
    fn infrastructure_failures_remain_retryable_and_unlock_is_attempted() {
        let input = input();
        for stage in [
            "bind",
            "lock",
            "run",
            "manifest",
            "execute",
            "transaction",
            "unlock",
        ] {
            let mut store = FakeStore::new("accepted", &input);
            store.fail_on = Some(stage);
            assert_eq!(
                run(&mut store, &input),
                Err(AnalysisWorkerError::ExecutionFailed)
            );
            if stage != "bind" && stage != "lock" && stage != "unlock" {
                assert!(!store.locked);
            }
        }
    }

    #[test]
    fn invalid_completion_clock_fails_before_running_publication() {
        let input = input();
        let mut store = FakeStore::new("accepted", &input);
        assert_eq!(
            execute_one(
                &mut store,
                Uuid::nil(),
                Uuid::from_u128(1),
                &input,
                &identity(),
                "not-a-clock",
            ),
            Err(AnalysisWorkerError::InvalidInput)
        );
        assert!(store.executed.is_empty());
    }
}
