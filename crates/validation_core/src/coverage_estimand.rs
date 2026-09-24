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

    /// Number of rolling-origin windows that every successful v1 DGP must contribute.
    ///
    /// Equal-window aggregation is scientifically meaningful only when every
    /// successful DGP contains the complete prospectively declared window set.
    /// Numerical failure is represented by the absence of a coverage vector; it
    /// must not be represented by silently shortening this window set.
    #[must_use]
    pub const fn declared_rolling_origin_window_count(&self) -> usize {
        5
    }

    /// Stable identity of the Monte Carlo standard-error estimator for the failure rate.
    ///
    /// The current validation owner treats numerical failure as a Bernoulli outcome
    /// over the unconditional attempted-DGP denominator and reports the plug-in
    /// Monte Carlo standard error `sqrt(p_hat * (1 - p_hat) / n)`. This identity is
    /// reporting provenance only: it does not define an acceptable failure rate or
    /// promote a scientific claim.
    #[must_use]
    pub const fn failure_rate_monte_carlo_standard_error_method_id(&self) -> &'static str {
        "tepp.coverage.failure_rate_mcse.bernoulli_plugin_sqrt_p_one_minus_p_over_n.v1"
    }

    /// Stable identity of the Monte Carlo standard-error estimator used for the coverage mean.
    ///
    /// The current validation owner computes the between-DGP sample standard deviation
    /// with the `n - 1` denominator and divides it by `sqrt(n)`. Binding that rule keeps
    /// the prospective `maximum_monte_carlo_standard_error` criterion from silently
    /// changing meaning if a future implementation adopts a different finite-sample
    /// variance or standard-error estimator.
    #[must_use]
    pub const fn coverage_monte_carlo_standard_error_method_id(&self) -> &'static str {
        "tepp.coverage.mcse.sample_sd_n_minus_1_over_sqrt_n.v1"
    }

    /// Stable identity of the empirical percentile estimator used in durable evidence.
    ///
    /// The current Monte Carlo owner uses the inclusive nearest-rank convention:
    /// sort finite successful DGP values, compute `ceil(p * n)`, then select the
    /// corresponding one-based rank with endpoint saturation. Binding this method
    /// prevents a future interpolation-rule change from retaining the same
    /// prospective design identity.
    #[must_use]
    pub const fn coverage_percentile_method_id(&self) -> &'static str {
        "tepp.coverage.percentile.inclusive_nearest_rank.v1"
    }

    /// Lower empirical percentile probability fixed before the calibration run.
    #[must_use]
    pub const fn coverage_percentile_lower_probability(&self) -> f64 {
        0.025
    }

    /// Upper empirical percentile probability fixed before the calibration run.
    #[must_use]
    pub const fn coverage_percentile_upper_probability(&self) -> f64 {
        0.975
    }
}
