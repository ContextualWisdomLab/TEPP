//! Prospective identity of the interval-coverage quantity evaluated by calibration studies.

use crate::coverage::CoverageCalibrationDesign;

/// Versioned scientific estimand for a coverage-calibration design.
///
/// This identity is deliberately narrower than a generic "95% coverage" label.
/// It states which fitted population is covered and how repeated rolling-origin
/// observations are collapsed before between-DGP Monte Carlo inference. It does
/// not authenticate source data, define numerical-failure acceptability, or turn
/// training-fit calibration into held-out predictive validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoverageCalibrationEstimand {
    /// Marginal ALR intervals for fitted training documents, with one window-local
    /// document-by-coordinate coverage proportion and equal weighting of declared
    /// rolling-origin windows inside each independent DGP replication.
    TrainingFitAlrMarginalEqualWindow,
}

impl CoverageCalibrationEstimand {
    /// Stable wire identity for the exact covered population and aggregation rule.
    #[must_use]
    pub const fn estimand_id(self) -> &'static str {
        match self {
            Self::TrainingFitAlrMarginalEqualWindow => {
                "tepp.coverage.estimand.training_fit_alr_marginal_equal_window.v1"
            }
        }
    }
}

impl CoverageCalibrationDesign {
    /// Scientific coverage quantity fixed before the expensive calibration run.
    ///
    /// The v1 design intentionally targets fit-bound training-document latent-state
    /// intervals. Held-out state recovery and predictive evaluation remain separate
    /// validation claims and must not be described as this calibration estimand.
    #[must_use]
    pub const fn estimand(&self) -> CoverageCalibrationEstimand {
        CoverageCalibrationEstimand::TrainingFitAlrMarginalEqualWindow
    }
}
