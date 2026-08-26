//! Independent Event Lineage pair-criterion posterior producer contract.
//!
//! The contract transports posterior draws and TDT/CHRONOS temporal
//! provenance without turning record order, lexical overlap, or a locally
//! selected threshold into criterion truth.  Mathematical estimation remains
//! in TEPP scientific crates; this module validates and publishes its artifact.

use std::collections::BTreeSet;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ApiError;
use crate::wire::{from_json, require_byte_limit, to_json_with_limit};

/// Exact schema consumed by fast-mlsirm's Event Lineage weight boundary.
pub const LINEAGE_PAIR_CRITERION_POSTERIOR_SCHEMA: &str =
    "tepp.lineage_pair_criterion_posterior.v2";
/// Maximum serialized posterior artifact size.
pub const DEFAULT_LINEAGE_PAIR_CRITERION_BYTE_LIMIT: usize = 16 * 1024 * 1024;

/// Unique independent anchor basis for criterion interpretation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineageAnchorBasis {
    /// Opaque basis identity.
    pub basis_id: String,
    /// Lowercase SHA-256 of the immutable basis.
    pub basis_sha256: String,
    /// Alignment result; exactly `unique`.
    pub alignment_status: String,
    /// Ambiguous alignment count; exactly zero.
    pub tie_count: u64,
}

/// TDT/CHRONOS temporal inference provenance.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineageTemporalProvenance {
    /// `TDT`, `CHRONOS`, or their joint identified model.
    pub method_code: String,
    /// Immutable model/configuration digest.
    pub configuration_sha256: String,
    /// Producer-owned event-clock identity.
    pub event_clock_code: String,
    /// Digest of admitted temporal dependencies.
    pub temporal_dependency_sha256: String,
    /// Digest of admitted branches and transitions.
    pub branch_transition_sha256: String,
}

/// Posterior draw-generation provenance.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineageDrawProvenance {
    /// Independent deterministic seed domain.
    pub seed_domain: String,
    /// Common posterior draw count.
    pub draw_count: usize,
}

/// CPU or GPU execution receipt over the same fitted posterior.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineageComputeReceipt {
    /// Backend code, exactly `rust_cpu` or `mlx_metal_macos_native` as position requires.
    pub backend_code: String,
    /// Actual execution environment, such as `macos_native` or `linux_container`.
    pub execution_environment_code: String,
    /// Digest of the objective and admitted data.
    pub objective_sha256: String,
    /// Digest of fitted parameters.
    pub parameter_sha256: String,
    /// Digest of posterior draws in canonical pair/draw order.
    pub draw_sha256: String,
    /// Maximum discrepancy from the CPU f64 reference under the parity method.
    pub observed_maximum_difference: f64,
}

/// Method-derived parity evidence, never a consumer-selected tolerance.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineageComputeReceipts {
    /// CPU f64 reference receipt.
    pub cpu: LineageComputeReceipt,
    /// MLX Apple Silicon accelerator execution receipt.
    pub gpu: LineageComputeReceipt,
    /// Published parity method identity.
    pub parity_method_code: String,
    /// Method-derived parity bound.
    pub parity_bound: f64,
}

/// One pair's independent criterion and event-time posterior draws.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineagePairCriterionPosterior {
    /// Opaque pair identity.
    pub pair_id: String,
    /// Opaque predecessor record identity.
    pub predecessor_record_id: String,
    /// Opaque successor record identity.
    pub successor_record_id: String,
    /// Predecessor record creation instant, distinct from event time.
    pub predecessor_record_created_at: String,
    /// Instant at which predecessor evidence became available.
    pub predecessor_available_at: String,
    /// Successor record creation instant, distinct from event time.
    pub successor_record_created_at: String,
    /// Instant at which successor evidence became available.
    pub successor_available_at: String,
    /// Predecessor event-time posterior draws.
    pub predecessor_event_time_draws: Vec<String>,
    /// Successor event-time posterior draws.
    pub successor_event_time_draws: Vec<String>,
    /// Continuous independent-criterion posterior draws in `[0, 1]`.
    pub criterion_draws: Vec<f64>,
}

/// Complete TEPP producer artifact consumed by fast-mlsirm.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineagePairCriterionPosteriorArtifact {
    /// Exact schema identity.
    pub schema_version: String,
    /// Consumer estimation-run identity.
    pub estimation_run_id: String,
    /// TEPP producer run identity.
    pub tepp_run_id: String,
    /// Immutable source snapshot digest.
    pub source_snapshot_sha256: String,
    /// Historical knowledge cutoff.
    pub knowledge_cutoff: String,
    /// Stable pre-fusion channel order.
    pub channel_codes: Vec<String>,
    /// Exact admitted pair identities.
    pub admitted_pair_ids: Vec<String>,
    /// Independent anchor basis.
    pub anchor_basis: LineageAnchorBasis,
    /// TDT/CHRONOS provenance.
    pub temporal_provenance: LineageTemporalProvenance,
    /// Draw provenance.
    pub draw_provenance: LineageDrawProvenance,
    /// Complete pair posterior set.
    pub pair_posteriors: Vec<LineagePairCriterionPosterior>,
    /// CPU/GPU execution and parity receipts.
    pub compute_receipts: LineageComputeReceipts,
}

impl LineagePairCriterionPosteriorArtifact {
    /// Parse and validate a bounded producer artifact.
    ///
    /// # Errors
    ///
    /// Returns a redacted wire error for any missing, mixed, non-finite,
    /// temporally reversed, ambiguous-anchor, or parity-invalid evidence.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        require_byte_limit(payload, DEFAULT_LINEAGE_PAIR_CRITERION_BYTE_LIMIT)?;
        let artifact: Self = from_json(payload)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize this artifact only after full validation.
    ///
    /// # Errors
    ///
    /// Returns a redacted wire or size error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json_with_limit(self, DEFAULT_LINEAGE_PAIR_CRITERION_BYTE_LIMIT)
    }

    fn validate(&self) -> Result<(), ApiError> {
        let draws = self.draw_provenance.draw_count;
        let pair_ids = self
            .pair_posteriors
            .iter()
            .map(|pair| pair.pair_id.as_str())
            .collect::<BTreeSet<_>>();
        let admitted = self
            .admitted_pair_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let channels = self.channel_codes.iter().collect::<BTreeSet<_>>();
        if self.schema_version != LINEAGE_PAIR_CRITERION_POSTERIOR_SCHEMA
            || !canonical_uuid(&self.estimation_run_id)
            || !canonical_uuid(&self.tepp_run_id)
            || !digest(&self.source_snapshot_sha256)
            || parse_time(&self.knowledge_cutoff).is_none()
            || self.channel_codes.is_empty()
            || channels.len() != self.channel_codes.len()
            || !self.channel_codes.iter().all(|value| identifier(value))
            || draws < 2
            || !identifier(&self.draw_provenance.seed_domain)
            || pair_ids != admitted
            || admitted.len() != self.admitted_pair_ids.len()
            || pair_ids.len() != self.pair_posteriors.len()
            || self.anchor_basis.alignment_status != "unique"
            || self.anchor_basis.tie_count != 0
            || !canonical_uuid(&self.anchor_basis.basis_id)
            || !digest(&self.anchor_basis.basis_sha256)
            || !valid_temporal_provenance(&self.temporal_provenance)
            || !valid_receipts(&self.compute_receipts)
        {
            return Err(ApiError::InvalidWirePayload);
        }
        let cutoff = parse_time(&self.knowledge_cutoff).expect("validated cutoff");
        for pair in &self.pair_posteriors {
            if !valid_pair(pair, draws, cutoff) {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        Ok(())
    }
}

fn identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && value.trim() == value
}

fn canonical_uuid(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|parsed| parsed.hyphenated().to_string() == value)
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn parse_time(value: &str) -> Option<Timestamp> {
    value.parse::<Timestamp>().ok()
}

fn valid_temporal_provenance(value: &LineageTemporalProvenance) -> bool {
    matches!(
        value.method_code.as_str(),
        "TDT" | "CHRONOS" | "TDT_CHRONOS_JOINT"
    ) && digest(&value.configuration_sha256)
        && identifier(&value.event_clock_code)
        && digest(&value.temporal_dependency_sha256)
        && digest(&value.branch_transition_sha256)
}

fn valid_receipt(value: &LineageComputeReceipt) -> bool {
    identifier(&value.backend_code)
        && identifier(&value.execution_environment_code)
        && digest(&value.objective_sha256)
        && digest(&value.parameter_sha256)
        && digest(&value.draw_sha256)
        && value.observed_maximum_difference.is_finite()
        && value.observed_maximum_difference >= 0.0
}

fn valid_receipts(value: &LineageComputeReceipts) -> bool {
    valid_receipt(&value.cpu)
        && valid_receipt(&value.gpu)
        && value.cpu.backend_code == "rust_cpu"
        && valid_accelerator_backend(&value.gpu)
        && value.cpu.objective_sha256 == value.gpu.objective_sha256
        && identifier(&value.parity_method_code)
        && value.parity_bound.is_finite()
        && value.parity_bound > 0.0
        && value.cpu.observed_maximum_difference == 0.0
        && value.gpu.observed_maximum_difference <= value.parity_bound
}

fn valid_accelerator_backend(receipt: &LineageComputeReceipt) -> bool {
    match receipt.backend_code.as_str() {
        "mlx_metal_macos_native" => receipt.execution_environment_code == "macos_native",
        "mlx_cpu" | "mlx_cuda" | "rust_opencl" => {
            receipt.execution_environment_code == "linux_container"
        }
        _ => false,
    }
}

fn valid_pair(pair: &LineagePairCriterionPosterior, draws: usize, cutoff: Timestamp) -> bool {
    let predecessor_available_at = parse_time(&pair.predecessor_available_at);
    let successor_available_at = parse_time(&pair.successor_available_at);
    if !canonical_uuid(&pair.pair_id)
        || !identifier(&pair.predecessor_record_id)
        || !identifier(&pair.successor_record_id)
        || pair.predecessor_record_id == pair.successor_record_id
        || parse_time(&pair.predecessor_record_created_at).is_none()
        || parse_time(&pair.successor_record_created_at).is_none()
        || predecessor_available_at.is_none()
        || successor_available_at.is_none()
        || predecessor_available_at.is_some_and(|value| value > cutoff)
        || successor_available_at.is_some_and(|value| value > cutoff)
        || pair.predecessor_event_time_draws.len() != draws
        || pair.successor_event_time_draws.len() != draws
        || pair.criterion_draws.len() != draws
    {
        return false;
    }
    pair.predecessor_event_time_draws
        .iter()
        .zip(&pair.successor_event_time_draws)
        .zip(&pair.criterion_draws)
        .all(|((left, right), criterion)| {
            parse_time(left)
                .zip(parse_time(right))
                .is_some_and(|(left, right)| left <= right)
                && criterion.is_finite()
                && (0.0..=1.0).contains(criterion)
        })
}
