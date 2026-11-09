//! Offline, file-driven execution of the CPU `f64` topic-lineage estimator.
//!
//! One bounded JSON document supplies an analysis-run request, opaque document
//! identities with event and availability clocks, sparse term counts,
//! multiple-membership assignments, typed relations, and the reference model
//! configuration. Documents, memberships, and relations whose availability
//! time is later than the request knowledge cutoff are excluded and counted
//! before fitting; they never reach the estimator. Relation kinds that would
//! assert an identified causal claim are refused, because the fitted topic
//! association does not identify causation.
//!
//! The output carries the digest-bound [`TopicLineageArtifact`], the
//! request-bound terminal result, the input digest, and exclusion counts. It
//! contains opaque identities and fitted statistics only; no source text,
//! credential, or direct identity enters the input or the output contract.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use corpus_split::{CorpusDocument, CorpusSnapshot, cutoff_eligible};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalError, TemporalInterval,
    TemporalPrecision,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult, ApiError};
use topic_measurement::{
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix, TopicMeasurementError,
};
use uuid::Uuid;

use crate::{AnalysisEngineError, TopicLineageArtifact, execute_topic_lineage_run, format_digest};

/// Exact schema identity required in the offline input document.
pub const TOPIC_LINEAGE_CLI_INPUT_SCHEMA_VERSION: &str = "tepp.topic_lineage_cli_input.v1";
/// Exact schema identity emitted in the offline output document.
pub const TOPIC_LINEAGE_CLI_OUTPUT_SCHEMA_VERSION: &str = "tepp.topic_lineage_cli_output.v1";
/// Maximum accepted input size in bytes; reads stop at this limit plus one
/// byte, and oversized input is refused before parsing.
pub const TOPIC_LINEAGE_CLI_INPUT_BYTE_LIMIT: usize = 16 * 1024 * 1024;
/// Maximum number of offered documents, including excluded documents.
pub const TOPIC_LINEAGE_CLI_DOCUMENT_LIMIT: usize = 10_000;
/// Maximum vocabulary size (number of term columns).
pub const TOPIC_LINEAGE_CLI_VOCABULARY_LIMIT: usize = 100_000;
/// Maximum total number of sparse term entries across all offered documents.
pub const TOPIC_LINEAGE_CLI_NONZERO_LIMIT: usize = 1_000_000;
/// Maximum number of offered memberships and, separately, of offered relations.
pub const TOPIC_LINEAGE_CLI_EDGE_LIMIT: usize = 100_000;
/// Maximum number of initialization seeds.
pub const TOPIC_LINEAGE_CLI_SEED_LIMIT: usize = 32;
/// Maximum iteration budget per seed.
pub const TOPIC_LINEAGE_CLI_ITERATION_LIMIT: usize = 100_000;
/// Maximum declared sparse-estimator work, computed as
/// `(nonzeros + vocabulary) * topics * seeds * iterations`.
pub const TOPIC_LINEAGE_CLI_WORK_LIMIT: u128 = 10_000_000_000;
/// Maximum dense prevalence-design cells admitted before allocation.
pub const TOPIC_LINEAGE_CLI_DESIGN_CELL_LIMIT: usize = 25_000_000;
/// Maximum dense-design fitting work admitted before allocation.
pub const TOPIC_LINEAGE_CLI_DESIGN_WORK_LIMIT: u128 = 10_000_000_000;
/// Maximum dense prevalence-design `f64` payload bytes, excluding row headers
/// and other estimator state; this is not a process RSS guarantee.
pub const TOPIC_LINEAGE_CLI_DESIGN_BYTE_LIMIT: u128 =
    topic_measurement::REFERENCE_DESIGN_BYTE_LIMIT;

/// Complete offline input document.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CliInput {
    /// Must equal [`TOPIC_LINEAGE_CLI_INPUT_SCHEMA_VERSION`].
    schema_version: String,
    /// Analysis-run request; its cutoff governs every exclusion decision.
    request: AnalysisRunRequest,
    /// Opaque run identity bound into the accepted receipt.
    run_id: String,
    /// Caller-supplied completion system time, so runs are reproducible.
    completed_at: String,
    /// Number of term columns.
    vocabulary_size: usize,
    /// Offered documents in estimator row order.
    documents: Vec<CliDocument>,
    /// Offered document-to-group membership assignments.
    memberships: Vec<CliMembership>,
    /// Offered typed relations between documents.
    relations: Vec<CliRelation>,
    /// Reference estimator configuration.
    model: CliModel,
}

/// One offered document with sparse term counts.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CliDocument {
    /// Opaque analytical document identity.
    document_id: Uuid,
    /// Event-valid time used by the estimator.
    event_time: String,
    /// Time the document became available as evidence.
    available_time: String,
    /// Strictly increasing term column indices.
    term_indices: Vec<usize>,
    /// Nonnegative finite counts aligned with `term_indices`.
    term_counts: Vec<f64>,
}

/// One offered membership of a document in a group context.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CliMembership {
    /// Member document identity.
    document_id: Uuid,
    /// Opaque group identity.
    group_id: Uuid,
    /// Closed membership role.
    role: MembershipRole,
    /// Finite nonnegative membership weight.
    weight: f64,
    /// Inclusive event-time start of validity.
    valid_from: String,
    /// Inclusive event-time end of validity.
    valid_to: String,
    /// Time the membership evidence became available.
    available_time: String,
}

/// One offered typed relation between two documents.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CliRelation {
    /// Source document identity.
    source_document_id: Uuid,
    /// Target document identity.
    target_document_id: Uuid,
    /// Closed relation kind.
    kind: RelationKind,
    /// Observed or inferred evidence status.
    evidence_status: RelationEvidenceStatus,
    /// Inclusive start of the source event interval.
    source_event_start: String,
    /// Inclusive end of the source event interval.
    source_event_end: String,
    /// Inclusive start of the target event interval.
    target_event_start: String,
    /// Inclusive end of the target event interval.
    target_event_end: String,
    /// Time the relation evidence became available.
    available_time: String,
}

/// Reference estimator configuration.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CliModel {
    /// Number of global topics `K`.
    topic_count: usize,
    /// Deterministic initialization seeds.
    seeds: Vec<u64>,
    /// Iteration budget per seed.
    maximum_iterations: usize,
    /// Relative convergence tolerance.
    tolerance: f64,
    /// Prevalence prior variance.
    prior_variance: f64,
    /// Relational smoothing strength.
    relation_strength: f64,
    /// Ridge penalty.
    ridge: f64,
    /// Topic-term smoothing.
    topic_smoothing: f64,
    /// Gradient step size.
    step_size: f64,
}

/// Successful offline execution output.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TopicLineageCliOutput {
    /// Exact output schema identity.
    pub schema_version: String,
    /// Lowercase SHA-256 digest of the exact input bytes.
    pub input_sha256: String,
    /// Documents available by the cutoff and modeled by the estimator.
    pub admitted_document_count: u64,
    /// Documents excluded because they were unavailable at the cutoff.
    pub excluded_document_count: u64,
    /// Memberships excluded by the cutoff or by an excluded document.
    pub excluded_membership_count: u64,
    /// Relations excluded by the cutoff or by an excluded endpoint.
    pub excluded_relation_count: u64,
    /// Digest-bound completed topic-lineage artifact.
    pub artifact: TopicLineageArtifact,
    /// Request-bound terminal result carrying the artifact digest.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Fail-closed offline execution error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopicLineageCliError {
    /// The command line did not name exactly one input file.
    Usage,
    /// The input is not a readable regular file, or output writing failed.
    Io,
    /// The input exceeded [`TOPIC_LINEAGE_CLI_INPUT_BYTE_LIMIT`].
    InputTooLarge,
    /// The input was not valid JSON for the closed input schema.
    MalformedInput,
    /// The input declared an unsupported schema version.
    UnsupportedSchemaVersion,
    /// A count, dimension, or declared-work bound was exceeded.
    LimitExceeded,
    /// A timestamp was not a strict RFC 3339 instant.
    InvalidTimestamp,
    /// Two offered documents reused one identity.
    DuplicateDocument,
    /// A document's term indices and counts had different lengths.
    InvalidTermCounts,
    /// A membership or relation referenced an identity absent from the input.
    UnknownDocumentReference,
    /// A membership weight, validity interval, or duplicate key was invalid.
    InvalidMembership,
    /// A relation interval, temporal order, duplicate, or cycle was invalid.
    InvalidRelation,
    /// A relation kind asserted an identified causal claim.
    CausalClaimRefused,
    /// The analysis engine, estimator, or API contract rejected the run.
    Engine(AnalysisEngineError),
}

impl TopicLineageCliError {
    /// Return the stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Usage => "usage",
            Self::Io => "io",
            Self::InputTooLarge => "input_too_large",
            Self::MalformedInput => "malformed_input",
            Self::UnsupportedSchemaVersion => "unsupported_schema_version",
            Self::LimitExceeded => "limit_exceeded",
            Self::InvalidTimestamp => "invalid_timestamp",
            Self::DuplicateDocument => "duplicate_document",
            Self::InvalidTermCounts => "invalid_term_counts",
            Self::UnknownDocumentReference => "unknown_document_reference",
            Self::InvalidMembership => "invalid_membership",
            Self::InvalidRelation => "invalid_relation",
            Self::CausalClaimRefused => "causal_claim_refused",
            Self::Engine(_) => "analysis_engine_rejected",
        }
    }

    /// Return the process exit status: `2` usage, `3` invalid input, `4`
    /// estimator non-convergence or non-finite estimate, and `5` I/O.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::Usage => 2,
            Self::Io => 5,
            Self::Engine(AnalysisEngineError::TopicMeasurement(
                TopicMeasurementError::DidNotConverge | TopicMeasurementError::NonFiniteEstimate,
            )) => 4,
            _ => 3,
        }
    }
}

impl fmt::Display for TopicLineageCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Usage => "usage: tepp-topic-lineage <input.json>",
            Self::Io => "topic-lineage input or output I/O failed",
            Self::InputTooLarge => "topic-lineage input exceeded its byte limit",
            Self::MalformedInput => "malformed topic-lineage input",
            Self::UnsupportedSchemaVersion => "unsupported topic-lineage input schema version",
            Self::LimitExceeded => "topic-lineage input exceeded an execution bound",
            Self::InvalidTimestamp => "invalid topic-lineage timestamp",
            Self::DuplicateDocument => "duplicate topic-lineage document identity",
            Self::InvalidTermCounts => "topic-lineage term indices and counts differ in length",
            Self::UnknownDocumentReference => "topic-lineage reference to an unknown document",
            Self::InvalidMembership => "invalid topic-lineage membership",
            Self::InvalidRelation => "invalid topic-lineage relation",
            Self::CausalClaimRefused => {
                "identified causal relation kinds are refused by the association estimator"
            }
            Self::Engine(error) => return error.fmt(formatter),
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TopicLineageCliError {}

impl From<AnalysisEngineError> for TopicLineageCliError {
    fn from(error: AnalysisEngineError) -> Self {
        Self::Engine(error)
    }
}

impl From<TopicMeasurementError> for TopicLineageCliError {
    fn from(error: TopicMeasurementError) -> Self {
        Self::Engine(AnalysisEngineError::TopicMeasurement(error))
    }
}

impl From<ApiError> for TopicLineageCliError {
    fn from(error: ApiError) -> Self {
        Self::Engine(AnalysisEngineError::Api(error))
    }
}

/// Offered documents after cutoff admission.
struct AdmittedCorpus {
    /// Eligible-document snapshot consumed by the estimator.
    snapshot: CorpusSnapshot,
    /// Admitted identities in estimator row order.
    ids: Vec<Uuid>,
    /// Admitted event times aligned with `ids`.
    event_times: Vec<EventTime>,
    /// Admitted term-count rows.
    counts: SparseMatrix,
    /// Every offered identity with its event time and admission decision.
    offered: BTreeMap<Uuid, (EventTime, bool)>,
}

impl AdmittedCorpus {
    fn offered(&self, document_id: Uuid) -> Result<(EventTime, bool), TopicLineageCliError> {
        self.offered
            .get(&document_id)
            .copied()
            .ok_or(TopicLineageCliError::UnknownDocumentReference)
    }
}

/// Execute one offline topic-lineage run from exact input bytes.
///
/// # Errors
///
/// Returns a fail-closed [`TopicLineageCliError`] for oversized, malformed,
/// unsupported, out-of-bound, temporally invalid, duplicate, dangling, causal,
/// or estimator-rejected input. No partial artifact is produced.
pub fn run_topic_lineage(input: &[u8]) -> Result<TopicLineageCliOutput, TopicLineageCliError> {
    if input.len() > TOPIC_LINEAGE_CLI_INPUT_BYTE_LIMIT {
        return Err(TopicLineageCliError::InputTooLarge);
    }
    let parsed: CliInput =
        serde_json::from_slice(input).map_err(|_| TopicLineageCliError::MalformedInput)?;
    if parsed.schema_version != TOPIC_LINEAGE_CLI_INPUT_SCHEMA_VERSION {
        return Err(TopicLineageCliError::UnsupportedSchemaVersion);
    }
    check_limits(&parsed)?;
    let cutoff = parse_clock(
        &parsed.request.knowledge_cutoff,
        KnowledgeCutoff::parse_rfc3339,
    )?;
    let corpus = admit_documents(&parsed, cutoff)?;
    let (memberships, excluded_membership_count) =
        admit_memberships(&parsed.memberships, &corpus, cutoff)?;
    let (relations, excluded_relation_count) = admit_relations(&parsed.relations, &corpus, cutoff)?;
    check_dense_design_limits(
        corpus.ids.len(),
        &corpus.ids,
        &corpus.event_times,
        &memberships,
        parsed.vocabulary_size,
        &parsed.model,
    )?;
    let model_input = ReferenceTopicInput::new(
        &corpus.snapshot,
        corpus.ids.clone(),
        &corpus.counts,
        &corpus.event_times,
        None,
        &memberships,
        &relations,
    )?;
    let model = &parsed.model;
    let config = ReferenceTopicModelConfig::new(
        model.topic_count,
        model.seeds.clone(),
        model.maximum_iterations,
        model.tolerance,
    )?
    .with_hyperparameters(
        model.prior_variance,
        model.relation_strength,
        model.ridge,
        model.topic_smoothing,
        model.step_size,
    )?;
    let accepted = AnalysisRunAccepted::new(
        parsed.run_id.as_str(),
        "accepted",
        parsed.request.idempotency_key.as_str(),
    )?;
    let execution = execute_topic_lineage_run(
        &parsed.request,
        &accepted,
        &parsed.request.snapshot_id,
        cutoff,
        &model_input,
        &config,
        parsed.completed_at.as_str(),
    )?;
    let admitted_document_count = corpus.ids.len() as u64;
    Ok(TopicLineageCliOutput {
        schema_version: TOPIC_LINEAGE_CLI_OUTPUT_SCHEMA_VERSION.to_owned(),
        input_sha256: format_digest(Sha256::digest(input)),
        admitted_document_count,
        excluded_document_count: parsed.documents.len() as u64 - admitted_document_count,
        excluded_membership_count,
        excluded_relation_count,
        artifact: execution.artifact,
        terminal_result: execution.terminal_result,
    })
}

/// Run the command-line contract: read one named file, write one JSON output
/// line on success, or one JSON error line to `diagnostics` on failure.
///
/// Diagnostics contain only a stable error code and a fixed message; input
/// content is never echoed. A failure to write diagnostics does not change
/// the returned exit status.
pub fn run_topic_lineage_cli<I, O, D>(arguments: I, output: &mut O, diagnostics: &mut D) -> u8
where
    I: IntoIterator<Item = OsString>,
    O: Write,
    D: Write,
{
    let result = read_input_file(arguments)
        .and_then(|bytes| run_topic_lineage(&bytes))
        .and_then(|value| {
            serde_json::to_string(&value)
                .map_err(|_| AnalysisEngineError::SerializationFailure.into())
        })
        .and_then(|json| writeln!(output, "{json}").map_err(|_| TopicLineageCliError::Io));
    match result {
        Ok(()) => 0,
        Err(error) => {
            let diagnostic = serde_json::json!({
                "error_code": error.code(),
                "message": error.to_string(),
            });
            let _ = writeln!(diagnostics, "{diagnostic}");
            error.exit_code()
        }
    }
}

fn read_input_file<I>(arguments: I) -> Result<Vec<u8>, TopicLineageCliError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut arguments = arguments.into_iter();
    let (Some(path), None) = (arguments.next(), arguments.next()) else {
        return Err(TopicLineageCliError::Usage);
    };
    // Refuse directories, devices, sockets, and FIFOs before opening them.
    // Regular-file symlinks remain supported; verify the opened object too.
    let metadata = std::fs::metadata(&path).map_err(|_| TopicLineageCliError::Io)?;
    if !metadata.is_file() {
        return Err(TopicLineageCliError::Io);
    }
    let mut options = OpenOptions::new();
    options.read(true);
    // A pathname can become a FIFO after metadata admission. On Unix the
    // nonblocking open prevents waiting for a writer during that race.
    #[cfg(unix)]
    options.custom_flags(libc::O_NONBLOCK);
    let file = options.open(path).map_err(|_| TopicLineageCliError::Io)?;
    let metadata = file.metadata().map_err(|_| TopicLineageCliError::Io)?;
    if !metadata.is_file() {
        return Err(TopicLineageCliError::Io);
    }
    let mut bytes = Vec::new();
    file.take(TOPIC_LINEAGE_CLI_INPUT_BYTE_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| TopicLineageCliError::Io)?;
    Ok(bytes)
}

#[cfg(test)]
#[path = "topic_lineage_cli_resource_tests.rs"]
mod resource_tests;

fn check_limits(input: &CliInput) -> Result<(), TopicLineageCliError> {
    let nonzeros: usize = input
        .documents
        .iter()
        .map(|document| document.term_indices.len())
        .sum();
    let model = &input.model;
    let within_bounds = input.documents.len() <= TOPIC_LINEAGE_CLI_DOCUMENT_LIMIT
        && input.vocabulary_size <= TOPIC_LINEAGE_CLI_VOCABULARY_LIMIT
        && nonzeros <= TOPIC_LINEAGE_CLI_NONZERO_LIMIT
        && input.memberships.len() <= TOPIC_LINEAGE_CLI_EDGE_LIMIT
        && input.relations.len() <= TOPIC_LINEAGE_CLI_EDGE_LIMIT
        && model.seeds.len() <= TOPIC_LINEAGE_CLI_SEED_LIMIT
        && model.maximum_iterations <= TOPIC_LINEAGE_CLI_ITERATION_LIMIT
        && model.topic_count <= input.vocabulary_size;
    if !within_bounds {
        return Err(TopicLineageCliError::LimitExceeded);
    }
    check_topic_state_limits(
        input.documents.len(),
        input.vocabulary_size,
        model.topic_count,
        0,
    )?;
    // Every factor is bounded above, so this `u128` product cannot overflow.
    let declared_work = (nonzeros as u128 + input.vocabulary_size as u128)
        * model.topic_count as u128
        * model.seeds.len() as u128
        * model.maximum_iterations as u128;
    if declared_work <= TOPIC_LINEAGE_CLI_WORK_LIMIT {
        Ok(())
    } else {
        Err(TopicLineageCliError::LimitExceeded)
    }
}

/// Refuse dense fitting payload before any topic matrix allocation.
/// `F=0` is the early offered-document check; the admitted-context check
/// includes design and coefficients. Checked arithmetic also rejects overflow.
fn check_topic_state_limits(
    document_count: usize,
    vocabulary_size: usize,
    topic_count: usize,
    feature_count: usize,
) -> Result<(), TopicLineageCliError> {
    topic_measurement::check_reference_fit_limits(
        document_count,
        vocabulary_size,
        topic_count,
        feature_count,
    )
    .map_err(|_| TopicLineageCliError::LimitExceeded)
}

fn check_dense_design_limits(
    document_count: usize,
    document_ids: &[Uuid],
    event_times: &[EventTime],
    memberships: &MembershipNetwork,
    vocabulary_size: usize,
    model: &CliModel,
) -> Result<(), TopicLineageCliError> {
    let document_times: BTreeMap<_, _> = document_ids
        .iter()
        .copied()
        .zip(event_times.iter().copied())
        .collect();
    let mut membership_keys = BTreeSet::new();
    for assignment in memberships.assignments() {
        if document_times
            .get(&assignment.member_id().as_uuid())
            .is_some_and(|time| assignment.is_active_at(*time))
        {
            membership_keys.insert((assignment.role(), assignment.group_id()));
        }
    }
    check_design_limits(
        document_count,
        membership_keys.len(),
        model.topic_count,
        model.seeds.len(),
        model.maximum_iterations,
    )?;
    let feature_count = membership_keys
        .len()
        .checked_add(2)
        .ok_or(TopicLineageCliError::LimitExceeded)?;
    check_topic_state_limits(
        document_count,
        vocabulary_size,
        model.topic_count,
        feature_count,
    )
}

fn check_design_limits(
    document_count: usize,
    membership_context_count: usize,
    topic_count: usize,
    seed_count: usize,
    iteration_count: usize,
) -> Result<(), TopicLineageCliError> {
    let feature_count = 2_usize
        .checked_add(membership_context_count)
        .ok_or(TopicLineageCliError::LimitExceeded)?;
    let design_cells = document_count
        .checked_mul(feature_count)
        .ok_or(TopicLineageCliError::LimitExceeded)?;
    topic_measurement::check_reference_design_limits(document_count, feature_count)
        .map_err(|_| TopicLineageCliError::LimitExceeded)?;
    if design_cells > TOPIC_LINEAGE_CLI_DESIGN_CELL_LIMIT
        || (design_cells as u128) * 8 > TOPIC_LINEAGE_CLI_DESIGN_BYTE_LIMIT
    {
        return Err(TopicLineageCliError::LimitExceeded);
    }
    // Each iteration traverses D*F*(K-1) in prevalence_means and again in
    // update_coefficients. This counts those dense traversals only, not all
    // estimator operations, elapsed time, preprocessing, or peak memory.
    let fitting_work = (design_cells as u128)
        .checked_mul(topic_count.saturating_sub(1) as u128)
        .and_then(|value| value.checked_mul(2))
        .and_then(|value| value.checked_mul(seed_count as u128))
        .and_then(|value| value.checked_mul(iteration_count as u128))
        .ok_or(TopicLineageCliError::LimitExceeded)?;
    if fitting_work > TOPIC_LINEAGE_CLI_DESIGN_WORK_LIMIT {
        Err(TopicLineageCliError::LimitExceeded)
    } else {
        Ok(())
    }
}

fn parse_clock<T>(
    value: &str,
    parse: fn(&str) -> Result<T, TemporalError>,
) -> Result<T, TopicLineageCliError> {
    parse(value).map_err(|_| TopicLineageCliError::InvalidTimestamp)
}

fn admit_documents(
    input: &CliInput,
    cutoff: KnowledgeCutoff,
) -> Result<AdmittedCorpus, TopicLineageCliError> {
    let mut snapshot = CorpusSnapshot::new();
    let mut offered = BTreeMap::new();
    let mut ids = Vec::new();
    let mut event_times = Vec::new();
    let mut offsets = vec![0];
    let mut indices = Vec::new();
    let mut values = Vec::new();
    for document in &input.documents {
        let event_time = parse_clock(&document.event_time, EventTime::parse_rfc3339)?;
        let available_time = parse_clock(&document.available_time, AvailableTime::parse_rfc3339)?;
        if document.term_indices.len() != document.term_counts.len() {
            return Err(TopicLineageCliError::InvalidTermCounts);
        }
        let admitted = cutoff_eligible(&available_time, &cutoff);
        if offered
            .insert(document.document_id, (event_time, admitted))
            .is_some()
        {
            return Err(TopicLineageCliError::DuplicateDocument);
        }
        if admitted {
            snapshot
                .insert_if_eligible(
                    CorpusDocument::new(document.document_id, available_time),
                    &cutoff,
                )
                .map_err(|_| TopicLineageCliError::DuplicateDocument)?;
            ids.push(document.document_id);
            event_times.push(event_time);
            indices.extend_from_slice(&document.term_indices);
            values.extend_from_slice(&document.term_counts);
            offsets.push(indices.len());
        }
    }
    let counts =
        SparseMatrix::from_csr(ids.len(), input.vocabulary_size, offsets, indices, values)?;
    Ok(AdmittedCorpus {
        snapshot,
        ids,
        event_times,
        counts,
        offered,
    })
}

fn admit_memberships(
    items: &[CliMembership],
    corpus: &AdmittedCorpus,
    cutoff: KnowledgeCutoff,
) -> Result<(MembershipNetwork, u64), TopicLineageCliError> {
    let mut network = MembershipNetwork::new();
    let mut excluded = 0_u64;
    for item in items {
        let available_time = parse_clock(&item.available_time, AvailableTime::parse_rfc3339)?;
        let valid_from = parse_clock(&item.valid_from, EventTime::parse_rfc3339)?;
        let valid_to = parse_clock(&item.valid_to, EventTime::parse_rfc3339)?;
        let (_, document_admitted) = corpus.offered(item.document_id)?;
        let weight = MembershipWeight::new(item.weight)
            .map_err(|_| TopicLineageCliError::InvalidMembership)?;
        let assignment = MembershipAssignment::new(
            MemberId::from_uuid(item.document_id),
            GroupId::from_uuid(item.group_id),
            item.role,
            weight,
            valid_from,
            valid_to,
        )
        .map_err(|_| TopicLineageCliError::InvalidMembership)?;
        if document_admitted && cutoff_eligible(&available_time, &cutoff) {
            network
                .insert(assignment)
                .map_err(|_| TopicLineageCliError::InvalidMembership)?;
        } else {
            excluded += 1;
        }
    }
    Ok((network, excluded))
}

fn admit_relations(
    items: &[CliRelation],
    corpus: &AdmittedCorpus,
    cutoff: KnowledgeCutoff,
) -> Result<(RelationGraph, u64), TopicLineageCliError> {
    let mut graph = RelationGraph::new();
    let mut keys = BTreeSet::new();
    let mut excluded = 0_u64;
    for item in items {
        if item.kind.is_identified_causal_claim() {
            return Err(TopicLineageCliError::CausalClaimRefused);
        }
        let available_time = parse_clock(&item.available_time, AvailableTime::parse_rfc3339)?;
        let source_interval = event_interval(&item.source_event_start, &item.source_event_end)?;
        let target_interval = event_interval(&item.target_event_start, &item.target_event_end)?;
        let (source_time, source_admitted) = corpus.offered(item.source_document_id)?;
        let (target_time, target_admitted) = corpus.offered(item.target_document_id)?;
        if (item.kind.is_transition_edge() && source_time > target_time)
            || !source_interval.contains(source_time)
            || !target_interval.contains(target_time)
            || !keys.insert((
                item.source_document_id,
                item.target_document_id,
                item.kind,
                item.evidence_status,
            ))
        {
            return Err(TopicLineageCliError::InvalidRelation);
        }
        let edge = RelationEdge::new(
            item.kind,
            RelationEndpointId::from_uuid(item.source_document_id),
            RelationEndpointId::from_uuid(item.target_document_id),
            item.evidence_status,
            source_interval,
            target_interval,
        )
        .map_err(|_| TopicLineageCliError::InvalidRelation)?;
        if source_admitted && target_admitted && cutoff_eligible(&available_time, &cutoff) {
            graph
                .insert(edge)
                .map_err(|_| TopicLineageCliError::InvalidRelation)?;
        } else {
            excluded += 1;
        }
    }
    Ok((graph, excluded))
}

fn event_interval(
    start: &str,
    end: &str,
) -> Result<TemporalInterval<EventTime>, TopicLineageCliError> {
    let start = parse_clock(start, EventTime::parse_rfc3339)?;
    let end = parse_clock(end, EventTime::parse_rfc3339)?;
    TemporalInterval::bounded(
        TemporalBoundary::Included(start),
        TemporalBoundary::Included(end),
        TemporalPrecision::Second,
    )
    .map_err(|_| TopicLineageCliError::InvalidRelation)
}

#[cfg(test)]
mod tests {
    use super::{
        CliDocument, CliInput, CliModel, TOPIC_LINEAGE_CLI_DOCUMENT_LIMIT,
        TOPIC_LINEAGE_CLI_EDGE_LIMIT, TOPIC_LINEAGE_CLI_ITERATION_LIMIT,
        TOPIC_LINEAGE_CLI_NONZERO_LIMIT, TOPIC_LINEAGE_CLI_SEED_LIMIT,
        TOPIC_LINEAGE_CLI_VOCABULARY_LIMIT, TopicLineageCliError, check_design_limits,
        check_limits, check_topic_state_limits,
    };
    use tepp_api::AnalysisRunRequest;
    use uuid::Uuid;

    fn document(nonzeros: usize) -> CliDocument {
        CliDocument {
            document_id: Uuid::nil(),
            event_time: String::new(),
            available_time: String::new(),
            term_indices: vec![0; nonzeros],
            term_counts: Vec::new(),
        }
    }

    fn input() -> CliInput {
        CliInput {
            schema_version: String::new(),
            request: AnalysisRunRequest {
                contract_version: 1,
                idempotency_key: String::new(),
                tenant_workspace_id: String::new(),
                snapshot_id: String::new(),
                knowledge_cutoff: String::new(),
                model_contract_version: String::new(),
                output_profile: String::new(),
            },
            run_id: String::new(),
            completed_at: String::new(),
            vocabulary_size: 4,
            documents: vec![document(2), document(2)],
            memberships: Vec::new(),
            relations: Vec::new(),
            model: CliModel {
                topic_count: 2,
                seeds: vec![1, 2],
                maximum_iterations: 10,
                tolerance: 1e-5,
                prior_variance: 1.0,
                relation_strength: 0.5,
                ridge: 0.01,
                topic_smoothing: 0.05,
                step_size: 0.2,
            },
        }
    }

    #[test]
    fn dense_topic_payload_boundary_and_overflow_are_dimension_only() {
        // 8 * (3*64*87375 + 6*2*64 + 2*3*63 + 2*3 + 64) = 128 MiB.
        assert_eq!(check_topic_state_limits(2, 87_375, 64, 3), Ok(()));
        for dimensions in [
            (2, 87_376, 64, 3),
            (3, 87_375, 64, 3),
            (2, 87_375, 65, 3),
            (2, 87_375, 64, 4),
            (usize::MAX, usize::MAX, usize::MAX, usize::MAX),
        ] {
            assert_eq!(
                check_topic_state_limits(dimensions.0, dimensions.1, dimensions.2, dimensions.3),
                Err(TopicLineageCliError::LimitExceeded)
            );
        }
        assert_eq!(check_topic_state_limits(4, 4, 2, 3), Ok(()));
    }

    #[test]
    fn dense_topic_vocabulary_is_refused_without_allocating_matrices() {
        let mut value = input();
        value.vocabulary_size = 100_000;
        value.model.topic_count = 49_998;
        value.model.seeds = vec![1];
        value.model.maximum_iterations = 2;
        assert_eq!(
            check_limits(&value),
            Err(TopicLineageCliError::LimitExceeded)
        );
    }

    #[test]
    fn dense_design_limits_fail_closed_before_allocation() {
        assert_eq!(
            check_design_limits(10_000, 3_000, 2, 1, 4),
            Err(TopicLineageCliError::LimitExceeded)
        );
        assert_eq!(check_design_limits(4, 2, 2, 1, 4), Ok(()));
        assert_eq!(check_design_limits(4, 50_000, 2, 1, 4), Ok(()));
        assert_eq!(
            check_design_limits(4, 50_000, 2, 32, 100_000),
            Err(TopicLineageCliError::LimitExceeded)
        );
    }
    #[test]
    fn simultaneous_maximum_dimensions_are_refused_before_work_arithmetic() {
        assert_eq!(check_limits(&input()), Ok(()));
        let mut value = input();
        value.vocabulary_size = usize::MAX;
        value.model.topic_count = usize::MAX;
        value.model.maximum_iterations = usize::MAX;
        assert_eq!(
            check_limits(&value),
            Err(TopicLineageCliError::LimitExceeded)
        );
    }

    #[test]
    fn every_execution_bound_fails_closed_one_past_its_limit() {
        type InputMutation = Box<dyn Fn(&mut CliInput)>;
        assert_eq!(check_limits(&input()), Ok(()));
        let empty_membership = || {
            serde_json::from_str(
            r#"{"document_id":"00000000-0000-0000-0000-000000000000","group_id":"00000000-0000-0000-0000-000000000000","role":"project","weight":1.0,"valid_from":"","valid_to":"","available_time":""}"#,
        )
        .expect("membership")
        };
        let empty_relation = || {
            serde_json::from_str(
            r#"{"source_document_id":"00000000-0000-0000-0000-000000000000","target_document_id":"00000000-0000-0000-0000-000000000000","kind":"transitions_to","evidence_status":"observed","source_event_start":"","source_event_end":"","target_event_start":"","target_event_end":"","available_time":""}"#,
        )
        .expect("relation")
        };
        let mutations: Vec<InputMutation> = vec![
            Box::new(|value| {
                value.documents = (0..=TOPIC_LINEAGE_CLI_DOCUMENT_LIMIT)
                    .map(|_| document(0))
                    .collect();
            }),
            Box::new(|value| value.vocabulary_size = TOPIC_LINEAGE_CLI_VOCABULARY_LIMIT + 1),
            Box::new(|value| value.documents = vec![document(TOPIC_LINEAGE_CLI_NONZERO_LIMIT + 1)]),
            Box::new(move |value| {
                value.memberships = (0..=TOPIC_LINEAGE_CLI_EDGE_LIMIT)
                    .map(|_| empty_membership())
                    .collect();
            }),
            Box::new(move |value| {
                value.relations = (0..=TOPIC_LINEAGE_CLI_EDGE_LIMIT)
                    .map(|_| empty_relation())
                    .collect();
            }),
            Box::new(|value| value.model.seeds = vec![1; TOPIC_LINEAGE_CLI_SEED_LIMIT + 1]),
            Box::new(|value| {
                value.model.maximum_iterations = TOPIC_LINEAGE_CLI_ITERATION_LIMIT + 1;
            }),
            Box::new(|value| value.model.topic_count = 5),
            Box::new(|value| {
                value.vocabulary_size = TOPIC_LINEAGE_CLI_VOCABULARY_LIMIT;
                value.model.topic_count = TOPIC_LINEAGE_CLI_VOCABULARY_LIMIT;
                value.model.maximum_iterations = TOPIC_LINEAGE_CLI_ITERATION_LIMIT;
            }),
        ];
        for mutate in mutations {
            let mut value = input();
            mutate(&mut value);
            assert_eq!(
                check_limits(&value),
                Err(TopicLineageCliError::LimitExceeded)
            );
        }
    }
}
