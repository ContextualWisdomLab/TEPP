//! Bind immutable evidence to a durable scientific-acceptance validation run.
//!
//! This is the first GAP-003A slice: cutoff-safe evidence, tenant workspace,
//! output profile, model, seed, backend, precision, and the SE-gate multiplier
//! hash to one durable run identity. The accepted receipt carries no scientific
//! metrics. Completion asks `validation_core` for RMSE, bias, coverage,
//! temporal-order accuracy, and an SE-aware gate, then emits
//! `tepp.scientific_acceptance.v1`. Recovery vectors must be stamped with that
//! same run identity and the pre-registered multiplier; a different run, model,
//! snapshot, seed, tenant, profile, eligible evidence set, or post-hoc `k`
//! fails closed. LLM-authored recovery, non-finite inputs, empty or duplicate
//! evidence, snapshot mismatch, oversized recovery, oversized `k`, and
//! cutoff-empty corpora fail closed. Postgres persistence remains GAP-003B.

use crate::{AnalysisCorpus, AnalysisEngineError, valid_identifier};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunRequest, ApiError};
use validation_core::{
    ValidationError, ValidationReport, accept_within_standard_errors, bias_standard_error,
    interval_coverage, mean_bias, rmse_standard_error, root_mean_square_error,
    temporal_order_accuracy, wilson_coverage_interval,
};

/// Versioned scientific-acceptance artifact schema.
pub const SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION: &str = "tepp.scientific_acceptance.v1";
/// Output profile that selects this validation-run executor.
pub const SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE: &str = "scientific_acceptance_v1";
/// CPU `f64` reference model identity for scientific acceptance.
pub const VALIDATION_CPU_F64_MODEL: &str = "validation_cpu_f64_v1";
/// Backend identity bound into the durable run.
pub const VALIDATION_BACKEND: &str = "cpu";
/// Numeric precision bound into the durable run.
pub const VALIDATION_PRECISION: &str = "f64";
/// Prefix of the hash-stable durable run identity.
pub const VALIDATION_RUN_ID_PREFIX: &str = "tepp-validation-";
/// Hex characters taken from the binding digest for `run_id`.
pub const VALIDATION_RUN_ID_HEX_LEN: usize = 32;
/// Wilson critical value for nominal 95% coverage bounds.
pub const WILSON_Z: f64 = 1.96;
/// Maximum length of one recovery, interval, or event-time vector.
pub const MAX_RECOVERY_VECTOR_LEN: usize = 10_000;
/// Largest finite SE-gate multiplier that may be pre-registered on a run.
///
/// Conventional three-SE gates sit inside this bound. A larger `k` would make
/// `|RMSE| ≤ k · SE(RMSE)` an effectively unlimited post-hoc acceptance rule.
pub const MAX_SE_GATE_K: f64 = 8.0;

/// Durable identity of one submitted validation run. Receipts never carry
/// scientific metrics. Fields are private so callers cannot rewrite the
/// binding or the pre-registered SE-gate multiplier after submit.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ValidationRunReceipt {
    run_id: String,
    binding_sha256: String,
    tenant_workspace_id: String,
    snapshot_id: String,
    knowledge_cutoff: String,
    model: String,
    seed: u64,
    backend: String,
    precision: String,
    output_profile: String,
    eligible_evidence_count: u64,
    se_gate_k: f64,
}

impl ValidationRunReceipt {
    /// Serialize the receipt after confirming it carries no metrics.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::SerializationFailure`] when JSON encoding
    /// fails.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)
    }

    /// Return the durable run identity.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Return the canonical binding digest.
    #[must_use]
    pub fn binding_sha256(&self) -> &str {
        &self.binding_sha256
    }

    /// Return the bound tenant workspace identity.
    #[must_use]
    pub fn tenant_workspace_id(&self) -> &str {
        &self.tenant_workspace_id
    }

    /// Return the bound snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return the bound knowledge cutoff.
    #[must_use]
    pub fn knowledge_cutoff(&self) -> &str {
        &self.knowledge_cutoff
    }

    /// Return the bound model identity.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Return the bound seed.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Return the bound backend.
    #[must_use]
    pub fn backend(&self) -> &str {
        &self.backend
    }

    /// Return the bound precision.
    #[must_use]
    pub fn precision(&self) -> &str {
        &self.precision
    }

    /// Return the bound output profile.
    #[must_use]
    pub fn output_profile(&self) -> &str {
        &self.output_profile
    }

    /// Return the eligible evidence count.
    #[must_use]
    pub const fn eligible_evidence_count(&self) -> u64 {
        self.eligible_evidence_count
    }

    /// Return the pre-registered SE-gate multiplier.
    #[must_use]
    pub const fn se_gate_k(&self) -> f64 {
        self.se_gate_k
    }

    fn identity_record(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{:016x}",
            self.run_id,
            self.binding_sha256,
            self.tenant_workspace_id,
            self.snapshot_id,
            self.knowledge_cutoff,
            self.model,
            self.seed,
            self.backend,
            self.precision,
            self.output_profile,
            self.eligible_evidence_count,
            self.se_gate_k.to_bits()
        )
    }
}

/// Known-truth recovery vectors offered to complete a validation run.
#[derive(Clone, Debug, PartialEq)]
pub struct RecoveryObservation {
    run_id: String,
    binding_sha256: String,
    study_label: String,
    truth: Vec<f64>,
    recovered: Vec<f64>,
    interval_lower: Vec<f64>,
    interval_upper: Vec<f64>,
    truth_times: Vec<f64>,
    recovered_times: Vec<f64>,
    se_gate_k: f64,
    authored_by_llm: bool,
}

impl RecoveryObservation {
    /// Construct a recovery observation stamped to one submitted receipt.
    ///
    /// LLM authorship is recorded here and refused at completion. The SE-gate
    /// multiplier must equal the pre-registered receipt value; a post-hoc `k`
    /// is a binding mismatch. Empty, length-mismatched, or oversized vectors
    /// fail closed immediately.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] for an empty, oversized,
    /// or control-bearing study label or for empty/mismatched vectors,
    /// [`AnalysisEngineError::LimitExceeded`] when any vector exceeds
    /// [`MAX_RECOVERY_VECTOR_LEN`], [`AnalysisEngineError::BindingMismatch`]
    /// when `se_gate_k` differs from the receipt, and
    /// [`AnalysisEngineError::Validation`] for a non-finite or out-of-policy SE
    /// multiplier.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        receipt: &ValidationRunReceipt,
        study_label: impl Into<String>,
        truth: Vec<f64>,
        recovered: Vec<f64>,
        interval_lower: Vec<f64>,
        interval_upper: Vec<f64>,
        truth_times: Vec<f64>,
        recovered_times: Vec<f64>,
        se_gate_k: f64,
        authored_by_llm: bool,
    ) -> Result<Self, AnalysisEngineError> {
        let study_label = study_label.into();
        if !valid_identifier(&study_label) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let lengths = [
            truth.len(),
            recovered.len(),
            interval_lower.len(),
            interval_upper.len(),
            truth_times.len(),
            recovered_times.len(),
        ];
        if lengths.iter().any(|len| *len > MAX_RECOVERY_VECTOR_LEN) {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let n = truth.len();
        if n == 0 || lengths.iter().any(|len| *len != n) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let se_gate_k = require_se_gate_k(se_gate_k)?;
        if se_gate_k.to_bits() != receipt.se_gate_k.to_bits() {
            return Err(AnalysisEngineError::BindingMismatch);
        }
        Ok(Self {
            run_id: receipt.run_id.clone(),
            binding_sha256: receipt.binding_sha256.clone(),
            study_label,
            truth,
            recovered,
            interval_lower,
            interval_upper,
            truth_times,
            recovered_times,
            se_gate_k,
            authored_by_llm,
        })
    }

    /// Return the stamped run identity.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Return the stamped binding digest.
    #[must_use]
    pub fn binding_sha256(&self) -> &str {
        &self.binding_sha256
    }

    /// Return the study label.
    #[must_use]
    pub fn study_label(&self) -> &str {
        &self.study_label
    }

    /// Return the truth vector.
    #[must_use]
    pub fn truth(&self) -> &[f64] {
        &self.truth
    }

    /// Return the recovered vector.
    #[must_use]
    pub fn recovered(&self) -> &[f64] {
        &self.recovered
    }

    /// Return interval lower bounds.
    #[must_use]
    pub fn interval_lower(&self) -> &[f64] {
        &self.interval_lower
    }

    /// Return interval upper bounds.
    #[must_use]
    pub fn interval_upper(&self) -> &[f64] {
        &self.interval_upper
    }

    /// Return truth event times.
    #[must_use]
    pub fn truth_times(&self) -> &[f64] {
        &self.truth_times
    }

    /// Return recovered event times.
    #[must_use]
    pub fn recovered_times(&self) -> &[f64] {
        &self.recovered_times
    }

    /// Return the SE-gate multiplier.
    #[must_use]
    pub const fn se_gate_k(&self) -> f64 {
        self.se_gate_k
    }

    /// Return whether an LLM authored the recovery.
    #[must_use]
    pub const fn authored_by_llm(&self) -> bool {
        self.authored_by_llm
    }

    fn digest_hex(&self) -> String {
        let mut canonical = String::from("tepp.recovery_observation.v1\n");
        let _ = writeln!(canonical, "run_id={}", self.run_id);
        let _ = writeln!(canonical, "binding={}", self.binding_sha256);
        let _ = writeln!(canonical, "study={}", self.study_label);
        let _ = writeln!(canonical, "se_gate_k={:016x}", self.se_gate_k.to_bits());
        let _ = writeln!(canonical, "authored_by_llm={}", self.authored_by_llm);
        append_f64_vector(&mut canonical, "truth", &self.truth);
        append_f64_vector(&mut canonical, "recovered", &self.recovered);
        append_f64_vector(&mut canonical, "interval_lower", &self.interval_lower);
        append_f64_vector(&mut canonical, "interval_upper", &self.interval_upper);
        append_f64_vector(&mut canonical, "truth_times", &self.truth_times);
        append_f64_vector(&mut canonical, "recovered_times", &self.recovered_times);
        format_hex(Sha256::digest(canonical.into_bytes()))
    }
}

/// Operator-usable scientific acceptance evidence for one completed run.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ScientificAcceptanceEvidence {
    schema_version: String,
    run_id: String,
    binding_sha256: String,
    recovery_sha256: String,
    tenant_workspace_id: String,
    snapshot_id: String,
    knowledge_cutoff: String,
    model: String,
    seed: u64,
    backend: String,
    precision: String,
    output_profile: String,
    eligible_evidence_count: u64,
    se_gate_accepted: bool,
    se_gate_k: f64,
    report: ValidationReport,
}

impl ScientificAcceptanceEvidence {
    /// Serialize canonical evidence JSON after report validation.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::Validation`] when a report field is
    /// non-finite, or [`AnalysisEngineError::SerializationFailure`] when JSON
    /// encoding fails.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.report.validate()?;
        serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)
    }

    /// Return the lowercase SHA-256 digest of the canonical evidence JSON.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::to_json`].
    pub fn sha256(&self) -> Result<String, AnalysisEngineError> {
        self.to_json()
            .map(|json| format_hex(Sha256::digest(json.into_bytes())))
    }

    /// Render the nested recovery summary line.
    #[must_use]
    pub fn to_human_summary(&self) -> String {
        self.report.to_human_summary()
    }

    /// Return the artifact schema version.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Return the durable run identity.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Return the scientific binding digest.
    #[must_use]
    pub fn binding_sha256(&self) -> &str {
        &self.binding_sha256
    }

    /// Return the recovery-vector digest stamped into this artifact.
    #[must_use]
    pub fn recovery_sha256(&self) -> &str {
        &self.recovery_sha256
    }

    /// Return the tenant workspace identity.
    #[must_use]
    pub fn tenant_workspace_id(&self) -> &str {
        &self.tenant_workspace_id
    }

    /// Return the snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return the knowledge cutoff.
    #[must_use]
    pub fn knowledge_cutoff(&self) -> &str {
        &self.knowledge_cutoff
    }

    /// Return the bound model identity.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Return the bound seed.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Return the bound backend.
    #[must_use]
    pub fn backend(&self) -> &str {
        &self.backend
    }

    /// Return the bound precision.
    #[must_use]
    pub fn precision(&self) -> &str {
        &self.precision
    }

    /// Return the output profile.
    #[must_use]
    pub fn output_profile(&self) -> &str {
        &self.output_profile
    }

    /// Return the eligible evidence count.
    #[must_use]
    pub const fn eligible_evidence_count(&self) -> u64 {
        self.eligible_evidence_count
    }

    /// Return whether the SE-aware gate accepted RMSE toward 0.
    #[must_use]
    pub const fn se_gate_accepted(&self) -> bool {
        self.se_gate_accepted
    }

    /// Return the SE-gate multiplier used for acceptance.
    #[must_use]
    pub const fn se_gate_k(&self) -> f64 {
        self.se_gate_k
    }
}

struct CanonicalBinding {
    tenant_workspace_id: String,
    snapshot_id: String,
    knowledge_cutoff: String,
    model: String,
    seed: u64,
    backend: String,
    precision: String,
    output_profile: String,
    se_gate_k: f64,
    eligible_ids: Vec<String>,
}

impl CanonicalBinding {
    fn digest_hex(&self) -> String {
        format_hex(Sha256::digest(self.canonical_bytes()))
    }

    fn run_id(digest_hex: &str) -> String {
        let mut run_id =
            String::with_capacity(VALIDATION_RUN_ID_PREFIX.len() + VALIDATION_RUN_ID_HEX_LEN);
        run_id.push_str(VALIDATION_RUN_ID_PREFIX);
        run_id.push_str(&digest_hex[..VALIDATION_RUN_ID_HEX_LEN]);
        run_id
    }

    fn eligible_count(&self) -> u64 {
        self.eligible_ids.len() as u64
    }

    fn canonical_bytes(&self) -> String {
        let mut canonical = String::from("tepp.validation_binding.v1\n");
        let _ = writeln!(canonical, "tenant={}", self.tenant_workspace_id);
        let _ = writeln!(canonical, "snapshot={}", self.snapshot_id);
        let _ = writeln!(canonical, "cutoff={}", self.knowledge_cutoff);
        let _ = writeln!(canonical, "model={}", self.model);
        let _ = writeln!(canonical, "seed={}", self.seed);
        let _ = writeln!(canonical, "backend={}", self.backend);
        let _ = writeln!(canonical, "precision={}", self.precision);
        let _ = writeln!(canonical, "profile={}", self.output_profile);
        let _ = writeln!(canonical, "se_gate_k={:016x}", self.se_gate_k.to_bits());
        for identity in &self.eligible_ids {
            let _ = writeln!(canonical, "evidence={identity}");
        }
        canonical
    }

    fn identity_record(&self, digest_hex: &str, run_id: &str) -> String {
        format!(
            "{run_id}\n{digest_hex}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{:016x}",
            self.tenant_workspace_id,
            self.snapshot_id,
            self.knowledge_cutoff,
            self.model,
            self.seed,
            self.backend,
            self.precision,
            self.output_profile,
            self.eligible_count(),
            self.se_gate_k.to_bits()
        )
    }
}

/// Submit immutable evidence to a durable validation run.
///
/// The receipt is hash-stable for one scientific binding and never includes
/// RMSE, bias, coverage, or gate results.
///
/// # Errors
///
/// Returns a fail-closed engine error for an invalid request, wrong output
/// profile or model, an out-of-policy SE-gate multiplier, snapshot mismatch,
/// duplicate or empty evidence, or a cutoff that admits no evidence.
pub fn submit_validation_run(
    request: &AnalysisRunRequest,
    corpus: &AnalysisCorpus,
    seed: u64,
    se_gate_k: f64,
) -> Result<ValidationRunReceipt, AnalysisEngineError> {
    let binding = bind_validation_run(request, corpus, seed, se_gate_k)?;
    let binding_sha256 = binding.digest_hex();
    let run_id = CanonicalBinding::run_id(&binding_sha256);
    let eligible_evidence_count = binding.eligible_count();
    Ok(ValidationRunReceipt {
        run_id,
        binding_sha256,
        tenant_workspace_id: binding.tenant_workspace_id,
        snapshot_id: binding.snapshot_id,
        knowledge_cutoff: binding.knowledge_cutoff,
        model: binding.model,
        seed: binding.seed,
        backend: binding.backend,
        precision: binding.precision,
        output_profile: binding.output_profile,
        eligible_evidence_count,
        se_gate_k: binding.se_gate_k,
    })
}

/// Complete a previously submitted validation run with known-truth recovery.
///
/// # Errors
///
/// Returns a fail-closed engine error when recovery was LLM-authored, the
/// observation is stamped to a different run identity, the receipt does not
/// match the rebound scientific identity, metric inputs are invalid, or
/// report validation fails.
pub fn complete_validation_run(
    receipt: &ValidationRunReceipt,
    request: &AnalysisRunRequest,
    corpus: &AnalysisCorpus,
    observation: &RecoveryObservation,
) -> Result<ScientificAcceptanceEvidence, AnalysisEngineError> {
    if observation.authored_by_llm {
        return Err(AnalysisEngineError::LlmAuthoredRecovery);
    }
    if observation.run_id != receipt.run_id
        || observation.binding_sha256 != receipt.binding_sha256
        || observation.se_gate_k.to_bits() != receipt.se_gate_k.to_bits()
        || receipt.output_profile != SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::BindingMismatch);
    }
    let binding = bind_validation_run(request, corpus, receipt.seed, receipt.se_gate_k)?;
    let binding_sha256 = binding.digest_hex();
    let run_id = CanonicalBinding::run_id(&binding_sha256);
    if receipt.identity_record() != binding.identity_record(&binding_sha256, &run_id) {
        return Err(AnalysisEngineError::BindingMismatch);
    }
    let report = compute_report(observation)?;
    let se_gate_accepted = accept_within_standard_errors(
        report.rmse,
        0.0,
        report.rmse_standard_error,
        receipt.se_gate_k,
    )?;
    let eligible_evidence_count = binding.eligible_count();
    let evidence = ScientificAcceptanceEvidence {
        schema_version: SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION.to_owned(),
        run_id,
        binding_sha256,
        recovery_sha256: observation.digest_hex(),
        tenant_workspace_id: binding.tenant_workspace_id,
        snapshot_id: binding.snapshot_id,
        knowledge_cutoff: binding.knowledge_cutoff,
        model: binding.model,
        seed: binding.seed,
        backend: binding.backend,
        precision: binding.precision,
        output_profile: SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE.to_owned(),
        eligible_evidence_count,
        se_gate_accepted,
        se_gate_k: receipt.se_gate_k,
        report,
    };
    evidence.to_json()?;
    Ok(evidence)
}

fn require_se_gate_k(se_gate_k: f64) -> Result<f64, AnalysisEngineError> {
    if !se_gate_k.is_finite() {
        return Err(AnalysisEngineError::Validation(
            ValidationError::InvalidInput,
        ));
    }
    if se_gate_k < 0.0 || se_gate_k > MAX_SE_GATE_K {
        return Err(AnalysisEngineError::Validation(
            ValidationError::InvalidConfiguration,
        ));
    }
    Ok(if se_gate_k == 0.0 { 0.0 } else { se_gate_k })
}

fn bind_validation_run(
    request: &AnalysisRunRequest,
    corpus: &AnalysisCorpus,
    seed: u64,
    se_gate_k: f64,
) -> Result<CanonicalBinding, AnalysisEngineError> {
    request.to_json()?;
    let se_gate_k = require_se_gate_k(se_gate_k)?;
    let requested = format!(
        "{}\n{}",
        request.output_profile, request.model_contract_version
    );
    let expected = format!("{SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE}\n{VALIDATION_CPU_F64_MODEL}");
    if requested != expected {
        return Err(AnalysisEngineError::InvalidValidationProfile);
    }
    if request.snapshot_id != corpus.snapshot_id() {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::Api(ApiError::InvalidWirePayload))?;
    let mut identities = BTreeSet::new();
    let mut eligible = BTreeSet::new();
    for unit in corpus.evidence_units() {
        if !identities.insert(unit.evidence_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        if unit.available_time().instant() <= cutoff.instant() {
            eligible.insert(unit.evidence_id().to_owned());
        }
    }
    if identities.is_empty() {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if eligible.is_empty() {
        return Err(AnalysisEngineError::NoEligibleEvidence);
    }
    Ok(CanonicalBinding {
        tenant_workspace_id: request.tenant_workspace_id.clone(),
        snapshot_id: request.snapshot_id.clone(),
        knowledge_cutoff: cutoff.to_rfc3339(),
        model: VALIDATION_CPU_F64_MODEL.to_owned(),
        seed,
        backend: VALIDATION_BACKEND.to_owned(),
        precision: VALIDATION_PRECISION.to_owned(),
        output_profile: SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE.to_owned(),
        se_gate_k,
        eligible_ids: eligible.into_iter().collect(),
    })
}

fn compute_report(
    observation: &RecoveryObservation,
) -> Result<ValidationReport, AnalysisEngineError> {
    let rmse = root_mean_square_error(&observation.truth, &observation.recovered)?;
    let rmse_standard_error = rmse_standard_error(&observation.truth, &observation.recovered)?;
    let mean_bias = mean_bias(&observation.truth, &observation.recovered)?;
    let bias_standard_error = bias_standard_error(&observation.truth, &observation.recovered)?;
    let interval_coverage = interval_coverage(
        &observation.truth,
        &observation.interval_lower,
        &observation.interval_upper,
    )?;
    let (coverage_wilson_lower, coverage_wilson_upper) = wilson_coverage_interval(
        &observation.truth,
        &observation.interval_lower,
        &observation.interval_upper,
        WILSON_Z,
    )?;
    let temporal_order_accuracy =
        temporal_order_accuracy(&observation.truth_times, &observation.recovered_times)?;
    let report = ValidationReport {
        study_label: observation.study_label.clone(),
        rmse,
        rmse_standard_error,
        mean_bias,
        bias_standard_error,
        interval_coverage,
        coverage_wilson_lower,
        coverage_wilson_upper,
        temporal_order_accuracy,
        monte_carlo_rmse: None,
    };
    report.validate()?;
    Ok(report)
}

fn append_f64_vector(canonical: &mut String, label: &str, values: &[f64]) {
    let _ = writeln!(canonical, "{label}_len={}", values.len());
    for value in values {
        let _ = writeln!(canonical, "{label}={:016x}", value.to_bits());
    }
}

fn format_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_RECOVERY_VECTOR_LEN, MAX_SE_GATE_K, RecoveryObservation,
        SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE, SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION,
        VALIDATION_BACKEND, VALIDATION_CPU_F64_MODEL, VALIDATION_PRECISION,
        VALIDATION_RUN_ID_PREFIX, WILSON_Z, complete_validation_run, submit_validation_run,
    };
    use crate::{
        AnalysisCorpus, AnalysisEngineError, AnalysisEvidenceUnit, MAX_ANALYSIS_IDENTIFIER_BYTES,
    };
    use temporal_core::{AvailableTime, EventTime};
    use tepp_api::AnalysisRunRequest;
    use validation_core::ValidationError;

    fn request() -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: 1,
            idempotency_key: "idem-validation-1".into(),
            tenant_workspace_id: "tenant-workspace-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: VALIDATION_CPU_F64_MODEL.into(),
            output_profile: SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE.into(),
        }
    }

    fn unit(id: &str, available: &str) -> AnalysisEvidenceUnit {
        AnalysisEvidenceUnit::new(
            id,
            EventTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("event"),
            AvailableTime::parse_rfc3339(available).expect("available"),
            1,
        )
        .expect("unit")
    }

    fn corpus(units: Vec<AnalysisEvidenceUnit>) -> AnalysisCorpus {
        AnalysisCorpus::new("snapshot-1", units).expect("corpus")
    }

    fn recovery(
        receipt: &super::ValidationRunReceipt,
        truth: Vec<f64>,
        recovered: Vec<f64>,
        k: f64,
    ) -> RecoveryObservation {
        let times = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let times = times[..truth.len()].to_vec();
        let lower: Vec<f64> = truth.iter().map(|value| value - 0.5).collect();
        let upper: Vec<f64> = truth.iter().map(|value| value + 0.5).collect();
        RecoveryObservation::new(
            receipt,
            "foundation-recovery",
            truth,
            recovered,
            lower,
            upper,
            times.clone(),
            times,
            k,
            false,
        )
        .expect("observation")
    }

    fn two_point(
        receipt: &super::ValidationRunReceipt,
        label: impl Into<String>,
        k: f64,
        authored_by_llm: bool,
    ) -> Result<RecoveryObservation, AnalysisEngineError> {
        RecoveryObservation::new(
            receipt,
            label,
            vec![1.0, 2.0],
            vec![1.0, 2.0],
            vec![0.0, 1.0],
            vec![2.0, 3.0],
            vec![1.0, 2.0],
            vec![1.0, 2.0],
            k,
            authored_by_llm,
        )
    }

    #[test]
    fn submit_is_hash_stable_cutoff_safe_and_metric_free() {
        let first = corpus(vec![
            unit("evidence-b", "2026-07-15T00:00:00Z"),
            unit("evidence-a", "2026-07-10T00:00:00Z"),
            unit("late", "2026-08-02T00:00:00Z"),
        ]);
        let second = corpus(vec![
            unit("evidence-a", "2026-07-10T00:00:00Z"),
            unit("evidence-b", "2026-07-15T00:00:00Z"),
        ]);
        let receipt_a = submit_validation_run(&request(), &first, 7, 3.0).expect("submit a");
        let receipt_b = submit_validation_run(&request(), &second, 7, 3.0).expect("submit b");
        assert_eq!(receipt_a.run_id(), receipt_b.run_id());
        assert_eq!(receipt_a.binding_sha256(), receipt_b.binding_sha256());
        assert!(receipt_a.run_id().starts_with(VALIDATION_RUN_ID_PREFIX));
        assert_eq!(
            receipt_a.run_id().len(),
            VALIDATION_RUN_ID_PREFIX.len() + 32
        );
        assert_eq!(receipt_a.eligible_evidence_count(), 2);
        assert_eq!(receipt_a.tenant_workspace_id(), "tenant-workspace-1");
        assert_eq!(receipt_a.snapshot_id(), "snapshot-1");
        assert_eq!(receipt_a.knowledge_cutoff(), "2026-08-01T00:00:00Z");
        assert_eq!(receipt_a.model(), VALIDATION_CPU_F64_MODEL);
        assert_eq!(receipt_a.seed(), 7);
        assert_eq!(receipt_a.backend(), VALIDATION_BACKEND);
        assert_eq!(receipt_a.precision(), VALIDATION_PRECISION);
        assert_eq!(
            receipt_a.output_profile(),
            SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE
        );
        assert!((receipt_a.se_gate_k() - 3.0).abs() < f64::EPSILON);
        let json = receipt_a.to_json().expect("json");
        assert!(!json.contains("rmse"));
        assert!(!json.contains("bias"));
        assert!(!json.contains("coverage"));
        let other_seed = submit_validation_run(&request(), &first, 8, 3.0).expect("seed");
        assert_ne!(receipt_a.run_id(), other_seed.run_id());
        let mut other_tenant = request();
        other_tenant.tenant_workspace_id = "tenant-workspace-2".into();
        let other_tenant_receipt =
            submit_validation_run(&other_tenant, &first, 7, 3.0).expect("tenant");
        assert_ne!(receipt_a.run_id(), other_tenant_receipt.run_id());
        let other_k = submit_validation_run(&request(), &first, 7, 4.0).expect("other k");
        assert_ne!(receipt_a.run_id(), other_k.run_id());
        assert!((other_k.se_gate_k() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn complete_emits_operator_usable_scientific_acceptance_evidence() {
        let offered = corpus(vec![
            unit("evidence-a", "2026-07-10T00:00:00Z"),
            unit("evidence-b", "2026-07-15T00:00:00Z"),
        ]);
        let receipt = submit_validation_run(&request(), &offered, 11, 3.0).expect("submit");
        let observation = recovery(
            &receipt,
            vec![0.70, 0.55, 0.40, -0.20, 0.85],
            vec![0.70, 0.55, 0.40, -0.20, 0.85],
            3.0,
        );
        assert_eq!(observation.run_id(), receipt.run_id());
        assert_eq!(observation.binding_sha256(), receipt.binding_sha256());
        assert_eq!(observation.study_label(), "foundation-recovery");
        assert_eq!(observation.truth().len(), 5);
        assert_eq!(observation.recovered().len(), 5);
        assert_eq!(observation.interval_lower().len(), 5);
        assert_eq!(observation.interval_upper().len(), 5);
        assert_eq!(observation.truth_times().len(), 5);
        assert_eq!(observation.recovered_times().len(), 5);
        assert!((observation.se_gate_k() - 3.0).abs() < f64::EPSILON);
        assert!(!observation.authored_by_llm());
        let evidence = complete_validation_run(&receipt, &request(), &offered, &observation)
            .expect("complete");
        assert_eq!(
            evidence.schema_version(),
            SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION
        );
        assert_eq!(evidence.run_id(), receipt.run_id());
        assert_eq!(evidence.binding_sha256(), receipt.binding_sha256());
        assert_eq!(evidence.recovery_sha256().len(), 64);
        assert_eq!(evidence.tenant_workspace_id(), "tenant-workspace-1");
        assert!(evidence.se_gate_accepted());
        assert_eq!(
            evidence.output_profile(),
            SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE
        );
        assert_eq!(evidence.eligible_evidence_count(), 2);
        assert!((evidence.se_gate_k() - 3.0).abs() < f64::EPSILON);
        assert_eq!(evidence.snapshot_id(), "snapshot-1");
        assert_eq!(evidence.knowledge_cutoff(), "2026-08-01T00:00:00Z");
        assert_eq!(evidence.model(), VALIDATION_CPU_F64_MODEL);
        assert_eq!(evidence.seed(), 11);
        assert_eq!(evidence.backend(), VALIDATION_BACKEND);
        assert_eq!(evidence.precision(), VALIDATION_PRECISION);
        assert!((WILSON_Z - 1.96).abs() < f64::EPSILON);
        let json = evidence.to_json().expect("json");
        assert!(json.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION));
        assert!(json.contains("rmse"));
        assert!(json.contains("recovery_sha256"));
        assert_eq!(evidence.sha256().expect("digest").len(), 64);
        assert!(evidence.to_human_summary().contains("foundation-recovery"));
        let rejected = recovery(
            &receipt,
            vec![0.0, 1.0, 2.0, 3.0, 4.0],
            vec![10.0, 11.0, 12.0, 13.0, 14.0],
            3.0,
        );
        let refused =
            complete_validation_run(&receipt, &request(), &offered, &rejected).expect("refused");
        assert!(!refused.se_gate_accepted());
        assert_ne!(refused.recovery_sha256(), evidence.recovery_sha256());
    }

    #[test]
    fn submit_and_complete_fail_closed_on_trust_and_shape_errors() {
        let offered = corpus(vec![unit("evidence-a", "2026-07-10T00:00:00Z")]);
        let mut wrong_profile = request();
        wrong_profile.output_profile = "validation-report".into();
        assert_eq!(
            submit_validation_run(&wrong_profile, &offered, 1, 3.0),
            Err(AnalysisEngineError::InvalidValidationProfile)
        );
        let mut wrong_model = request();
        wrong_model.model_contract_version = "temporal-evidence-v1".into();
        assert_eq!(
            submit_validation_run(&wrong_model, &offered, 1, 3.0),
            Err(AnalysisEngineError::InvalidValidationProfile)
        );
        let mismatched = AnalysisCorpus::new(
            "snapshot-other",
            vec![unit("evidence-a", "2026-07-10T00:00:00Z")],
        )
        .expect("other");
        assert_eq!(
            submit_validation_run(&request(), &mismatched, 1, 3.0),
            Err(AnalysisEngineError::SnapshotMismatch)
        );
        let duplicate = corpus(vec![
            unit("same", "2026-07-10T00:00:00Z"),
            unit("same", "2026-07-11T00:00:00Z"),
        ]);
        assert_eq!(
            submit_validation_run(&request(), &duplicate, 1, 3.0),
            Err(AnalysisEngineError::DuplicateEvidence)
        );
        let empty = AnalysisCorpus::new("snapshot-1", Vec::new()).expect("empty");
        assert_eq!(
            submit_validation_run(&request(), &empty, 1, 3.0),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        let late = corpus(vec![unit("late", "2026-08-02T00:00:00Z")]);
        assert_eq!(
            submit_validation_run(&request(), &late, 1, 3.0),
            Err(AnalysisEngineError::NoEligibleEvidence)
        );
        let mut invalid_request = request();
        invalid_request.idempotency_key.clear();
        assert!(matches!(
            submit_validation_run(&invalid_request, &offered, 1, 3.0),
            Err(AnalysisEngineError::Api(_))
        ));
        let mut invalid_cutoff = request();
        invalid_cutoff.knowledge_cutoff = "not-a-time".into();
        assert!(matches!(
            submit_validation_run(&invalid_cutoff, &offered, 1, 3.0),
            Err(AnalysisEngineError::Api(_))
        ));
    }

    #[test]
    fn complete_refuses_llm_authored_recovery_and_tampered_receipt() {
        let offered = corpus(vec![
            unit("evidence-a", "2026-07-10T00:00:00Z"),
            unit("evidence-b", "2026-07-15T00:00:00Z"),
        ]);
        let receipt = submit_validation_run(&request(), &offered, 3, 3.0).expect("submit");
        let llm = RecoveryObservation::new(
            &receipt,
            "llm-study",
            vec![1.0, 2.0, 3.0],
            vec![1.0, 2.0, 3.0],
            vec![0.5, 1.5, 2.5],
            vec![1.5, 2.5, 3.5],
            vec![1.0, 2.0, 3.0],
            vec![1.0, 2.0, 3.0],
            3.0,
            true,
        )
        .expect("llm");
        assert!(llm.authored_by_llm());
        assert_eq!(
            complete_validation_run(&receipt, &request(), &offered, &llm),
            Err(AnalysisEngineError::LlmAuthoredRecovery)
        );
        let mut tampered = receipt.clone();
        tampered.run_id = "tepp-validation-deadbeefdeadbeefdeadbeefdeadbeef".into();
        let observation = recovery(&receipt, vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0], 3.0);
        assert_eq!(
            complete_validation_run(&tampered, &request(), &offered, &observation),
            Err(AnalysisEngineError::BindingMismatch)
        );
        let mut profile_tampered = receipt.clone();
        profile_tampered.output_profile = "tampered-profile".into();
        assert_eq!(
            complete_validation_run(&profile_tampered, &request(), &offered, &observation),
            Err(AnalysisEngineError::BindingMismatch)
        );
        let mut k_tampered = observation.clone();
        k_tampered.se_gate_k = 8.0;
        assert_eq!(
            complete_validation_run(&receipt, &request(), &offered, &k_tampered),
            Err(AnalysisEngineError::BindingMismatch)
        );
    }

    #[test]
    fn complete_refuses_recovery_stamped_to_a_different_run() {
        let first = corpus(vec![unit("evidence-a", "2026-07-10T00:00:00Z")]);
        let second = corpus(vec![
            unit("evidence-a", "2026-07-10T00:00:00Z"),
            unit("evidence-b", "2026-07-15T00:00:00Z"),
        ]);
        let receipt_a = submit_validation_run(&request(), &first, 3, 3.0).expect("a");
        let receipt_b = submit_validation_run(&request(), &second, 3, 3.0).expect("b");
        assert_ne!(receipt_a.run_id(), receipt_b.run_id());
        let foreign = recovery(&receipt_a, vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0], 3.0);
        assert_eq!(
            complete_validation_run(&receipt_b, &request(), &second, &foreign),
            Err(AnalysisEngineError::BindingMismatch)
        );
        let other_seed = submit_validation_run(&request(), &first, 9, 3.0).expect("seed");
        assert_eq!(
            complete_validation_run(&other_seed, &request(), &first, &foreign),
            Err(AnalysisEngineError::BindingMismatch)
        );
        let mut other_tenant = request();
        other_tenant.tenant_workspace_id = "tenant-workspace-other".into();
        let other_tenant_receipt =
            submit_validation_run(&other_tenant, &first, 3, 3.0).expect("tenant");
        assert_eq!(
            complete_validation_run(&other_tenant_receipt, &other_tenant, &first, &foreign),
            Err(AnalysisEngineError::BindingMismatch)
        );
    }

    #[test]
    fn complete_refuses_nonfinite_recovery_and_invalid_observation() {
        let offered = corpus(vec![
            unit("evidence-a", "2026-07-10T00:00:00Z"),
            unit("evidence-b", "2026-07-15T00:00:00Z"),
        ]);
        let receipt = submit_validation_run(&request(), &offered, 3, 3.0).expect("submit");
        let nan = RecoveryObservation::new(
            &receipt,
            "nan-study",
            vec![1.0, 2.0, 3.0],
            vec![f64::NAN, 2.0, 3.0],
            vec![0.5, 1.5, 2.5],
            vec![1.5, 2.5, 3.5],
            vec![1.0, 2.0, 3.0],
            vec![1.0, 2.0, 3.0],
            3.0,
            false,
        )
        .expect("nan observation");
        assert_eq!(
            complete_validation_run(&receipt, &request(), &offered, &nan),
            Err(AnalysisEngineError::Validation(
                ValidationError::InvalidInput
            ))
        );
        assert_eq!(
            two_point(&receipt, "", 3.0, false),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            two_point(
                &receipt,
                "x".repeat(MAX_ANALYSIS_IDENTIFIER_BYTES + 1),
                3.0,
                false
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            two_point(&receipt, "nan-k", f64::NAN, false),
            Err(AnalysisEngineError::Validation(
                ValidationError::InvalidInput
            ))
        );
        assert_eq!(
            two_point(&receipt, "neg-k", -1.0, false),
            Err(AnalysisEngineError::Validation(
                ValidationError::InvalidConfiguration
            ))
        );
        let oversized = vec![0.0; MAX_RECOVERY_VECTOR_LEN + 1];
        assert_eq!(
            RecoveryObservation::new(
                &receipt,
                "oversized",
                oversized.clone(),
                oversized.clone(),
                oversized.clone(),
                oversized.clone(),
                oversized.clone(),
                oversized,
                3.0,
                false,
            ),
            Err(AnalysisEngineError::LimitExceeded)
        );
        let converted: AnalysisEngineError = ValidationError::InvalidInput.into();
        assert_eq!(converted.to_string(), "invalid validation input");
    }

    #[test]
    fn se_gate_k_is_pre_registered_and_empty_vectors_fail_closed() {
        let offered = corpus(vec![
            unit("evidence-a", "2026-07-10T00:00:00Z"),
            unit("evidence-b", "2026-07-15T00:00:00Z"),
        ]);
        let receipt = submit_validation_run(&request(), &offered, 3, 3.0).expect("submit");
        assert_eq!(
            two_point(&receipt, "huge-k", MAX_SE_GATE_K + 0.01, false),
            Err(AnalysisEngineError::Validation(
                ValidationError::InvalidConfiguration
            ))
        );
        assert_eq!(
            two_point(&receipt, "post-hoc-k", 4.0, false),
            Err(AnalysisEngineError::BindingMismatch)
        );
        assert_eq!(
            submit_validation_run(&request(), &offered, 3, MAX_SE_GATE_K + 0.01),
            Err(AnalysisEngineError::Validation(
                ValidationError::InvalidConfiguration
            ))
        );
        assert_eq!(
            submit_validation_run(&request(), &offered, 3, f64::NAN),
            Err(AnalysisEngineError::Validation(
                ValidationError::InvalidInput
            ))
        );
        assert!(submit_validation_run(&request(), &offered, 3, MAX_SE_GATE_K).is_ok());
        assert_eq!(
            RecoveryObservation::new(
                &receipt,
                "empty",
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                3.0,
                false,
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            RecoveryObservation::new(
                &receipt,
                "mismatch",
                vec![1.0, 2.0],
                vec![1.0],
                vec![0.0, 1.0],
                vec![2.0, 3.0],
                vec![1.0, 2.0],
                vec![1.0, 2.0],
                3.0,
                false,
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
