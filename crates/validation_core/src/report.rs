//! Machine-readable validation artifacts.

use crate::MonteCarloSummary;
use crate::ValidationError;
use serde::{Deserialize, Serialize};

/// Machine-readable recovery report for a single study.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ValidationReport {
    /// Study label (not free-form PII).
    pub study_label: String,
    /// Root-mean-square error.
    pub rmse: f64,
    /// RMSE standard error.
    pub rmse_standard_error: f64,
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
    /// Serialize to canonical JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when serialization fails.
    pub fn to_json(&self) -> Result<String, ValidationError> {
        serde_json::to_string(self).map_err(|_| ValidationError::InvalidInput)
    }

    /// Render a short human-readable summary line.
    #[must_use]
    pub fn to_human_summary(&self) -> String {
        format!(
            "study={} rmse={:.6} (se={:.6}) bias={:.6} (se={:.6}) coverage={:.3} temporal_order={:.3}",
            self.study_label,
            self.rmse,
            self.rmse_standard_error,
            self.mean_bias,
            self.bias_standard_error,
            self.interval_coverage,
            self.temporal_order_accuracy
        )
    }
}

// Serde for MonteCarloSummary
impl Serialize for MonteCarloSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
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
        Ok(Self {
            replication_count: raw.replication_count,
            mean: raw.mean,
            standard_deviation: raw.standard_deviation,
            standard_error: raw.standard_error,
            percentile_lower: raw.percentile_lower,
            percentile_upper: raw.percentile_upper,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ValidationReport;
    use crate::MonteCarloSummary;

    #[test]
    fn report_json_and_human_summary_round_trip() {
        let report = ValidationReport {
            study_label: "foundation-recovery".into(),
            rmse: 0.1,
            rmse_standard_error: 0.01,
            mean_bias: 0.0,
            bias_standard_error: 0.02,
            interval_coverage: 0.95,
            coverage_wilson_lower: 0.9,
            coverage_wilson_upper: 0.98,
            temporal_order_accuracy: 1.0,
            monte_carlo_rmse: Some(MonteCarloSummary {
                replication_count: 10,
                mean: 0.11,
                standard_deviation: 0.01,
                standard_error: 0.003,
                percentile_lower: 0.09,
                percentile_upper: 0.13,
            }),
        };
        let json = report.to_json().expect("json");
        let decoded: ValidationReport = serde_json::from_str(&json).expect("decode");
        assert_eq!(decoded.study_label, "foundation-recovery");
        assert!(report.to_human_summary().contains("rmse=0.100000"));
        let none_report = ValidationReport {
            monte_carlo_rmse: None,
            ..report
        };
        assert!(none_report.to_json().expect("json").contains("null"));
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
