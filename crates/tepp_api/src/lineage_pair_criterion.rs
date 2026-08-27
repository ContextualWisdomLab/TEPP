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

#[cfg(test)]
mod branch_coverage_tests {
    use super::{
        LineageAnchorBasis, LineageComputeReceipt, LineageComputeReceipts,
        LineageDrawProvenance, LineagePairCriterionPosterior, LineagePairCriterionPosteriorArtifact,
        LineageTemporalProvenance, ApiError, digest, identifier, valid_accelerator_backend,
        valid_pair, valid_receipt, valid_receipts, valid_temporal_provenance,
    };

    fn receipt(backend: &str) -> LineageComputeReceipt {
        LineageComputeReceipt {
            backend_code: backend.into(),
            execution_environment_code: if backend == "mlx_metal_macos_native" {
                "macos_native".into()
            } else {
                "linux_container".into()
            },
            objective_sha256: "a".repeat(64),
            parameter_sha256: "b".repeat(64),
            draw_sha256: "c".repeat(64),
            observed_maximum_difference: if backend == "rust_cpu" { 0.0 } else { 5.0e-9 },
        }
    }

    fn receipts() -> LineageComputeReceipts {
        LineageComputeReceipts {
            cpu: receipt("rust_cpu"),
            gpu: receipt("mlx_metal_macos_native"),
            parity_method_code: "producer_method_derived_v1".into(),
            parity_bound: 1.0e-8,
        }
    }

    fn posterior() -> LineagePairCriterionPosterior {
        LineagePairCriterionPosterior {
            pair_id: "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b3".into(),
            predecessor_record_id: "record-a".into(),
            successor_record_id: "record-b".into(),
            predecessor_record_created_at: "2026-01-03T00:00:00Z".into(),
            predecessor_available_at: "2026-01-04T00:00:00Z".into(),
            successor_record_created_at: "2026-01-01T00:00:00Z".into(),
            successor_available_at: "2026-01-05T00:00:00Z".into(),
            predecessor_event_time_draws: vec![
                "2025-12-01T00:00:00Z".into(),
                "2025-12-02T00:00:00Z".into(),
            ],
            successor_event_time_draws: vec![
                "2025-12-10T00:00:00Z".into(),
                "2025-12-11T00:00:00Z".into(),
            ],
            criterion_draws: vec![0.35, 0.65],
        }
    }

    fn artifact() -> LineagePairCriterionPosteriorArtifact {
        LineagePairCriterionPosteriorArtifact {
            schema_version: "tepp.lineage_pair_criterion_posterior.v1".into(),
            estimation_run_id: "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b1".into(),
            tepp_run_id: "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b2".into(),
            source_snapshot_sha256: "d".repeat(64),
            knowledge_cutoff: "2026-08-25T00:00:00Z".into(),
            channel_codes: vec!["temporal".into(), "text".into()],
            admitted_pair_ids: vec!["018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b3".into()],
            anchor_basis: LineageAnchorBasis {
                basis_id: "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b4".into(),
                basis_sha256: "e".repeat(64),
                alignment_status: "unique".into(),
                tie_count: 0,
            },
            temporal_provenance: LineageTemporalProvenance {
                method_code: "TDT_CHRONOS_JOINT".into(),
                configuration_sha256: "f".repeat(64),
                event_clock_code: "event_valid_time".into(),
                temporal_dependency_sha256: "1".repeat(64),
                branch_transition_sha256: "2".repeat(64),
            },
            draw_provenance: LineageDrawProvenance {
                seed_domain: "independent-lineage-criterion".into(),
                draw_count: 2,
            },
            pair_posteriors: vec![posterior()],
            compute_receipts: receipts(),
        }
    }

    #[test]
    fn identifiers_and_digests_fail_closed_on_each_guard_arm() {
        assert!(!identifier(""));
        assert!(!identifier(&"x".repeat(257)));
        assert!(!identifier(" padded "));
        assert!(!digest(&"X".repeat(64)));
        assert!(!digest(&"a".repeat(63)));
    }

    #[test]
    fn temporal_provenance_rejects_each_guard_arm() {
        let mut base = LineageTemporalProvenance {
            method_code: "TDT_CHRONOS_JOINT".into(),
            configuration_sha256: "f".repeat(64),
            event_clock_code: "event_valid_time".into(),
            temporal_dependency_sha256: "1".repeat(64),
            branch_transition_sha256: "2".repeat(64),
        };
        assert!(valid_temporal_provenance(&base));
        base.method_code = "garbage".into();
        assert!(!valid_temporal_provenance(&base));
        base.method_code = "TDT".into();
        base.configuration_sha256 = "z".into();
        assert!(!valid_temporal_provenance(&base));
        base.configuration_sha256 = "f".repeat(64);
        base.event_clock_code = "  a  ".into();
        assert!(!valid_temporal_provenance(&base));
        base.event_clock_code = "event_valid_time".into();
        base.temporal_dependency_sha256 = "q".into();
        assert!(!valid_temporal_provenance(&base));
    }

    #[test]
    fn receipts_reject_each_guard_arm_and_backend_rules() {
        assert!(valid_receipts(&receipts()));
        assert!(valid_accelerator_backend(&receipt("mlx_cpu")));
        assert!(valid_accelerator_backend(&receipt("mlx_cuda")));
        assert!(valid_accelerator_backend(&receipt("rust_opencl")));

        let mut wrong_env = receipt("mlx_cpu");
        wrong_env.execution_environment_code = "macos_native".into();
        assert!(!valid_accelerator_backend(&wrong_env));

        let mut unknown = receipt("garbage");
        unknown.observed_maximum_difference = 0.0;
        assert!(!valid_accelerator_backend(&unknown));

        let mut bad_reference = receipt("rust_cpu");
        bad_reference.observed_maximum_difference = 1.0e-9;
        assert!(!valid_receipts(&LineageComputeReceipts {
            cpu: bad_reference,
            gpu: receipt("mlx_metal_macos_native"),
            parity_method_code: "producer_method_derived_v1".into(),
            parity_bound: 1.0e-8,
        }));

        let mut nan = receipt("rust_cpu");
        nan.observed_maximum_difference = f64::NAN;
        assert!(!valid_receipt(&nan));

        let mut divergent = receipts();
        divergent.gpu.observed_maximum_difference = 1.0e-7;
        assert!(!valid_receipts(&divergent));

        let mut mismatched_objective = receipts();
        mismatched_objective.gpu.objective_sha256 = "c".repeat(64);
        assert!(!valid_receipts(&mismatched_objective));
        let mut bad_parity = receipts();
        bad_parity.parity_method_code = "  whitespace  ".into();
        assert!(!valid_receipts(&bad_parity));

        let mut nan_parity = receipts();
        nan_parity.parity_bound = f64::NAN;
        assert!(!valid_receipts(&nan_parity));

        let mut zero_parity = receipts();
        zero_parity.parity_bound = 0.0;
        assert!(!valid_receipts(&zero_parity));

        let mut foreign_cpu = receipts();
        foreign_cpu.cpu.backend_code = "aggregator_cpu".into();
        assert!(!valid_receipts(&foreign_cpu));

        let mut missing_cpu_backend = receipts();
        missing_cpu_backend.cpu.backend_code = "".into();
        assert!(!valid_receipts(&missing_cpu_backend));

        let mut dirty_gpu_env = receipts();
        dirty_gpu_env.gpu.execution_environment_code = "host".into();
        assert!(!valid_receipts(&dirty_gpu_env));

        let mut empty_env = receipts();
        empty_env.gpu.execution_environment_code = "".into();
        assert!(!valid_receipts(&empty_env));

        let mut short_objective = receipts();
        short_objective.gpu.objective_sha256 = "z".into();
        assert!(!valid_receipt(&short_objective.gpu));

        let mut short_parameter = receipts();
        short_parameter.gpu.parameter_sha256 = "z".into();
        assert!(!valid_receipt(&short_parameter.gpu));

        let mut short_draw = receipts();
        short_draw.gpu.draw_sha256 = "z".into();
        assert!(!valid_receipt(&short_draw.gpu));
    }

    #[test]
    fn pair_validator_rejects_each_clause() {
        let cutoff = "2026-08-25T00:00:00Z".parse().expect("timestamp");

        let mut nonuuid = posterior();
        nonuuid.pair_id = "not-a-uuid".into();
        assert!(!valid_pair(&nonuuid, 2, cutoff));

        let mut same = posterior();
        same.successor_record_id = "record-a".into();
        assert!(!valid_pair(&same, 2, cutoff));

        let mut unavailable = posterior();
        unavailable.predecessor_record_created_at = "bad".into();
        assert!(!valid_pair(&unavailable, 2, cutoff));

        let mut future = posterior();
        future.successor_available_at = "2026-12-01T00:00:00Z".into();
        assert!(!valid_pair(&future, 2, cutoff));

        let mut short = posterior();
        short.criterion_draws = vec![0.5];
        assert!(!valid_pair(&short, 2, cutoff));

        let mut unparsed_draw = posterior();
        unparsed_draw.predecessor_event_time_draws[0] = "not-a-time".into();
        assert!(!valid_pair(&unparsed_draw, 2, cutoff));

        let mut reversed = posterior();
        reversed.predecessor_event_time_draws[0] = "2025-12-20T00:00:00Z".into();
        assert!(!valid_pair(&reversed, 2, cutoff));

        let mut out_of_range = posterior();
        out_of_range.criterion_draws[0] = 1.5;
        assert!(!valid_pair(&out_of_range, 2, cutoff));

        let mut empty_records = posterior();
        empty_records.predecessor_record_id = "".into();
        assert!(!valid_pair(&empty_records, 2, cutoff));

        let mut padded_records = posterior();
        padded_records.successor_record_id = " padded ".into();
        assert!(!valid_pair(&padded_records, 2, cutoff));

        let mut unparsed_created = posterior();
        unparsed_created.successor_record_created_at = "not-a-time".into();
        assert!(!valid_pair(&unparsed_created, 2, cutoff));

        let mut unparsed_available = posterior();
        unparsed_available.predecessor_available_at = "not-a-time".into();
        assert!(!valid_pair(&unparsed_available, 2, cutoff));

        let mut short_draws = posterior();
        short_draws.successor_event_time_draws.pop();
        assert!(!valid_pair(&short_draws, 2, cutoff));

        let mut nan_criterion = posterior();
        nan_criterion.criterion_draws = vec![f64::NAN, 0.5];
        assert!(!valid_pair(&nan_criterion, 2, cutoff));

        let mut short_first_draws = posterior();
        short_first_draws.predecessor_event_time_draws.pop();
        assert!(!valid_pair(&short_first_draws, 2, cutoff));

        let mut short_second_draws = posterior();
        short_second_draws.successor_event_time_draws.pop();
        assert!(!valid_pair(&short_second_draws, 2, cutoff));

        let mut invalid_created = posterior();
        invalid_created.predecessor_record_created_at = "not-a-time".into();
        assert!(!valid_pair(&invalid_created, 2, cutoff));
    }

    #[test]
    fn artifact_validator_rejects_each_remaining_clause() {
        let mut duplicate_channels = artifact();
        duplicate_channels.channel_codes.push("temporal".into());
        assert_eq!(duplicate_channels.to_json(), Err(ApiError::InvalidWirePayload));

        let mut extra_posterior_ids = artifact();
        extra_posterior_ids.admitted_pair_ids = vec![
            "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b3".into(),
            "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b9".into(),
        ];
        assert_eq!(extra_posterior_ids.to_json(), Err(ApiError::InvalidWirePayload));

        let mut duplicated_admission = artifact();
        duplicated_admission.admitted_pair_ids = vec![
            "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b3".into(),
            "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b3".into(),
        ];
        assert_eq!(duplicated_admission.to_json(), Err(ApiError::InvalidWirePayload));

        let mut dropped_pair = artifact();
        dropped_pair.pair_posteriors = vec![];
        assert_eq!(dropped_pair.to_json(), Err(ApiError::InvalidWirePayload));

        let mut dirty_clock = artifact();
        dirty_clock.temporal_provenance.event_clock_code = "  clock  ".into();
        assert_eq!(dirty_clock.to_json(), Err(ApiError::InvalidWirePayload));

        let mut unknown_backend = artifact();
        unknown_backend.compute_receipts.gpu.backend_code = "tp_mixture_mlx".into();
        assert_eq!(unknown_backend.to_json(), Err(ApiError::InvalidWirePayload));

        let mut bad_pair_id = artifact();
        bad_pair_id.pair_posteriors[0].pair_id = "not-a-uuid".into();
        assert_eq!(bad_pair_id.to_json(), Err(ApiError::InvalidWirePayload));
    }
}
