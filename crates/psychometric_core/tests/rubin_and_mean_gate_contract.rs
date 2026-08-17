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
fn metric_status_matches_hash84_metric_and_refuses_latent_means() {
    assert_eq!(
        MeanInvarianceStatus::Metric.as_measurement_invariance_wire_name(),
        "metric"
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
        "scalar"
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
