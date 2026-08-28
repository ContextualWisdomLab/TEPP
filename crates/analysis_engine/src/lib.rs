#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Deterministic, cutoff-safe execution for the first TEPP analysis vertical slice.
//!
//! The engine consumes identity-free evidence metadata, excludes evidence that
//! was unavailable at the requested knowledge cutoff, counts multiple-membership
//! assignments without collapsing them, and emits a digest-bound terminal result
//! through [`tepp_api`]. It deliberately does not claim latent-variable or topic
//! estimation authority; those estimators remain separate scientific crates.
//! estimation authority; it invokes estimators through their scientific crate
//! contracts and preserves their artifact meaning.

mod case_deletion_refit;
mod lineage_criterion;
mod topic_context_posterior;
mod topic_lineage_artifact;

use model_selection::{FittedCandidateSelectionFailure, ModelSelectionError};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
    ApiError,
};
use topic_measurement::TopicMeasurementError;

/// One document admitted to exhaustive case-deletion fitting.
pub use case_deletion_refit::CaseDeletionDocument;
/// Fit context with independent seed-domain provenance.
pub use case_deletion_refit::CaseDeletionFitContext;
/// Generic scientific fitter invoked for full and actual deleted corpora.
pub use case_deletion_refit::CaseDeletionRefitter;
/// One actual deleted-data posterior fit.
pub use case_deletion_refit::DeletedDocumentRefit;
/// Fail-closed exhaustive case-deletion error.
pub use case_deletion_refit::ExhaustiveCaseDeletionError;
/// Exhaustive full and deleted-data posterior fits.
pub use case_deletion_refit::ExhaustiveCaseDeletionFits;
/// Fit the full corpus and every actual one-document deletion.
pub use case_deletion_refit::fit_exhaustive_case_deletion;
/// Rust-owned independent TDT link-criterion posterior fitting contracts.
pub use lineage_criterion::{
    LineageCriterionFit, LineageCriterionFitError, LineageCriterionObservation,
    fit_lineage_criterion_posteriors,
};
/// Bounded posterior topic-context producer contract and record types.
pub use topic_context_posterior::{
    TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT, TOPIC_CONTEXT_POSTERIOR_OUTPUT_PROFILE,
    TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION, TopicActivityInterval, TopicContextMembership,
    TopicContextPosteriorArtifact, TopicDocumentRelation, TopicLineageEvent,
    TopicPostPlausibleValue, assemble_topic_context_posterior,
};
/// Topic-lineage artifact and execution contracts from this engine.
pub use topic_lineage_artifact::{
    TOPIC_LINEAGE_ARTIFACT_BYTE_LIMIT, TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION,
    TOPIC_LINEAGE_MODEL_CONTRACT_VERSION, TOPIC_LINEAGE_OUTPUT_PROFILE, TopicLineageArtifact,
    TopicLineageArtifactEdge, TopicLineageCandidateOutcome, TopicLineageExecution,
    TopicLineageFitManifest, execute_selected_topic_lineage_run, execute_topic_lineage_run,
};

/// Versioned artifact schema emitted by this engine.
pub const ANALYSIS_ARTIFACT_SCHEMA_VERSION: &str = "tepp.temporal_evidence_readiness.v1";
/// Number of deterministic statistics represented in the artifact summary.
pub const ANALYSIS_STATISTIC_COUNT: u64 = 4;
/// Maximum number of evidence units accepted by one in-memory execution.
pub const MAX_EVIDENCE_UNITS: usize = 100_000;
/// Maximum UTF-8 byte length of one snapshot or opaque evidence identifier.
pub const MAX_ANALYSIS_IDENTIFIER_BYTES: usize = 256;

/// A bounded identity-free evidence unit offered to one analysis run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisEvidenceUnit {
    evidence_id: String,
    event_time: EventTime,
    available_time: AvailableTime,
    membership_count: u32,
}

impl AnalysisEvidenceUnit {
    /// Construct an evidence unit with explicit event, availability, and
    /// multiple-membership metadata.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the identity is
    /// empty or no membership assignment is supplied.
    pub fn new(
        evidence_id: impl Into<String>,
        event_time: EventTime,
        available_time: AvailableTime,
        membership_count: u32,
    ) -> Result<Self, AnalysisEngineError> {
        let evidence_id = evidence_id.into();
        if !valid_identifier(&evidence_id) || membership_count == 0 {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            evidence_id,
            event_time,
            available_time,
            membership_count,
        })
    }

    /// Return the opaque evidence identity.
    #[must_use]
    pub fn evidence_id(&self) -> &str {
        &self.evidence_id
    }

    /// Return the event-valid time.
    #[must_use]
    pub const fn event_time(&self) -> EventTime {
        self.event_time
    }

    /// Return the evidence availability time.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }

    /// Return the number of simultaneous membership assignments.
    #[must_use]
    pub const fn membership_count(&self) -> u32 {
        self.membership_count
    }
}

/// A bounded snapshot of evidence metadata for one analysis run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisCorpus {
    snapshot_id: String,
    evidence_units: Vec<AnalysisEvidenceUnit>,
}

impl AnalysisCorpus {
    /// Construct a snapshot-owned corpus.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] for an empty snapshot
    /// identity or [`AnalysisEngineError::LimitExceeded`] for an oversized
    /// in-memory corpus.
    pub fn new(
        snapshot_id: impl Into<String>,
        evidence_units: Vec<AnalysisEvidenceUnit>,
    ) -> Result<Self, AnalysisEngineError> {
        let snapshot_id = snapshot_id.into();
        if !valid_identifier(&snapshot_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        if evidence_units.len() > MAX_EVIDENCE_UNITS {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        Ok(Self {
            snapshot_id,
            evidence_units,
        })
    }

    /// Return the immutable snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return the evidence units in source order.
    #[must_use]
    pub fn evidence_units(&self) -> &[AnalysisEvidenceUnit] {
        &self.evidence_units
    }
}

/// Digest-bound, identity-free output artifact for one successful execution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AnalysisArtifact {
    /// Versioned artifact schema.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical cutoff applied to availability.
    pub knowledge_cutoff: String,
    /// Number of evidence units available by the cutoff.
    pub eligible_evidence_count: u64,
    /// Sum of preserved multiple-membership assignments.
    pub eligible_membership_count: u64,
    /// Earliest event-valid time among eligible evidence.
    pub earliest_event_time: String,
    /// Latest event-valid time among eligible evidence.
    pub latest_event_time: String,
}

impl AnalysisArtifact {
    /// Serialize the canonical artifact bytes used for digesting.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::SerializationFailure`] if serialization
    /// unexpectedly fails.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)
    }

    /// Return the lowercase SHA-256 digest of the canonical artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::SerializationFailure`] if serialization
    /// unexpectedly fails.
    pub fn sha256(&self) -> Result<String, AnalysisEngineError> {
        self.to_json()
            .map(|json| format_digest(Sha256::digest(json.into_bytes())))
    }
}

/// One complete execution response, including the internal artifact and the
/// request-bound terminal wire result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisExecution {
    /// Digest-bound artifact, present only when the terminal result succeeded.
    pub artifact: Option<AnalysisArtifact>,
    /// Request-bound terminal result returned to the service boundary.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Fail-closed errors from the deterministic analysis vertical slice.
#[derive(Clone, Debug, PartialEq)]
pub enum AnalysisEngineError {
    /// Request or accepted receipt failed its API contract.
    Api(ApiError),
    /// Evidence metadata was empty or structurally invalid.
    InvalidEvidence,
    /// Two evidence units reused one opaque identity.
    DuplicateEvidence,
    /// Corpus snapshot identity differed from the request snapshot.
    SnapshotMismatch,
    /// A bounded integer aggregation overflowed.
    ArithmeticOverflow,
    /// A serialized artifact could not be produced.
    SerializationFailure,
    /// The in-memory corpus exceeded the execution bound.
    LimitExceeded,
    /// A topic-measurement estimator rejected or could not complete the fit.
    TopicMeasurement(TopicMeasurementError),
    /// Candidate-`K` fitting or statistical selection failed.
    ModelSelection(ModelSelectionError),
    /// Fitted candidate selection failed with its completed diagnostic receipt.
    FittedModelSelection(FittedCandidateSelectionFailure),
    /// A topic-lineage artifact violated its bounded schema or count invariants.
    InvalidTopicLineageArtifact,
}

impl fmt::Display for AnalysisEngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Api(error) => return error.fmt(formatter),
            Self::InvalidEvidence => "invalid analysis evidence",
            Self::DuplicateEvidence => "duplicate analysis evidence identity",
            Self::SnapshotMismatch => "analysis snapshot identity mismatch",
            Self::ArithmeticOverflow => "analysis evidence count overflow",
            Self::SerializationFailure => "analysis artifact serialization failed",
            Self::LimitExceeded => "analysis corpus exceeded its execution bound",
            Self::TopicMeasurement(error) => return error.fmt(formatter),
            Self::ModelSelection(error) => return error.fmt(formatter),
            Self::FittedModelSelection(error) => return error.fmt(formatter),
            Self::InvalidTopicLineageArtifact => "invalid topic lineage artifact",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AnalysisEngineError {}

impl From<ApiError> for AnalysisEngineError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<TopicMeasurementError> for AnalysisEngineError {
    fn from(error: TopicMeasurementError) -> Self {
        Self::TopicMeasurement(error)
    }
}

/// Execute the cutoff-safe temporal evidence readiness analysis.
///
/// Evidence whose `available_time` is later than the request cutoff is excluded
/// before aggregation. Event time remains a separate clock, and all membership
/// assignments are summed rather than collapsed to one group. Successful output
/// contains only bounded counts and temporal extrema; source text, credentials,
/// and direct identities never enter the artifact.
///
/// # Errors
///
/// Returns a fail-closed error for invalid contracts, snapshot mismatch,
/// duplicate evidence identities, or invalid arithmetic/serialization state.
pub fn execute_analysis_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    corpus: &AnalysisCorpus,
    completed_at: impl Into<String>,
) -> Result<AnalysisExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != corpus.snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::Api(ApiError::InvalidWirePayload))?;
    let mut identities = BTreeSet::new();
    let mut eligible = Vec::new();
    for unit in &corpus.evidence_units {
        if !identities.insert(unit.evidence_id.clone()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        if unit.available_time.instant() <= cutoff.instant() {
            eligible.push(unit);
        }
    }

    let completed_at = completed_at.into();
    if eligible.is_empty() {
        let terminal_result = AnalysisRunTerminalResult::failed(
            request,
            accepted,
            completed_at,
            "no_eligible_evidence",
        )?;
        return Ok(AnalysisExecution {
            artifact: None,
            terminal_result,
        });
    }

    // The corpus bound makes this conversion strictly smaller than
    // `u64::MAX`; the fold still fails closed through checked arithmetic so a
    // future bound change cannot wrap membership totals silently.
    let eligible_evidence_count = eligible.len() as u64;
    let eligible_membership_count = eligible.iter().try_fold(0_u64, |sum, unit| {
        add_membership_count(sum, unit.membership_count)
    })?;
    let (earliest, latest) = eligible.iter().fold(
        (eligible[0].event_time, eligible[0].event_time),
        |(earliest, latest), unit| (earliest.min(unit.event_time), latest.max(unit.event_time)),
    );
    let artifact = AnalysisArtifact {
        schema_version: ANALYSIS_ARTIFACT_SCHEMA_VERSION.to_owned(),
        run_id: accepted.run_id.clone(),
        snapshot_id: request.snapshot_id.clone(),
        knowledge_cutoff: cutoff.to_rfc3339(),
        eligible_evidence_count,
        eligible_membership_count,
        earliest_event_time: earliest.to_rfc3339(),
        latest_event_time: latest.to_rfc3339(),
    };
    artifact.sha256().and_then(move |digest| {
        let artifact_id = format!("analysis_artifact_{}", &digest[..16]);
        let summary = AnalysisResultSummary {
            analysis_family: "temporal_evidence_readiness".to_owned(),
            evidence_count: eligible_evidence_count,
            statistic_count: ANALYSIS_STATISTIC_COUNT,
            validation_status: "validated".to_owned(),
        };
        let terminal_result = AnalysisRunTerminalResult::succeeded(
            request,
            accepted,
            artifact_id,
            digest,
            ANALYSIS_ARTIFACT_SCHEMA_VERSION,
            completed_at,
            summary,
        )
        .map_err(AnalysisEngineError::from)?;
        Ok(AnalysisExecution {
            artifact: Some(artifact),
            terminal_result,
        })
    })
}

fn add_membership_count(sum: u64, membership_count: u32) -> Result<u64, AnalysisEngineError> {
    sum.checked_add(u64::from(membership_count))
        .ok_or(AnalysisEngineError::ArithmeticOverflow)
}

/// Require the accepted receipt to carry the request's idempotency identity.
fn require_receipt_identity(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
) -> Result<(), AnalysisEngineError> {
    if request.idempotency_key != accepted.idempotency_key {
        return Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload));
    }
    Ok(())
}

fn format_digest(digest: impl AsRef<[u8]>) -> String {
    let mut output = String::with_capacity(digest.as_ref().len() * 2);
    for byte in digest.as_ref() {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn valid_identifier(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= MAX_ANALYSIS_IDENTIFIER_BYTES
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::{
        ANALYSIS_ARTIFACT_SCHEMA_VERSION, ANALYSIS_STATISTIC_COUNT, AnalysisCorpus,
        AnalysisEngineError, AnalysisEvidenceUnit, MAX_ANALYSIS_IDENTIFIER_BYTES,
        MAX_EVIDENCE_UNITS, ModelSelectionError, TopicMeasurementError, add_membership_count,
        execute_analysis_run,
    };
    use temporal_core::{AvailableTime, EventTime};
    use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState, ApiError};

    fn request() -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: 1,
            idempotency_key: "idem-analysis-1".into(),
            tenant_workspace_id: "tenant-workspace-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "temporal-evidence-v1".into(),
            output_profile: "validation-report".into(),
        }
    }

    fn accepted() -> AnalysisRunAccepted {
        AnalysisRunAccepted::new("run-1", "accepted", "idem-analysis-1").expect("accepted")
    }

    fn unit(id: &str, event: &str, available: &str, memberships: u32) -> AnalysisEvidenceUnit {
        AnalysisEvidenceUnit::new(
            id,
            EventTime::parse_rfc3339(event).expect("event"),
            AvailableTime::parse_rfc3339(available).expect("available"),
            memberships,
        )
        .expect("unit")
    }

    #[test]
    fn successful_run_is_cutoff_safe_and_preserves_multiple_memberships() {
        let corpus = AnalysisCorpus::new(
            "snapshot-1",
            vec![
                unit(
                    "evidence-1",
                    "2026-07-01T00:00:00Z",
                    "2026-07-15T00:00:00Z",
                    2,
                ),
                unit(
                    "evidence-2",
                    "2026-07-20T00:00:00Z",
                    "2026-08-01T00:00:00Z",
                    3,
                ),
                unit(
                    "late-evidence",
                    "2026-07-25T00:00:00Z",
                    "2026-08-02T00:00:00Z",
                    9,
                ),
            ],
        )
        .expect("corpus");
        let execution =
            execute_analysis_run(&request(), &accepted(), &corpus, "2026-08-03T00:00:00Z")
                .expect("execution");
        let artifact = execution.artifact.expect("artifact");
        assert_eq!(artifact.schema_version, ANALYSIS_ARTIFACT_SCHEMA_VERSION);
        assert_eq!(artifact.eligible_evidence_count, 2);
        assert_eq!(artifact.eligible_membership_count, 5);
        assert_eq!(artifact.earliest_event_time, "2026-07-01T00:00:00Z");
        assert_eq!(artifact.latest_event_time, "2026-07-20T00:00:00Z");
        assert_eq!(
            execution.terminal_result.run_state,
            AnalysisRunTerminalState::Succeeded
        );
        let summary = execution.terminal_result.summary.as_ref().expect("summary");
        assert_eq!(summary.evidence_count, 2);
        assert_eq!(summary.statistic_count, ANALYSIS_STATISTIC_COUNT);
        assert_eq!(summary.validation_status, "validated");
        assert!(execution.terminal_result.result_sha256.is_some());
        assert!(execution.terminal_result.to_json().is_ok());
    }

    #[test]
    fn no_eligible_evidence_returns_a_redacted_failure_result() {
        let corpus = AnalysisCorpus::new(
            "snapshot-1",
            vec![unit(
                "late",
                "2026-07-25T00:00:00Z",
                "2026-08-02T00:00:00Z",
                1,
            )],
        )
        .expect("corpus");
        let execution =
            execute_analysis_run(&request(), &accepted(), &corpus, "2026-08-03T00:00:00Z")
                .expect("failure result");
        assert!(execution.artifact.is_none());
        assert_eq!(
            execution.terminal_result.run_state,
            AnalysisRunTerminalState::Failed
        );
        assert_eq!(
            execution.terminal_result.failure_code.as_deref(),
            Some("no_eligible_evidence")
        );
        assert!(execution.terminal_result.summary.is_none());
    }

    #[test]
    fn trust_boundary_and_shape_errors_fail_closed() {
        assert_eq!(
            AnalysisEvidenceUnit::new(
                "",
                EventTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("event"),
                AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available"),
                1,
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            AnalysisCorpus::new("", Vec::new()),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            AnalysisCorpus::new("\n", Vec::new()),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            AnalysisCorpus::new("s".repeat(MAX_ANALYSIS_IDENTIFIER_BYTES + 1), Vec::new()),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            AnalysisEvidenceUnit::new(
                "e",
                EventTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("event"),
                AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available"),
                0,
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            AnalysisEvidenceUnit::new(
                "e".repeat(MAX_ANALYSIS_IDENTIFIER_BYTES + 1),
                EventTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("event"),
                AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available"),
                1,
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        let corpus = AnalysisCorpus::new(
            "snapshot-2",
            vec![unit(
                "evidence-1",
                "2026-07-01T00:00:00Z",
                "2026-07-01T00:00:00Z",
                1,
            )],
        )
        .expect("corpus");
        assert_eq!(
            execute_analysis_run(&request(), &accepted(), &corpus, "2026-08-03T00:00:00Z"),
            Err(AnalysisEngineError::SnapshotMismatch)
        );
        let mismatched_receipt =
            AnalysisRunAccepted::new("run-1", "accepted", "other-idempotency").expect("receipt");
        let matching_corpus = AnalysisCorpus::new(
            "snapshot-1",
            vec![unit(
                "evidence-1",
                "2026-07-01T00:00:00Z",
                "2026-07-01T00:00:00Z",
                1,
            )],
        )
        .expect("corpus");
        assert_eq!(
            execute_analysis_run(
                &request(),
                &mismatched_receipt,
                &matching_corpus,
                "2026-08-03T00:00:00Z"
            ),
            Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
        );
        let duplicate = AnalysisCorpus::new(
            "snapshot-1",
            vec![
                unit("same", "2026-07-01T00:00:00Z", "2026-07-01T00:00:00Z", 1),
                unit("same", "2026-07-02T00:00:00Z", "2026-07-02T00:00:00Z", 1),
            ],
        )
        .expect("corpus");
        assert_eq!(
            execute_analysis_run(&request(), &accepted(), &duplicate, "2026-08-03T00:00:00Z"),
            Err(AnalysisEngineError::DuplicateEvidence)
        );
        assert_eq!(
            AnalysisEngineError::Api(ApiError::LimitExceeded).to_string(),
            "API request exceeded configured limits"
        );
        assert_eq!(
            AnalysisEngineError::SerializationFailure.to_string(),
            "analysis artifact serialization failed"
        );
    }

    #[test]
    fn public_accessors_limits_and_error_messages_are_executable() {
        let evidence = unit(
            "evidence-accessor",
            "2026-07-01T00:00:00Z",
            "2026-07-01T00:00:00Z",
            4,
        );
        assert_eq!(evidence.evidence_id(), "evidence-accessor");
        assert_eq!(
            evidence.event_time(),
            EventTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("event")
        );
        assert_eq!(
            evidence.available_time(),
            AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available")
        );
        assert_eq!(evidence.membership_count(), 4);
        let corpus =
            AnalysisCorpus::new("snapshot-accessor", vec![evidence.clone()]).expect("corpus");
        assert_eq!(corpus.snapshot_id(), "snapshot-accessor");
        assert_eq!(corpus.evidence_units(), &[evidence]);

        let oversized = AnalysisCorpus::new(
            "snapshot-limit",
            vec![
                unit("bounded", "2026-07-01T00:00:00Z", "2026-07-01T00:00:00Z", 1,);
                MAX_EVIDENCE_UNITS + 1
            ],
        );
        assert_eq!(oversized, Err(AnalysisEngineError::LimitExceeded));

        let messages = [
            (
                AnalysisEngineError::InvalidEvidence,
                "invalid analysis evidence",
            ),
            (
                AnalysisEngineError::DuplicateEvidence,
                "duplicate analysis evidence identity",
            ),
            (
                AnalysisEngineError::SnapshotMismatch,
                "analysis snapshot identity mismatch",
            ),
            (
                AnalysisEngineError::ArithmeticOverflow,
                "analysis evidence count overflow",
            ),
            (
                AnalysisEngineError::SerializationFailure,
                "analysis artifact serialization failed",
            ),
            (
                AnalysisEngineError::LimitExceeded,
                "analysis corpus exceeded its execution bound",
            ),
            (
                AnalysisEngineError::TopicMeasurement(TopicMeasurementError::DidNotConverge),
                "topic estimator did not converge",
            ),
            (
                AnalysisEngineError::ModelSelection(ModelSelectionError::NoSuccessfulFit),
                "no fitted candidate produced a finite diagnostic",
            ),
            (
                AnalysisEngineError::InvalidTopicLineageArtifact,
                "invalid topic lineage artifact",
            ),
        ];
        for (error, message) in messages {
            assert_eq!(error.to_string(), message);
        }
        let converted: AnalysisEngineError = ApiError::InvalidWirePayload.into();
        assert_eq!(converted.to_string(), "invalid API wire payload");
        let from_topic: AnalysisEngineError = TopicMeasurementError::DidNotConverge.into();
        assert_eq!(from_topic.to_string(), "topic estimator did not converge");
        assert_eq!(
            add_membership_count(u64::MAX, 1),
            Err(AnalysisEngineError::ArithmeticOverflow)
        );
        assert_eq!(add_membership_count(0, 4), Ok(4));
    }

    #[test]
    fn malformed_request_receipt_cutoff_and_completion_fail_closed() {
        let corpus = AnalysisCorpus::new(
            "snapshot-1",
            vec![unit(
                "evidence-1",
                "2026-07-01T00:00:00Z",
                "2026-07-01T00:00:00Z",
                1,
            )],
        )
        .expect("corpus");

        let mut invalid_request = request();
        invalid_request.idempotency_key.clear();
        assert_eq!(
            execute_analysis_run(
                &invalid_request,
                &accepted(),
                &corpus,
                "2026-08-03T00:00:00Z"
            ),
            Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
        );

        let mut invalid_accepted = accepted();
        invalid_accepted.run_id.clear();
        assert_eq!(
            execute_analysis_run(
                &request(),
                &invalid_accepted,
                &corpus,
                "2026-08-03T00:00:00Z"
            ),
            Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
        );

        let mut invalid_cutoff = request();
        invalid_cutoff.knowledge_cutoff = "not-a-time".into();
        assert_eq!(
            execute_analysis_run(
                &invalid_cutoff,
                &accepted(),
                &corpus,
                "2026-08-03T00:00:00Z"
            ),
            Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
        );

        assert_eq!(
            execute_analysis_run(&request(), &accepted(), &corpus, "not-a-time"),
            Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
        );

        let no_evidence = AnalysisCorpus::new(
            "snapshot-1",
            vec![unit(
                "late",
                "2026-07-01T00:00:00Z",
                "2026-08-02T00:00:00Z",
                1,
            )],
        )
        .expect("corpus");
        assert_eq!(
            execute_analysis_run(&request(), &accepted(), &no_evidence, "not-a-time"),
            Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
        );
    }
}
