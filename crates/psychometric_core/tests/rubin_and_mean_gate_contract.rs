//! Rubin T combining and strong-invariance latent-mean claim boundaries.
#![allow(clippy::cast_precision_loss)]

use psychometric_core::{
    GroupIndicatorSeries, IndicatorKind, MeanInvarianceStatus, PsychometricError,
    classify_two_group_ols_invariance, combine_draw_level_ols_loadings,
    recover_loading_point_estimate_mean, recover_strong_gated_latent_mean_difference,
};

fn rmse(truth: &[f64], recovered: &[f64]) -> f64 {
    let n = truth.len() as f64;
    let sum_sq: f64 = truth
        .iter()
        .zip(recovered)
        .map(|(left, right)| {
            let residual = left - right;
            residual * residual
        })
        .sum();
    (sum_sq / n).sqrt()
}

#[test]
fn rubin_t_mean_recovers_true_loading_and_is_not_a_point_estimate_alias() {
    let true_loading = 0.8_f64;
    let factors = [-2.0_f64, -1.0, 0.0, 1.0, 2.0];
    let draws = [
        factors
            .iter()
            .map(|score| (true_loading - 0.05) * score)
            .collect::<Vec<f64>>(),
        factors
            .iter()
            .map(|score| (true_loading + 0.05) * score)
            .collect::<Vec<f64>>(),
        factors
            .iter()
            .map(|score| true_loading * score)
            .collect::<Vec<f64>>(),
    ];
    let combined = combine_draw_level_ols_loadings(&factors, &draws, IndicatorKind::LogisticNormal)
        .expect("rubin");
    let point =
        recover_loading_point_estimate_mean(&factors, &draws, IndicatorKind::LogisticNormal)
            .expect("point");
    let error = rmse(&[true_loading], &[combined.mean_loading]);
    assert!(error < 1e-12, "Rubin mean RMSE {error}");
    assert!((combined.mean_loading - point).abs() < 1e-15);
    let expected = combined.within_variance
        + (1.0 + 1.0 / (combined.draw_count as f64)) * combined.between_variance;
    assert!((combined.total_variance - expected).abs() < 1e-15);
    assert!(combined.between_variance > 0.0);
}

#[test]
fn rubin_t_noisy_truth_reports_bias_rmse_and_interval_coverage() {
    let true_loading = 0.8_f64;
    let factors: Vec<f64> = (0..24).map(|index| f64::from(index) - 11.5).collect();
    let mut recovered = Vec::new();
    let mut covered = 0_usize;

    for replicate in 0..40 {
        let mut draws = Vec::new();
        for draw in 0..8 {
            let phase = f64::from(replicate) * 0.37 + f64::from(draw) * 0.91;
            draws.push(
                factors
                    .iter()
                    .enumerate()
                    .map(|(index, factor)| {
                        let position =
                            (f64::from(u32::try_from(index).expect("tiny")) + 1.0) * 0.73 + phase;
                        0.4 + true_loading * factor
                            + 0.25 * position.sin()
                            + 0.12 * (1.7 * position).cos()
                    })
                    .collect::<Vec<f64>>(),
            );
        }

        let combined =
            combine_draw_level_ols_loadings(&factors, &draws, IndicatorKind::LogisticNormal)
                .expect("noisy Rubin draw");
        assert!(combined.within_variance > 0.0);
        let half_width = 1.96 * combined.total_variance.sqrt();
        if (combined.mean_loading - true_loading).abs() <= half_width {
            covered += 1;
        }
        recovered.push(combined.mean_loading);
    }

    let mean = recovered.iter().sum::<f64>() / recovered.len() as f64;
    let bias = mean - true_loading;
    let rmse = (recovered
        .iter()
        .map(|estimate| (estimate - true_loading).powi(2))
        .sum::<f64>()
        / recovered.len() as f64)
        .sqrt();
    let coverage = covered as f64 / recovered.len() as f64;
    assert!(bias.abs() < 0.01, "loading bias {bias}");
    assert!(rmse < 0.02, "loading RMSE {rmse}");
    // CONTRIBUTING.md requires Monte Carlo thresholds to carry sampling
    // uncertainty: the acceptance floor is the nominal 95% target minus the
    // 1.96-quantile binomial standard error at that target over the 40
    // deterministic replicates, not the bare nominal rate.
    let replicates = recovered.len() as f64;
    let nominal = 0.95_f64;
    let monte_carlo_se = (nominal * (1.0 - nominal) / replicates).sqrt();
    let acceptance_floor = nominal - 1.96 * monte_carlo_se;
    assert!(
        coverage >= acceptance_floor,
        "95% interval coverage {coverage} below derived floor {acceptance_floor}"
    );
}

#[test]
fn hash84_wire_map_excludes_local_strict() {
    assert_eq!(MeanInvarianceStatus::Strict.as_str(), "strict");
    assert_eq!(
        MeanInvarianceStatus::Strict.as_measurement_invariance_wire_name(),
        None
    );
    assert!(MeanInvarianceStatus::Strict.licenses_latent_mean_comparison());
    assert!(MeanInvarianceStatus::Strict.licenses_shared_metric_meaning());
    let hash84_wire_names = ["configural", "metric", "scalar"];
    assert_eq!(
        MeanInvarianceStatus::Configural.as_measurement_invariance_wire_name(),
        Some("configural")
    );
    assert_eq!(
        MeanInvarianceStatus::Metric.as_measurement_invariance_wire_name(),
        Some("metric")
    );
    assert_eq!(
        MeanInvarianceStatus::Strong.as_measurement_invariance_wire_name(),
        Some("scalar")
    );
    assert!(!hash84_wire_names.contains(&"strict"));
}

#[test]
fn metric_status_matches_hash84_metric_and_refuses_latent_means() {
    assert_eq!(
        MeanInvarianceStatus::Metric.as_measurement_invariance_wire_name(),
        Some("metric")
    );
    assert!(MeanInvarianceStatus::Metric.licenses_shared_metric_meaning());
    assert!(!MeanInvarianceStatus::Metric.licenses_latent_mean_comparison());

    let reference = GroupIndicatorSeries {
        factor_scores: vec![-1.0, 0.0, 1.0],
        indicators: vec![0.2, 1.0, 1.8],
    };
    let comparison = GroupIndicatorSeries {
        factor_scores: vec![-1.0, 0.0, 1.0],
        indicators: vec![1.2, 2.0, 2.8],
    };
    let classified = classify_two_group_ols_invariance(
        &reference,
        &comparison,
        IndicatorKind::AdditiveLogRatio,
        1e-9,
        1e-9,
        1e-9,
    )
    .expect("metric");
    assert_eq!(classified.status, MeanInvarianceStatus::Metric);
    assert_eq!(
        recover_strong_gated_latent_mean_difference(
            &reference,
            &comparison,
            IndicatorKind::AdditiveLogRatio,
            1e-9,
            1e-9,
            1e-9,
        ),
        Err(PsychometricError::StrongInvarianceRequired)
    );
}

#[test]
fn strong_status_matches_hash84_scalar_and_recovers_mean_difference() {
    assert_eq!(
        MeanInvarianceStatus::Strong.as_measurement_invariance_wire_name(),
        Some("scalar")
    );
    assert!(MeanInvarianceStatus::Strong.licenses_latent_mean_comparison());

    let reference = GroupIndicatorSeries {
        factor_scores: vec![-1.0, 0.0, 1.0],
        indicators: vec![-0.8, 0.0, 0.8],
    };
    let comparison = GroupIndicatorSeries {
        factor_scores: vec![1.0, 2.0, 3.0],
        indicators: vec![0.8, 1.6, 2.4],
    };
    let difference = recover_strong_gated_latent_mean_difference(
        &reference,
        &comparison,
        IndicatorKind::IsometricLogRatio,
        1e-9,
        1e-9,
        1e-9,
    )
    .expect("strong");
    let error = rmse(&[2.0], &[difference]);
    assert!(error < 1e-12, "latent-mean RMSE {error}");
}

#[test]
fn two_observation_series_cap_at_strong_scalar_and_still_license_means() {
    let reference = GroupIndicatorSeries {
        factor_scores: vec![-1.0, 1.0],
        indicators: vec![-0.7, 1.7],
    };
    let comparison = GroupIndicatorSeries {
        factor_scores: vec![0.0, 2.0],
        indicators: vec![0.5, 2.9],
    };
    let classified = classify_two_group_ols_invariance(
        &reference,
        &comparison,
        IndicatorKind::AdditiveLogRatio,
        1e-9,
        1e-9,
        1e-9,
    )
    .expect("two-obs");
    assert_eq!(classified.status, MeanInvarianceStatus::Strong);
    assert_eq!(
        classified.status.as_measurement_invariance_wire_name(),
        Some("scalar")
    );
    assert_eq!(
        classified.reference_residual_variance.to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        classified.comparison_residual_variance.to_bits(),
        0.0_f64.to_bits()
    );
    assert!(classified.status.licenses_latent_mean_comparison());
    let difference = recover_strong_gated_latent_mean_difference(
        &reference,
        &comparison,
        IndicatorKind::AdditiveLogRatio,
        1e-9,
        1e-9,
        1e-9,
    )
    .expect("licensed");
    let error = rmse(&[1.0], &[difference]);
    assert!(error < 1e-12, "two-obs latent-mean RMSE {error}");
}
