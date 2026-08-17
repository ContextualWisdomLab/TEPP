//! Scientific claim boundaries for compositional coordinates and posterior draws.

use psychometric_core::{
    ClusteredEventScore, IndicatorKind, LagClock, LaggedWithinResidual,
    posterior_draw_point_estimate_mean, recover_irregular_centered_residual_log_rate,
    recover_loading_point_estimate_mean, recover_within_residual_event_time_log_rate,
};

#[test]
fn only_ilr_claims_orthonormal_aitchison_geometry() {
    assert!(IndicatorKind::AdditiveLogRatio.is_valid_structural_input());
    assert!(!IndicatorKind::AdditiveLogRatio.preserves_aitchison_distance());
    assert!(IndicatorKind::IsometricLogRatio.is_valid_structural_input());
    assert!(IndicatorKind::IsometricLogRatio.preserves_aitchison_distance());
    assert!(IndicatorKind::LogisticNormal.is_valid_structural_input());
    assert!(!IndicatorKind::LogisticNormal.preserves_aitchison_distance());
    assert!(!IndicatorKind::RawProportion.is_valid_structural_input());
    assert!(!IndicatorKind::RawProportion.preserves_aitchison_distance());
}

#[test]
fn posterior_draw_helpers_report_point_estimates_without_rubin_variance_claims() {
    let mean = posterior_draw_point_estimate_mean(&[0.7, 0.8, 0.9])
        .expect("finite posterior point estimates");
    assert!((mean - 0.8).abs() < 1e-15);

    let factor_scores = [-1.0_f64, 0.0, 1.0];
    let indicator_draws = vec![vec![-0.7, 0.0, 0.7], vec![-0.9, 0.0, 0.9]];
    let loading = recover_loading_point_estimate_mean(
        &factor_scores,
        &indicator_draws,
        IndicatorKind::AdditiveLogRatio,
    )
    .expect("posterior-draw point-estimate mean");
    assert!((loading - 0.8).abs() < 1e-15);
}

#[test]
fn person_mean_subtraction_on_raw_ar_is_not_the_lagged_within_effect() {
    let drift = -0.28_f64;
    let centered = recover_irregular_centered_residual_log_rate(
        &[LaggedWithinResidual {
            earlier_residual: 1.0,
            later_residual: (drift * 1.3).exp(),
            event_delta: 1.3,
        }],
        LagClock::EventTime,
    )
    .expect("already centered");
    assert!((centered - drift).abs() < 1e-12);

    let raw = [
        ClusteredEventScore {
            cluster_key: 1,
            event_time: 0.0,
            score: 6.0 + 1.0,
        },
        ClusteredEventScore {
            cluster_key: 1,
            event_time: 1.0,
            score: 6.0 + drift.exp(),
        },
        ClusteredEventScore {
            cluster_key: 1,
            event_time: 2.0,
            score: 6.0 + (drift * 2.0).exp(),
        },
        ClusteredEventScore {
            cluster_key: 2,
            event_time: 0.0,
            score: -3.0 + 1.2,
        },
        ClusteredEventScore {
            cluster_key: 2,
            event_time: 1.0,
            score: -3.0 + 1.2 * drift.exp(),
        },
        ClusteredEventScore {
            cluster_key: 2,
            event_time: 2.0,
            score: -3.0 + 1.2 * (drift * 2.0).exp(),
        },
    ];
    let cwc = recover_within_residual_event_time_log_rate(&raw, LagClock::EventTime).expect("cwc");
    assert!(
        (cwc - drift).abs() > 1e-6,
        "Curran & Bauer (2011, pp. 607–608): CWC of raw AR recovered {cwc}, which must not equal drift {drift}"
    );
}
