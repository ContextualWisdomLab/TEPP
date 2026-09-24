//! Prospective numerical fit design for coverage-calibration studies.

use sha2::{Digest, Sha256};

use crate::{FittedCandidateKConfig, ModelSelectionError};

const COVERAGE_CALIBRATION_FIT_DESIGN_FINGERPRINT_DOMAIN: &[u8] =
    b"tepp.model-selection.coverage-calibration-fit-design.v1\0";
const COVERAGE_CALIBRATION_FIT_SEEDS: [u64; 3] = [7, 11, 19];
const COVERAGE_CALIBRATION_MAXIMUM_ITERATIONS: usize = 2_000;
const COVERAGE_CALIBRATION_TOLERANCE: f64 = 0.001;
const COVERAGE_CALIBRATION_PRIOR_VARIANCE: f64 = 1.0;
const COVERAGE_CALIBRATION_RELATION_STRENGTH: f64 = 0.25;
const COVERAGE_CALIBRATION_RIDGE: f64 = 0.01;
const COVERAGE_CALIBRATION_TOPIC_SMOOTHING: f64 = 0.05;
const COVERAGE_CALIBRATION_STEP_SIZE: f64 = 0.2;

/// Versioned estimator configuration used by the prospective coverage-calibration experiment.
///
/// This owner fixes the numerical recovery design separately from both the DGP
/// simulation scenario and the validation criterion. The current strategy fits
/// exactly one candidate, the simulator-declared truth `K`, under fixed seeds,
/// convergence controls, and hyperparameters. It does not execute the 10,000-DGP
/// experiment or define whether an observed numerical-failure rate is acceptable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoverageCalibrationFitDesign;

impl CoverageCalibrationFitDesign {
    /// Construct TEPP's prospectively declared truth-K coverage fit design, version 1.
    #[must_use]
    pub const fn truth_k_v1() -> Self {
        Self
    }

    /// Stable identity of this numerical fit design.
    #[must_use]
    pub const fn design_id(self) -> &'static str {
        "tepp.model_selection.coverage_calibration_fit.truth_k.v1"
    }

    /// Stable identity of the candidate strategy used by this design.
    #[must_use]
    pub const fn candidate_strategy_id(self) -> &'static str {
        "tepp.model_selection.coverage_calibration_fit.candidate_strategy.truth_k_single_candidate.v1"
    }

    /// Deterministic estimator initialization seeds in declared order.
    #[must_use]
    pub const fn seeds(self) -> &'static [u64] {
        &COVERAGE_CALIBRATION_FIT_SEEDS
    }

    /// Maximum iterations allowed for each declared estimator seed.
    #[must_use]
    pub const fn maximum_iterations(self) -> usize {
        COVERAGE_CALIBRATION_MAXIMUM_ITERATIONS
    }

    /// Relative-objective convergence tolerance.
    #[must_use]
    pub const fn tolerance(self) -> f64 {
        COVERAGE_CALIBRATION_TOLERANCE
    }

    /// Logistic-normal prior variance fixed for the calibration fit.
    #[must_use]
    pub const fn prior_variance(self) -> f64 {
        COVERAGE_CALIBRATION_PRIOR_VARIANCE
    }

    /// Relation penalty strength fixed for the calibration fit.
    #[must_use]
    pub const fn relation_strength(self) -> f64 {
        COVERAGE_CALIBRATION_RELATION_STRENGTH
    }

    /// Prevalence coefficient ridge fixed for the calibration fit.
    #[must_use]
    pub const fn ridge(self) -> f64 {
        COVERAGE_CALIBRATION_RIDGE
    }

    /// Topic multinomial smoothing fixed for the calibration fit.
    #[must_use]
    pub const fn topic_smoothing(self) -> f64 {
        COVERAGE_CALIBRATION_TOPIC_SMOOTHING
    }

    /// Bounded generalized-EM step size fixed for the calibration fit.
    #[must_use]
    pub const fn step_size(self) -> f64 {
        COVERAGE_CALIBRATION_STEP_SIZE
    }

    /// Build the exact fitted-candidate configuration for one simulator truth `K`.
    ///
    /// The truth topic count remains owned by the simulation scenario; this owner
    /// supplies every numerical estimator choice applied to that topic count.
    /// Hyperparameters are set explicitly rather than inherited from mutable generic
    /// fitted-selection defaults.
    ///
    /// # Errors
    ///
    /// Returns the existing model-selection validation error when `truth_k` or any
    /// owner-declared numerical configuration cannot form a valid fitted-candidate design.
    pub fn config_for_truth_k(
        self,
        truth_k: u32,
    ) -> Result<FittedCandidateKConfig, ModelSelectionError> {
        FittedCandidateKConfig::new(
            vec![truth_k],
            self.seeds().to_vec(),
            self.maximum_iterations(),
            self.tolerance(),
        )?
        .with_hyperparameters(
            self.prior_variance(),
            self.relation_strength(),
            self.ridge(),
            self.topic_smoothing(),
            self.step_size(),
        )
    }

    /// Compute the canonical SHA-256 fingerprint of the concrete truth-K fit design.
    ///
    /// The digest binds the versioned design and candidate-strategy identities, the
    /// supplied truth `K`, ordered seed manifest, convergence controls, and every
    /// explicitly declared numerical hyperparameter. String/count geometry uses
    /// little-endian `u64`; floating-point values use exact IEEE-754 binary64 bits.
    /// The domain tag prevents this digest from being reused as another TEPP identity.
    ///
    /// # Errors
    ///
    /// Returns [`ModelSelectionError::InvalidDiagnostic`] when canonical platform
    /// lengths cannot be represented as `u64`, or propagates the existing configuration
    /// validation error when `truth_k` is invalid.
    pub fn fingerprint_for_truth_k(self, truth_k: u32) -> Result<String, ModelSelectionError> {
        self.config_for_truth_k(truth_k)?;

        let design_id = self.design_id().as_bytes();
        let design_id_len =
            u64::try_from(design_id.len()).map_err(|_| ModelSelectionError::InvalidDiagnostic)?;
        let candidate_strategy_id = self.candidate_strategy_id().as_bytes();
        let candidate_strategy_id_len = u64::try_from(candidate_strategy_id.len())
            .map_err(|_| ModelSelectionError::InvalidDiagnostic)?;
        let seed_count = u64::try_from(self.seeds().len())
            .map_err(|_| ModelSelectionError::InvalidDiagnostic)?;
        let maximum_iterations = u64::try_from(self.maximum_iterations())
            .map_err(|_| ModelSelectionError::InvalidDiagnostic)?;

        let mut hasher = Sha256::new();
        hasher.update(COVERAGE_CALIBRATION_FIT_DESIGN_FINGERPRINT_DOMAIN);
        hasher.update(design_id_len.to_le_bytes());
        hasher.update(design_id);
        hasher.update(candidate_strategy_id_len.to_le_bytes());
        hasher.update(candidate_strategy_id);
        hasher.update(truth_k.to_le_bytes());
        hasher.update(seed_count.to_le_bytes());
        for seed in self.seeds() {
            hasher.update(seed.to_le_bytes());
        }
        hasher.update(maximum_iterations.to_le_bytes());
        hasher.update(self.tolerance().to_bits().to_le_bytes());
        hasher.update(self.prior_variance().to_bits().to_le_bytes());
        hasher.update(self.relation_strength().to_bits().to_le_bytes());
        hasher.update(self.ridge().to_bits().to_le_bytes());
        hasher.update(self.topic_smoothing().to_bits().to_le_bytes());
        hasher.update(self.step_size().to_bits().to_le_bytes());

        let digest = hasher.finalize();
        Ok(format!("{digest:x}"))
    }
}
