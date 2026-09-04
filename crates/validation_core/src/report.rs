//! Machine-readable validation artifacts.

use crate::MonteCarloSummary;
use crate::ValidationError;
use serde::{Deserialize, Serialize};

const RMSE_STANDARD_ERROR_RELATIVE_TOLERANCE: f64 = 25769803776.0 * f64::EPSILON / 402653184.0;
const MONTE_CARLO_RMSE_SUPPORT_RELATIVE_TOLERANCE: f64 = 64.0 * f64::EPSILON;
const WILSON_PAIR_ABSOLUTE_TOLERANCE: f64 = 64.0 * f64::EPSILON;

/// Check whether a stored Wilson endpoint pair can arise from one Wilson score interval.
///
/// For empirical coverage `p` and `a = z² / n`, the Wilson roots satisfy
/// `L * U = p² / (1 + a)` and `L + U = 1 + (2p - 1) / (1 + a)`. Eliminating
/// the unrecorded `a` gives a necessary endpoint-pair identity. For `p < 0.5`,
/// the equivalent identity on the uncovered proportion avoids squaring a tiny
/// `p`. All terms remain probability-scaled, so a small absolute binary64
/// tolerance is sufficient without overflow-prone reconstruction of `n` or `z`.
/// At exact all-covered `p = 1`, the eliminated identity is degenerate, but the
/// canonical producer still has the stronger necessary support `L = n/(n+z²) > 0`
/// for every non-empty sample and finite represented `z²`.
fn wilson_pair_is_algebraically_coherent(p: f64, lower: f64, upper: f64) -> bool {
    if p == 0.0 {
        return true;
    }
    if p == 1.0 {
        return lower > 0.0;
    }

    let endpoint_sum = lower + upper;
    let (left, right) = if p >= 0.5 {
        (
            p * p * (endpoint_sum - 1.0),
            (2.5 * p - 0.5 - 0.5 * p) * lower * upper,
        )
    } else {
        let uncovered = 1.0 - p;
        (
            uncovered * uncovered * (1.0 - endpoint_sum),
            (1.0 - 2.0 * p) * (1.0 - lower) * (1.0 - upper),
        )
    };

    (left - right).abs() <= WILSON_PAIR_ABSOLUTE_TOLERANCE
}

/// Machine-readable recovery report for a single study.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationReport {
    /// Study label (not free-form PII).
    pub study_label: String,
    /// Root-mean-square error.
    pub rmse: f64,
    /// RMSE standard error.
    pub rmse_standard_error,
    /// Mean signed bias.
    pub mean_bias: f64,
    /// Bias standard error.
    pub bias_standard_error: f64,
    /// Empirical interval coverage.
    pub interval_coverage: f64,
    /// Wilson lower bound for coverage.
    pub coverage_wilson_lower: f64,
    /// Wilson upper bound for coverage.
    pub coverage_wilson_upper: f64,
    /// Temporal-order accuracy.
    pub temporal_order_accuracy: f64,
    /// Optional Monte Carlo RMSE summary.
    pub monte_carlo_rmse: Option<MonteCarloSummary>,
}

impl ValidationReport {
    /// Validate numeric and scientific invariants before serialization or nominal export.
    ///
    /// RMSE and standard errors are nonnegative. Under the crate's squared-residual
    /// delta-method producer, `SE(RMSE) <= RMSE / 2`: for `x_i = r_i^2 >= 0`, the
    /// sample standard deviation satisfies `sd(x) <= sqrt(n) * mean(x)`. Admission
    /// allows a small relative binary64 tolerance at that support boundary. Exact
    /// zero RMSE is perfect recovery and therefore still requires exact-zero RMSE
    /// standard error. Empirical coverage, Wilson endpoints, and temporal-order
    /// accuracy are probabilities in `[0, 1]`; the Wilson interval is ordered,
    /// contains the empirical coverage recorded in the same report, and its two
    /// endpoints must satisfy the same Wilson-score root identity for that
    /// coverage. Exact all-covered evidence additionally requires a strictly
    /// positive Wilson lower endpoint, matching the canonical `n / (n + z²)`
    /// producer for every non-empty sample and finite represented `z²`. These
    /// checks prevent individually plausible bounds from being combined into an
    /// interval that the canonical producer cannot emit. Mean signed bias remains
    /// unrestricted in sign. A generic [`MonteCarloSummary`] may summarize a signed
    /// metric, but when it occupies `monte_carlo_rmse` every retained replication is
    /// nonnegative. Its mean and percentile endpoints are therefore nonnegative.
    /// Nonnegative sample support additionally implies `SD <= sqrt(n) * mean`,
    /// `SE(mean) <= mean`, and every retained value—and thus every inclusive
    /// nearest-rank percentile endpoint—is at most `n * mean`. Admission evaluates
    /// the percentile support as `endpoint / mean <= n` with a small relative
    /// binary64 tolerance so the check does not overflow a finite sample sum. A
    /// zero Monte Carlo RMSE mean is exact perfect recovery across every retained
    /// replication, so spread, standard error, and empirical percentile endpoints
    /// must all be zero as well. These checks prevent a finite but scientifically
    /// impossible payload from becoming durable Validation Evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when any `f64` field is
    /// non-finite, violates its metric domain, point RMse and its standard error
    /// exceed squared-residual support, Wilson evidence is incoherent, or the
    /// optional Monte Carlo RMSE summary violates either generic summary invariants
    /// or the nonnegative RMSE support.
    pub fn validate(&self) -> Result<(), ValidationError> {
        for value in [
            self.rmse,
            self.rmse_standard_error,
            self.mean_bias,
            self.bias_standard_error,
            self.interval_coverage,
            self.coverage_wilson_lower,
            self.coverage_wilson_upper,
            self.temporal_order_accuracy,
        ] {
            if !value.is_finite() {
                return Err(ValidationError::InvalidInput);
            }
        }

        if self.rmse < 0.0 || self.rmse_standard_error < 0.0 || self.bias_standard_error < 0.0 {
            return Err(ValidationError::InvalidInput);
        }
        if self.rmse == 0.0 {
            if self.rmse_standard_error != 0.0 {
                return Err(ValidationError::InvalidInput);
            }
        } else {
            let relative_standard_error = self.rmse_standard_error / self.rmse;
            if !relative_standard_error.is_finite()
                || relative_standard_error > 0.5 + RMSE_STANDARD_ERROR_RELATIVE_TOLERANCE
            {
                return Err(ValidationError::InvalidInput);
            }
        }
        if !(0.0..=1.0).contains(&self.interval_coverage)
            || !(0.0..=1.0).contains(&self.coverage_wilson_lower)
            || !(0.0..=1.0).contains(&self.coverage_wilson_upper)
            || !(0.0..=1.0).contains(&self.temporal_order_accuracy)
        {
            return Err(ValidationError::InvalidInput);
        }
        if self.coverage_wilson_lower > self.coverage_wilson_upper
            || self.interval_coverage < self.coverage_wilson_lower
            || self.interval_coverage > self.coverage_wilson_upper
            || !wilson_pair_is_algebraically_coherent(
                self.interval_coverage,
                self.coverage_wilson_lower,
                self.coverage_wilson_upper,
            )
        {
            return Err(ValidationError::InvalidInput);
        }

        if let Some(summary) = self.monte_carlo_rmse {
            let summary = summary.validate()?;
            if summary.mean < 0.0
                || summary.percentile_lower < 0.0
                || summary.percentile_upper < 0.0
            {
                return Err(ValidationError::InvalidInput);
            }
            if summary.mean == 0.0 {
                if summary.standard_deviation != 0.0
                    || summary.standard_error != 0.0
                    || summary.percentile_lower != 0.0
                    || summary.percentile_upper != 0.0
                {
                    return Err(ValidationError::InvalidInput);
                }
            } else {
                let relative_standard_error = summary.standard_error / summary.mean;
                if !relative_standard_error.is_finite()
                    || relative_standard_error
                        > 1.0 + RMSE_STANDARD_ERROR_RELATIVE_TOLERANCE
                {
                    return Err(ValidationError::InvalidInput);
                }

                let relative_upper_percentile = summary.percentile_upper / summary.mean;
                let replication_support = summary.replication_count as f64;
                if !relative_upper_percentile.is_finite()
                    || relative_upper_percentile
                        > replication_support
                            * (1.0 + MONTE_CARLO_RMSE_SUPPORT_RELATIVE_TOLERANCE)
                {
                    return Err(ValidationError::InvalidInput);
                }
            }
        }
        Ok(())
    }

    /// Serialize to canonical JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when fields violate report
    /// invariants or serialization fails.
    pub fn to_json(&self) -> Result<String, ValidationError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| ValidationError::InvalidInput)
    }

    /// Render a short human-readable summary line after validating the report.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when the report violates its
    /// numeric or scientific invariants. Human-readable projection is therefore
    /// subject to the same fail-closed boundary as JSON ingress and egress.
    pub fn to_human_summary(&self) -> Result<String, ValidationError> {
        self.validate()?;
        Ok(format!(
            "study={} rmse={:.6} (se={:.6}) bias={:.6} (se={:.6}) coverage={:.3} temporal_order={:.3}",
            self.study_label,
            self.rmse,
            self.rmse_standard_error,
            self.mean_bias,
            self.bias_standard_error,
            self.interval_coverage,
            self.temporal_order_accuracy
        ))
    }
}

impl Serialize for ValidationReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        self.validate().map_err(serde::ser::Error::custom)?;
        let mut state = serializer.serialize_struct("ValidationReport", 10)?;
        state.serialize_field("study_label", &self.study_label)?;
        state.serialize_field("rmse", &self.rmse)?;
        state.serialize_field("rmse_standard_error", &self.rmse_standard_error)?;
        state.serialize_field("mean_bias", &self.mean_bias)?;
        state.serialize_field("bias_standard_error", &self.bias_standard_error)?;
        state.serialize_field("interval_coverage", &self.interval_coverage)?;
        state.serialize_field("coverage_wilson_lower", &self.coverage_wilson_lower)?;
        state.serialize_field("coverage_wilson_upper", &self.coverage_wilson_upper)?;
        state.serialize_field("temporal_order_accuracy", &self.temporal_order_accuracy)?;
        state.serialize_field("monte_carlo_rmse", &self.monte_carlo_rmse)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ValidationReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            study_label: String,
            rmse: f64,
            rmse_standard_error: f64,
            mean_bias: f64,
            bias_standard_error: f64,
            interval_coverage: f64,
            coverage_wilson_lower: f64,
            coverage_wilson_upper: f64,
            temporal_order_accuracy: f64,
            monte_carlo_rmse: Option<MonteCarloSummary>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let report = Self {
            study_label: raw.study_label,
            rmse: raw.rmse,
            rmse_standard_error: raw.rmse_standard_error,
            mean_bias: raw.mean_bias,
            bias_standard_error: raw.bias_standard_error,
            interval_coverage: raw.interval_coverage,
            coverage_wilson_lower: raw.coverage_wilson_lower,
            coverage_wilson_upper: raw.coverage_wilson_upper,
            temporal_order_accuracy: raw.temporal_order_accuracy,
            monte_carlo_rmse: raw.monte_carlo_rmse,
        };
        report.validate().map_err(serde::de::Error::custom)?;
        Ok(report)
    }
}

// Serde for MonteCarloSummary
impl Serialize for MonteCarloSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        (*self).validate().map_err(serde::ser::Error::custom)?;
        let mut state = serializer.serialize_struct("MonteCarloSummary", 6)?;
        state.serialize_field("replication_count", &self.replication_count)?;
        state.serialize_field("mean", &self.mean)?;
        state.serialize_field("standard_deviation", &self.standard_deviation)?;
        state.serialize_field("standard_error", &self.standard_error)?;
        state.serialize_field("percentile_lower", &self.percentile_lower)?;
        state.serialize_field("percentile_upper", &self.percentile_upper)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for MonteCarloSummary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            replication_count: usize,
            mean: f64,
            standard_deviation: f64,
            standard_error: f64,
            percentile_lower: f64,
            percentile_upper: f64,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self {
            replication_count: raw.replication_count,
            mean: raw.mean,
            standard_deviation: raw.standard_deviation,
            standard_error: raw.standard_error,
            percentile_lower: raw.percentile_lower,
            percentile_upper: raw.percentile_upper,
        }
        .validate()
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::ValidationReport;
    use crate::{MonteCarloSummary, ValidationError};

    #[test]
    fn report_json_and_human_summary_round_trip() {
        let report = ValidationReport {
            study_label: "foundation-recovery".into(),
            rmse: 0.1,
            rmse_standard_error: 0.01,
            mean_bias: 0.0,
            bias_standard_error: 0.02,
            interval_coverage: 0.5,
            coverage_wilson_lower: 0.2,
            coverage_wilson_upper: 0.8,
            temporal_order_accuracy: 1.0,
            monte_carlo_rmse: Some(MonteCarloSummary {
                replication_count: 10,
                mean: 0.11,
                standard_deviation: 0.01,
                standard_error: 0.01 / 10.0_f64.sqrt(),
                percentile_lower: 0.09,
                percentile_upper: 0.13,
            }),
        };
        let json = report.to_json().expect("json");
        let decoded: ValidationReport = serde_json::from_str(&[json.as_str()][0]).expect("decode");
        assert_eq!(decoded.study_label, "foundation-recovery");
        assert!(
            report
                .to_human_summary()
                .expect("human summary")
                .contains("rmse=0.100000")
        );
        let none_report = ValidationReport {
            monte_carlo_rmse: None,
            ..report.clone()
        };
        assert!(none_report.to_json().expect("json").contains("null"));
        let mut invalid = report.clone();
        invalid.rmse = f64::NAN;
        assert_eq!(invalid.to_json(), Err(ValidationError::InvalidInput));
        let bad_summary = r#"{"replication_count":0,"mean":0.0,"standard_deviation":0.0,"standard_error":0.0,"percentile_lower":0.0,"percentile_upper":1.0}"#;
        assert!(serde_json::from_str::<MonteCarloSummary>(bad_summary).is_err());
        let bad_order = r#"{"replication_count":2,"mean":0.0,"standard_deviation":0.0,"standard_error":0.0,"percentile_lower":1.0,"percentile_upper":0.0}"#;
        assert!(serde_json::from_str::<MonteCarloSummary>(bad_order).is_err());
        let bad_sd = r#"{"replication_count":2,"mean":0.0,"standard_deviation":-1.0,"standard_error":0.0,"percentile_lower":0.0,"percentile_upper":1.0}"#;
        assert!(serde_json::from_str::<MonteCarloSummary>(bad_sd).is_err());
    }

    #[test]
    fn foundation_recovery_study_recovers_known_parameters() {
        use crate::{
            EdgeIdentity, accept_within_standard_errors, bias_standard_error, edge_precision,
            edge_recall, interval_coverage, match_count, mean_bias, rmse_standard_error,
            root_mean_square_error, summarize_replications, temporal_order_accuracy,
            wilson_coverage_interval,
        };
        let truth = [0.70, 0.55, 0.40, -0.20, 0.85];
        let recovered = [0.72, 0.53, 0.41, -0.18, 0.84];
        let lower = [0.50, 0.35, 0.20, -0.40, 0.65];
        let upper = [0.90, 0.75, 0.60, 0.00, 1.00];
        let truth_times = [1.0, 2.0, 3.0, 4.0, 5.0];
        let recovered_times = [1.1, 1.9, 3.2, 3.8, 5.1];
        let rmse = root_mean_square_error(&truth, &recovered).expect("rmse");
        let rmse_se = rmse_standard_error(&truth, &recovered).expect("rmse se");
        let bias = mean_bias(&truth, &recovered).expect("bias");
        let bias_se = bias_standard_error(&truth, &recovered).expect("bias se");
        let coverage = interval_coverage(&truth, &lower, &upper).expect("cov");
        let (wilson_lo, wilson_hi) =
            wilson_coverage_interval(&truth, &lower, &upper, 1.96).expect("wilson");
        let order = temporal_order_accuracy(&truth_times, &recovered_times).expect("order");
        assert_eq!(match_count(&truth, &recovered, 0.05).expect("match"), 5);
        assert!(rmse < 0.05);
        assert!(accept_within_standard_errors(bias, 0.0, bias_se.max(1e-6), 3.0).expect("gate"));
        assert!((coverage - 1.0).abs() < 1e-12);
        assert!(wilson_lo <= coverage);
        assert!(coverage <= wilson_hi);
        assert!((order - 1.0).abs() < 1e-12);
        let truth_edges = [
            EdgeIdentity::new(1, 2),
            EdgeIdentity::new(2, 3),
            EdgeIdentity::new(3, 4),
        ];
        let recovered_edges = [
            EdgeIdentity::new(1, 2),
            EdgeIdentity::new(2, 3),
            EdgeIdentity::new(4, 5),
        ];
        assert!(
            (edge_precision(&truth_edges, &recovered_edges).expect("p") - (2.0 / 3.0)).abs()
                < 1e-12
        );
        assert!(
            (edge_recall(&truth_edges, &recovered_edges).expect("r") - (2.0 / 3.0)).abs() < 1e-12
        );
        let mc = summarize_replications(&[0.03, 0.04, 0.02, 0.05, 0.03], 0.1, 0.9).expect("mc");
        let report = ValidationReport {
            study_label: "foundation-loading-recovery".into(),
            rmse,
            rmse_standard_error: rmse_se,
            mean_bias: bias,
            bias_standard_error: bias_se,
            interval_coverage: coverage,
            coverage_wilson_lower: wilson_lo,
            coverage_wilson_upper: wilson_hi,
            temporal_order_accuracy: order,
            monte_carlo_rmse: Some(mc),
        };
        assert!(
            report
                .to_json()
                .expect("json")
                .contains("foundation-loading-recovery")
        );
    }
}
