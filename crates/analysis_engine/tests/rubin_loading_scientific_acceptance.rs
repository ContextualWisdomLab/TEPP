//! Repeated-sampling acceptance evidence for `rubin_loading_uncertainty_v1`.
//!
//! This test exercises the public Analysis Run profile end to end. It does not
//! implement a second loading estimator or treat complete-data indicator draws
//! as Mislevy person-level plausible values.

use analysis_engine::{
    RUBIN_LOADING_MODEL_CONTRACT_VERSION, RUBIN_LOADING_OUTPUT_PROFILE,
    RubinLoadingObservation, execute_rubin_loading_uncertainty_run,
};
use psychometric_core::IndicatorKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

const SNAPSHOT_ID: &str = "snapshot-rubin-scientific-acceptance";
const EARLY_AVAILABLE_AT: &str = "2026-07-01T00:00:00Z";
const LATE_AVAILABLE_AT: &str = "2026-08-15T00:00:00Z";
const EARLY_CUTOFF: &str = "2026-08-01T00:00:00Z";
const LATE_CUTOFF: &str = "2026-09-01T00:00:00Z";
const REPLICATIONS: usize = 512;
const NORMAL_975: f64 = 1.959_963_984_540_054;
const U32_RADIX: f64 = 4_294_967_296.0;
const F64_MANTISSA_CARDINALITY: f64 = 9_007_199_254_740_992.0;

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    observations: usize,
    draws: usize,
    loading: f64,
    residual_sd: f64,
    draw_noise_sd: f64,
    seed: u64,
    expected: ExpectedSummary,
}

#[derive(Clone, Copy)]
struct ExpectedSummary {
    bias: f64,
    rmse: f64,
    bias_mcse: f64,
    rmse_mcse: f64,
    coverage: f64,
    coverage_mcse: f64,
    mean_total_variance: f64,
    mean_within_variance: f64,
    mean_between_variance: f64,
}

#[derive(Debug)]
struct RecoverySummary {
    attempted: usize,
    recovered: usize,
    failed: usize,
    bias: f64,
    rmse: f64,
    bias_mcse: f64,
    rmse_mcse: f64,
    coverage: f64,
    coverage_mcse: f64,
    mean_total_variance: f64,
    mean_within_variance: f64,
    mean_between_variance: f64,
}

struct MetricSeries<'a> {
    truth: &'a [f64],
    recovered: &'a [f64],
    coverage_samples: &'a [f64],
    total_variances: &'a [f64],
    within_variances: &'a [f64],
    between_variances: &'a [f64],
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    fn open_unit_interval(&mut self) -> f64 {
        let mantissa = self.next_u64() >> 11;
        (exact_u64_to_f64(mantissa) + 0.5) / F64_MANTISSA_CARDINALITY
    }

    fn standard_normal(&mut self) -> f64 {
        let first = self.open_unit_interval();
        let second = self.open_unit_interval();
        (-2.0 * first.ln()).sqrt() * (std::f64::consts::TAU * second).cos()
    }
}

fn exact_u64_to_f64(value: u64) -> f64 {
    assert!(value <= (1_u64 << 53));
    let upper = u32::try_from(value >> 32).expect("upper 21 bits fit u32");
    let lower = u32::try_from(value & u64::from(u32::MAX)).expect("lower 32 bits fit u32");
    f64::from(upper) * U32_RADIX + f64::from(lower)
}

fn bounded_count(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("acceptance count fits u32"))
}

fn finite_mean(values: &[f64]) -> f64 {
    assert!(!values.is_empty());
    values.iter().sum::<f64>() / bounded_count(values.len())
}

fn root_mean_square(values: &[f64]) -> f64 {
    assert!(!values.is_empty());
    let mean_square = values.iter().map(|value| value * value).sum::<f64>()
        / bounded_count(values.len());
    mean_square.sqrt()
}

fn sample_standard_deviation(values: &[f64], mean: f64) -> f64 {
    assert!(values.len() >= 2);
    let squared_deviations = values
        .iter()
        .map(|value| {
            let deviation = value - mean;
            deviation * deviation
        })
        .sum::<f64>();
    (squared_deviations / (bounded_count(values.len()) - 1.0)).sqrt()
}

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("valid availability")
}

fn cutoff(stamp: &str) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(stamp).expect("valid cutoff")
}

fn request(cutoff_stamp: &str, idempotency_key: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: idempotency_key.into(),
        tenant_workspace_id: "tenant-rubin-scientific-acceptance".into(),
        snapshot_id: SNAPSHOT_ID.into(),
        knowledge_cutoff: cutoff_stamp.into(),
        model_contract_version: RUBIN_LOADING_MODEL_CONTRACT_VERSION.into(),
        output_profile: RUBIN_LOADING_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    let key = &request.idempotency_key;
    AnalysisRunAccepted::new(format!("run-{key}"), "accepted", key).expect("accepted receipt")
}

fn make_rows(
    rng: &mut SplitMix64,
    observations: usize,
    draws: usize,
    loading: f64,
    residual_sd: f64,
    draw_noise_sd: f64,
    early_observation_count: usize,
) -> Vec<RubinLoadingObservation> {
    let factor_scores: Vec<f64> = (0..observations).map(|_| rng.standard_normal()).collect();
    let base_outcomes: Vec<f64> = factor_scores
        .iter()
        .map(|factor_score| loading * factor_score + residual_sd * rng.standard_normal())
        .collect();
    let mut row_draws = vec![Vec::with_capacity(draws); observations];

    for _ in 0..draws {
        for (row_index, (draws_for_row, base_outcome)) in
            row_draws.iter_mut().zip(&base_outcomes).enumerate()
        {
            let draw_noise = if row_index.is_multiple_of(4) {
                draw_noise_sd * rng.standard_normal()
            } else {
                0.0
            };
            draws_for_row.push(base_outcome + draw_noise);
        }
    }

    factor_scores
        .into_iter()
        .zip(row_draws)
        .enumerate()
        .map(|(row_index, (factor_score, indicator_draws))| {
            let available_at = if row_index < early_observation_count {
                EARLY_AVAILABLE_AT
            } else {
                LATE_AVAILABLE_AT
            };
            RubinLoadingObservation::new(
                SNAPSHOT_ID,
                factor_score,
                indicator_draws,
                available(available_at),
            )
            .expect("valid simulated observation")
        })
        .collect()
}

fn summarize(series: &MetricSeries<'_>, attempted: usize, failed: usize) -> RecoverySummary {
    assert_eq!(series.truth.len(), series.recovered.len());
    assert_eq!(series.truth.len(), series.coverage_samples.len());
    assert_eq!(series.truth.len(), series.total_variances.len());
    assert_eq!(series.truth.len(), series.within_variances.len());
    assert_eq!(series.truth.len(), series.between_variances.len());
    assert_eq!(attempted, series.recovered.len() + failed);

    let errors: Vec<f64> = series
        .recovered
        .iter()
        .zip(series.truth)
        .map(|(estimate, truth)| estimate - truth)
        .collect();
    let squared_errors: Vec<f64> = errors.iter().map(|error| error * error).collect();
    let bias = finite_mean(&errors);
    let mean_squared_error = finite_mean(&squared_errors);
    let rmse = mean_squared_error.sqrt();
    let replication_count = bounded_count(series.recovered.len());
    let bias_mcse = sample_standard_deviation(&errors, bias) / replication_count.sqrt();
    let rmse_mcse = sample_standard_deviation(&squared_errors, mean_squared_error)
        / (2.0 * rmse * replication_count.sqrt());
    let coverage = finite_mean(series.coverage_samples);
    let coverage_mcse = sample_standard_deviation(series.coverage_samples, coverage)
        / replication_count.sqrt();

    RecoverySummary {
        attempted,
        recovered: series.recovered.len(),
        failed,
        bias,
        rmse,
        bias_mcse,
        rmse_mcse,
        coverage,
        coverage_mcse,
        mean_total_variance: finite_mean(series.total_variances),
        mean_within_variance: finite_mean(series.within_variances),
        mean_between_variance: finite_mean(series.between_variances),
    }
}

fn run_scenario(scenario: &Scenario) -> RecoverySummary {
    let run_request = request(EARLY_CUTOFF, scenario.name);
    let run_accepted = accepted(&run_request);
    let knowledge_cutoff = cutoff(EARLY_CUTOFF);
    let mut rng = SplitMix64::new(scenario.seed);
    let mut truth = Vec::with_capacity(REPLICATIONS);
    let mut recovered = Vec::with_capacity(REPLICATIONS);
    let mut coverage_samples = Vec::with_capacity(REPLICATIONS);
    let mut total_variances = Vec::with_capacity(REPLICATIONS);
    let mut within_variances = Vec::with_capacity(REPLICATIONS);
    let mut between_variances = Vec::with_capacity(REPLICATIONS);
    let mut failed = 0_usize;

    for _ in 0..REPLICATIONS {
        let rows = make_rows(
            &mut rng,
            scenario.observations,
            scenario.draws,
            scenario.loading,
            scenario.residual_sd,
            scenario.draw_noise_sd,
            scenario.observations,
        );
        match execute_rubin_loading_uncertainty_run(
            &run_request,
            &run_accepted,
            SNAPSHOT_ID,
            knowledge_cutoff,
            IndicatorKind::AdditiveLogRatio,
            &rows,
            "2026-09-14T00:00:00Z",
        ) {
            Ok(execution) => {
                let estimate = execution.artifact.point_estimate_mean;
                let interval_center = execution.artifact.mean_loading;
                let half_width = NORMAL_975 * execution.artifact.total_variance.sqrt();
                let interval_lower = interval_center - half_width;
                let interval_upper = interval_center + half_width;
                truth.push(scenario.loading);
                recovered.push(estimate);
                coverage_samples.push(if (interval_lower..=interval_upper).contains(&scenario.loading)
                {
                    1.0
                } else {
                    0.0
                });
                total_variances.push(execution.artifact.total_variance);
                within_variances.push(execution.artifact.within_variance);
                between_variances.push(execution.artifact.between_variance);
            }
            Err(_) => failed += 1,
        }
    }

    let series = MetricSeries {
        truth: &truth,
        recovered: &recovered,
        coverage_samples: &coverage_samples,
        total_variances: &total_variances,
        within_variances: &within_variances,
        between_variances: &between_variances,
    };
    summarize(&series, REPLICATIONS, failed)
}

fn assert_near(actual: f64, expected: f64, tolerance: f64, label: &str, scenario: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{scenario} {label}: expected {expected:.9}, got {actual:.9}"
    );
}

fn scenarios() -> [Scenario; 8] {
    [
        Scenario {
            name: "small_m8_low",
            observations: 48,
            draws: 8,
            loading: 0.4,
            residual_sd: 0.5,
            draw_noise_sd: 0.25,
            seed: 0x1001,
            expected: ExpectedSummary {
                bias: 0.002_108,
                rmse: 0.076_943,
                bias_mcse: 0.003_402,
                rmse_mcse: 0.002_491,
                coverage: 0.951_172,
                coverage_mcse: 0.009_534,
                mean_total_variance: 0.006_333,
                mean_within_variance: 0.005_945,
                mean_between_variance: 0.000_344,
            },
        },
        Scenario {
            name: "small_m32_low",
            observations: 48,
            draws: 32,
            loading: 0.4,
            residual_sd: 0.5,
            draw_noise_sd: 0.25,
            seed: 0x1002,
            expected: ExpectedSummary {
                bias: 0.000_426,
                rmse: 0.074_239,
                bias_mcse: 0.003_284,
                rmse_mcse: 0.002_370,
                coverage: 0.964_844,
                coverage_mcse: 0.008_147,
                mean_total_variance: 0.006_327,
                mean_within_variance: 0.005_974,
                mean_between_variance: 0.000_343,
            },
        },
        Scenario {
            name: "small_m8_high",
            observations: 48,
            draws: 8,
            loading: 1.2,
            residual_sd: 1.2,
            draw_noise_sd: 0.6,
            seed: 0x1003,
            expected: ExpectedSummary {
                bias: -0.015_361,
                rmse: 0.181_450,
                bias_mcse: 0.007_998,
                rmse_mcse: 0.005_577,
                coverage: 0.953_125,
                coverage_mcse: 0.009_351,
                mean_total_variance: 0.036_585,
                mean_within_variance: 0.034_424,
                mean_between_variance: 0.001_921,
            },
        },
        Scenario {
            name: "small_m32_high",
            observations: 48,
            draws: 32,
            loading: 1.2,
            residual_sd: 1.2,
            draw_noise_sd: 0.6,
            seed: 0x1004,
            expected: ExpectedSummary {
                bias: 0.003_076,
                rmse: 0.176_754,
                bias_mcse: 0.007_818,
                rmse_mcse: 0.005_550,
                coverage: 0.957_031,
                coverage_mcse: 0.008_971,
                mean_total_variance: 0.036_252,
                mean_within_variance: 0.034_184,
                mean_between_variance: 0.002_005,
            },
        },
        Scenario {
            name: "large_m8_low",
            observations: 160,
            draws: 8,
            loading: 0.4,
            residual_sd: 0.5,
            draw_noise_sd: 0.25,
            seed: 0x1005,
            expected: ExpectedSummary {
                bias: 0.001_179,
                rmse: 0.040_007,
                bias_mcse: 0.001_769,
                rmse_mcse: 0.001_279,
                coverage: 0.957_031,
                coverage_mcse: 0.008_971,
                mean_total_variance: 0.001_807,
                mean_within_variance: 0.001_694,
                mean_between_variance: 0.000_100,
            },
        },
        Scenario {
            name: "large_m32_low",
            observations: 160,
            draws: 32,
            loading: 0.4,
            residual_sd: 0.5,
            draw_noise_sd: 0.25,
            seed: 0x1006,
            expected: ExpectedSummary {
                bias: 0.000_850,
                rmse: 0.041_498,
                bias_mcse: 0.001_835,
                rmse_mcse: 0.001_359,
                coverage: 0.951_172,
                coverage_mcse: 0.009_534,
                mean_total_variance: 0.001_822,
                mean_within_variance: 0.001_719,
                mean_between_variance: 0.000_100,
            },
        },
        Scenario {
            name: "large_m8_high",
            observations: 160,
            draws: 8,
            loading: 1.2,
            residual_sd: 1.2,
            draw_noise_sd: 0.6,
            seed: 0x1007,
            expected: ExpectedSummary {
                bias: -0.007_944,
                rmse: 0.096_856,
                bias_mcse: 0.004_270,
                rmse_mcse: 0.003_125,
                coverage: 0.953_125,
                coverage_mcse: 0.009_351,
                mean_total_variance: 0.010_289,
                mean_within_variance: 0.009_633,
                mean_between_variance: 0.000_583,
            },
        },
        Scenario {
            name: "large_m32_high",
            observations: 160,
            draws: 32,
            loading: 1.2,
            residual_sd: 1.2,
            draw_noise_sd: 0.6,
            seed: 0x1008,
            expected: ExpectedSummary {
                bias: 0.003_612,
                rmse: 0.096_579,
                bias_mcse: 0.004_269,
                rmse_mcse: 0.002_863,
                coverage: 0.960_938,
                coverage_mcse: 0.008_571,
                mean_total_variance: 0.010_320,
                mean_within_variance: 0.009_718,
                mean_between_variance: 0.000_584,
            },
        },
    ]
}

#[test]
fn repeated_sampling_recovery_and_interval_coverage_match_checked_in_evidence() {
    for scenario in scenarios() {
        let summary = run_scenario(&scenario);
        let name = scenario.name;
        assert_eq!(summary.attempted, REPLICATIONS, "{name} attempted");
        assert_eq!(summary.recovered, REPLICATIONS, "{name} recovered");
        assert_eq!(summary.failed, 0, "{name} failed");
        assert!(
            summary.bias.abs() <= 0.01 + 3.0 * summary.bias_mcse,
            "{name} bias exceeds predeclared Monte Carlo envelope: {summary:?}"
        );
        assert!(
            summary.rmse <= 0.25 * scenario.residual_sd,
            "{name} RMSE exceeds predeclared design envelope: {summary:?}"
        );
        assert!(
            (summary.coverage - 0.95).abs() <= 0.01 + 2.0 * summary.coverage_mcse,
            "{name} coverage departs from the 95% diagnostic target: {summary:?}"
        );
        assert!(summary.mean_between_variance > 0.0, "{name} B must be exercised");
        assert!(
            summary.mean_total_variance > summary.mean_within_variance,
            "{name} T must include positive between-draw uncertainty"
        );

        let expected = scenario.expected;
        assert_near(summary.bias, expected.bias, 5e-6, "bias", name);
        assert_near(summary.rmse, expected.rmse, 5e-6, "rmse", name);
        assert_near(summary.bias_mcse, expected.bias_mcse, 5e-6, "bias_mcse", name);
        assert_near(summary.rmse_mcse, expected.rmse_mcse, 5e-6, "rmse_mcse", name);
        assert_near(summary.coverage, expected.coverage, 5e-6, "coverage", name);
        assert_near(
            summary.coverage_mcse,
            expected.coverage_mcse,
            5e-6,
            "coverage_mcse",
            name,
        );
        assert_near(
            summary.mean_total_variance,
            expected.mean_total_variance,
            5e-6,
            "mean_total_variance",
            name,
        );
        assert_near(
            summary.mean_within_variance,
            expected.mean_within_variance,
            5e-6,
            "mean_within_variance",
            name,
        );
        assert_near(
            summary.mean_between_variance,
            expected.mean_between_variance,
            5e-6,
            "mean_between_variance",
            name,
        );
    }
}

#[test]
fn rolling_origin_replay_excludes_late_rows_without_changing_the_earlier_result() {
    const OBSERVATIONS: usize = 64;
    const EARLY_OBSERVATIONS: usize = 48;
    const DRAWS: usize = 16;
    const LOADING: f64 = 0.8;
    const RESIDUAL_SD: f64 = 1.0;
    const DRAW_NOISE_SD: f64 = 0.5;

    let early_request = request(EARLY_CUTOFF, "rolling-origin-early");
    let early_accepted = accepted(&early_request);
    let late_request = request(LATE_CUTOFF, "rolling-origin-late");
    let late_accepted = accepted(&late_request);
    let mut rng = SplitMix64::new(0x5001);
    let mut early_errors = Vec::with_capacity(REPLICATIONS);
    let mut late_errors = Vec::with_capacity(REPLICATIONS);
    let mut early_total_variances = Vec::with_capacity(REPLICATIONS);
    let mut late_total_variances = Vec::with_capacity(REPLICATIONS);

    for _ in 0..REPLICATIONS {
        let rows = make_rows(
            &mut rng,
            OBSERVATIONS,
            DRAWS,
            LOADING,
            RESIDUAL_SD,
            DRAW_NOISE_SD,
            EARLY_OBSERVATIONS,
        );
        let early_full = execute_rubin_loading_uncertainty_run(
            &early_request,
            &early_accepted,
            SNAPSHOT_ID,
            cutoff(EARLY_CUTOFF),
            IndicatorKind::AdditiveLogRatio,
            &rows,
            "2026-09-14T00:00:00Z",
        )
        .expect("early full replay");
        let early_prefix = execute_rubin_loading_uncertainty_run(
            &early_request,
            &early_accepted,
            SNAPSHOT_ID,
            cutoff(EARLY_CUTOFF),
            IndicatorKind::AdditiveLogRatio,
            &rows[..EARLY_OBSERVATIONS],
            "2026-09-14T00:00:00Z",
        )
        .expect("early prefix replay");
        let late_full = execute_rubin_loading_uncertainty_run(
            &late_request,
            &late_accepted,
            SNAPSHOT_ID,
            cutoff(LATE_CUTOFF),
            IndicatorKind::AdditiveLogRatio,
            &rows,
            "2026-09-14T00:00:00Z",
        )
        .expect("late replay");

        assert_eq!(early_full.artifact.observation_count, 48);
        assert_eq!(early_full.artifact.excluded_after_cutoff_count, 16);
        assert_eq!(early_prefix.artifact.observation_count, 48);
        assert_eq!(early_prefix.artifact.excluded_after_cutoff_count, 0);
        assert_eq!(
            early_full.artifact.point_estimate_mean.to_bits(),
            early_prefix.artifact.point_estimate_mean.to_bits()
        );
        assert_eq!(
            early_full.artifact.mean_loading.to_bits(),
            early_prefix.artifact.mean_loading.to_bits()
        );
        assert_eq!(
            early_full.artifact.total_variance.to_bits(),
            early_prefix.artifact.total_variance.to_bits()
        );
        assert_eq!(late_full.artifact.observation_count, 64);
        assert_eq!(late_full.artifact.excluded_after_cutoff_count, 0);

        early_errors.push(early_full.artifact.point_estimate_mean - LOADING);
        late_errors.push(late_full.artifact.point_estimate_mean - LOADING);
        early_total_variances.push(early_full.artifact.total_variance);
        late_total_variances.push(late_full.artifact.total_variance);
    }

    let early_bias = finite_mean(&early_errors);
    let early_rmse = root_mean_square(&early_errors);
    let late_bias = finite_mean(&late_errors);
    let late_rmse = root_mean_square(&late_errors);
    let early_mean_total = finite_mean(&early_total_variances);
    let late_mean_total = finite_mean(&late_total_variances);

    assert_near(early_bias, 0.018_218, 5e-6, "early bias", "rolling-origin");
    assert_near(early_rmse, 0.155_489, 5e-6, "early rmse", "rolling-origin");
    assert_near(
        early_mean_total,
        0.025_215,
        5e-6,
        "early mean T",
        "rolling-origin",
    );
    assert_near(late_bias, 0.012_167, 5e-6, "late bias", "rolling-origin");
    assert_near(late_rmse, 0.131_383, 5e-6, "late rmse", "rolling-origin");
    assert_near(
        late_mean_total,
        0.018_653,
        5e-6,
        "late mean T",
        "rolling-origin",
    );
    assert!(
        late_rmse < early_rmse,
        "later availability should improve this predeclared design in aggregate"
    );
}

#[test]
fn monte_carlo_request_helpers_do_not_alias_immutable_snapshot_identity() {
    let first = request(EARLY_CUTOFF, "identity-red-0");
    let second = request(EARLY_CUTOFF, "identity-red-1");
    assert_ne!(first.snapshot_id, second.snapshot_id);
}
