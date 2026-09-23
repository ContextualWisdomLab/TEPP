//! Machine-readable evidence for prospective coverage-calibration studies.

use crate::{
    CoverageCalibrationDesign, MonteCarloRecoveryMetricSummary, ValidationError,
    assess_coverage_calibration, parse_commit_head,
};
use serde::Serialize;

const COVERAGE_CALIBRATION_EVIDENCE_SCHEMA_VERSION: u32 = 1;

/// Immutable evidence record for one prospective coverage-calibration experiment.
///
/// The record keeps validation criterion identity separate from simulation
/// scenario identity while binding both to an exact source commit. It preserves
/// the unconditional attempted/success/failure denominator and failure-rate
/// Monte Carlo uncertainty alongside conditional interval-calibration metrics.
/// A positive [`Self::supports_calibration_claim`] value is intentionally narrower
/// than estimator robustness, scientific promotion, or release authority.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CoverageCalibrationEvidenceRecord {
    schema_version: u32,
    validation_design_id: String,
    simulation_scenario_id: String,
    simulation_scenario_fingerprint: String,
    source_head: String,
    attempted_replication_count: usize,
    successful_replication_count: usize,
    failure_count: usize,
    failure_rate: f64,
    failure_rate_standard_error: f64,
    coverage_mean: Option<f64>,
    coverage_standard_deviation: Option<f64>,
    coverage_monte_carlo_standard_error: Option<f64>,
    coverage_percentile_lower: Option<f64>,
    coverage_percentile_upper: Option<f64>,
    coverage_within_practical_band: bool,
    monte_carlo_precision_sufficient: bool,
    supports_calibration_claim: bool,
}

impl CoverageCalibrationEvidenceRecord {
    /// Construct a persisted evidence record from owner validation design and summary.
    ///
    /// `simulation_scenario_id` and `simulation_scenario_fingerprint` are opaque
    /// values supplied by the simulation owner. This crate validates only their
    /// persistence-safe shape rather than importing simulation-domain source.
    /// `source_head` must be an exact lowercase forty-hex Git commit identity.
    /// The denominator is rechecked through [`assess_coverage_calibration`], so
    /// a summary from a shortened or extended experiment cannot be serialized
    /// under the prospective design identity.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] for an empty/control-bearing
    /// scenario identity, a non-canonical SHA-256 scenario fingerprint, a
    /// non-canonical Git head, or a summary whose attempt count differs from the
    /// prospective design.
    pub fn from_summary(
        design: &CoverageCalibrationDesign,
        summary: &MonteCarloRecoveryMetricSummary,
        simulation_scenario_id: &str,
        simulation_scenario_fingerprint: &str,
        source_head: &str,
    ) -> Result<Self, ValidationError> {
        if simulation_scenario_id.is_empty()
            || simulation_scenario_id.chars().any(char::is_control)
        {
            return Err(ValidationError::InvalidInput);
        }
        if !is_lower_hex(simulation_scenario_fingerprint, 64)
            || !is_lower_hex(source_head, 40)
        {
            return Err(ValidationError::InvalidInput);
        }
        parse_commit_head(source_head)?;
        let assessment = assess_coverage_calibration(design, summary)?;
        let successful_metric_summary = summary.successful_metric_summary();

        Ok(Self {
            schema_version: COVERAGE_CALIBRATION_EVIDENCE_SCHEMA_VERSION,
            validation_design_id: design.design_id().to_owned(),
            simulation_scenario_id: simulation_scenario_id.to_owned(),
            simulation_scenario_fingerprint: simulation_scenario_fingerprint.to_owned(),
            source_head: source_head.to_owned(),
            attempted_replication_count: assessment.attempted_replication_count(),
            successful_replication_count: assessment.successful_replication_count(),
            failure_count: assessment.failure_count(),
            failure_rate: assessment.failure_rate(),
            failure_rate_standard_error: summary.failure_rate_standard_error(),
            coverage_mean: assessment.coverage_mean(),
            coverage_standard_deviation: successful_metric_summary
                .map(|metric| metric.standard_deviation),
            coverage_monte_carlo_standard_error: assessment
                .coverage_monte_carlo_standard_error(),
            coverage_percentile_lower: successful_metric_summary
                .map(|metric| metric.percentile_lower),
            coverage_percentile_upper: successful_metric_summary
                .map(|metric| metric.percentile_upper),
            coverage_within_practical_band: assessment.coverage_within_practical_band(),
            monte_carlo_precision_sufficient: assessment.monte_carlo_precision_sufficient(),
            supports_calibration_claim: assessment.supports_calibration_claim(),
        })
    }

    /// Evidence wire-schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Prospective validation-design identity.
    #[must_use]
    pub fn validation_design_id(&self) -> &str {
        &self.validation_design_id
    }

    /// Simulation-owner scenario identity.
    #[must_use]
    pub fn simulation_scenario_id(&self) -> &str {
        &self.simulation_scenario_id
    }

    /// Immutable SHA-256 fingerprint of the declared simulation scenario.
    #[must_use]
    pub fn simulation_scenario_fingerprint(&self) -> &str {
        &self.simulation_scenario_fingerprint
    }

    /// Exact lowercase Git source commit that produced the evidence.
    #[must_use]
    pub fn source_head(&self) -> &str {
        &self.source_head
    }

    /// Unconditional number of DGP replications attempted.
    #[must_use]
    pub const fn attempted_replication_count(&self) -> usize {
        self.attempted_replication_count
    }

    /// Number of DGP replications with numerically available coverage.
    #[must_use]
    pub const fn successful_replication_count(&self) -> usize {
        self.successful_replication_count
    }

    /// Number of owner-admitted numerical failures.
    #[must_use]
    pub const fn failure_count(&self) -> usize {
        self.failure_count
    }

    /// Unconditional numerical failure proportion.
    #[must_use]
    pub const fn failure_rate(&self) -> f64 {
        self.failure_rate
    }

    /// Bernoulli Monte Carlo standard error of the unconditional failure proportion.
    #[must_use]
    pub const fn failure_rate_standard_error(&self) -> f64 {
        self.failure_rate_standard_error
    }

    /// Conditional mean DGP-level interval coverage, when any DGP succeeds.
    #[must_use]
    pub const fn coverage_mean(&self) -> Option<f64> {
        self.coverage_mean
    }

    /// Between-DGP sample standard deviation of conditional coverage when estimable.
    #[must_use]
    pub const fn coverage_standard_deviation(&self) -> Option<f64> {
        self.coverage_standard_deviation
    }

    /// Between-DGP Monte Carlo standard error of conditional coverage when estimable.
    #[must_use]
    pub const fn coverage_monte_carlo_standard_error(&self) -> Option<f64> {
        self.coverage_monte_carlo_standard_error
    }

    /// Lower empirical percentile of successful DGP-level coverage when estimable.
    #[must_use]
    pub const fn coverage_percentile_lower(&self) -> Option<f64> {
        self.coverage_percentile_lower
    }

    /// Upper empirical percentile of successful DGP-level coverage when estimable.
    #[must_use]
    pub const fn coverage_percentile_upper(&self) -> Option<f64> {
        self.coverage_percentile_upper
    }

    /// Whether conditional mean coverage lies in the prospective practical band.
    #[must_use]
    pub const fn coverage_within_practical_band(&self) -> bool {
        self.coverage_within_practical_band
    }

    /// Whether conditional coverage Monte Carlo precision meets the prospective target.
    #[must_use]
    pub const fn monte_carlo_precision_sufficient(&self) -> bool {
        self.monte_carlo_precision_sufficient
    }

    /// Whether this record supports the narrow conditional calibration claim.
    #[must_use]
    pub const fn supports_calibration_claim(&self) -> bool {
        self.supports_calibration_claim
    }

    /// Serialize the immutable evidence record to deterministic struct-order JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] if serialization unexpectedly fails.
    pub fn to_json(&self) -> Result<String, ValidationError> {
        serde_json::to_string(self).map_err(|_| ValidationError::InvalidInput)
    }
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
