//! Scientific claim boundaries for compositional coordinates and posterior draws.

use psychometric_core::{
    ClusteredEventScore, ClusteredScore, IndicatorKind, LagClock, LaggedWithinResidual,
    ordinary_least_squares_slope, posterior_draw_point_estimate_mean,
    recover_cluster_mean_within_between_slopes, recover_discrete_constant_predictor_effect,
    recover_discrete_process_noise, recover_discrete_time_varying_predictor_effect,
    recover_irregular_centered_residual_log_rate, recover_loading_point_estimate_mean,
    recover_within_residual_event_time_log_rate,
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

#[test]
fn cwc_cluster_mean_coefficient_is_not_the_between_cluster_effect() {
    let rows = [
        ClusteredScore {
            cluster_key: 1,
            predictor: 0.0,
            outcome: 2.0,
        },
        ClusteredScore {
            cluster_key: 1,
            predictor: 2.0,
            outcome: 3.0,
        },
        ClusteredScore {
            cluster_key: 2,
            predictor: 4.0,
            outcome: 10.0,
        },
        ClusteredScore {
            cluster_key: 2,
            predictor: 6.0,
            outcome: 11.0,
        },
    ];
    let recovered = recover_cluster_mean_within_between_slopes(&rows).expect("cwc");
    let predictors: Vec<f64> = rows.iter().map(|row| row.predictor).collect();
    let outcomes: Vec<f64> = rows.iter().map(|row| row.outcome).collect();
    let pooled = ordinary_least_squares_slope(&predictors, &outcomes).expect("pooled");
    assert!(
        (recovered.contextual_effect - recovered.between_slope).abs() > 1e-9,
        "Enders & Tofighi (2007, Table 2, pp. 124–127): CWC γ01 is contextual, not between"
    );
    assert!(
        (recovered.contextual_effect - pooled).abs() > 1e-9,
        "pooled OLS must not be treated as the CWC contextual effect"
    );
    assert!(
        ((recovered.contextual_effect + recovered.within_slope) - recovered.between_slope).abs()
            < 1e-15,
        "adding CWC γ01 to γ10 must recover the between-cluster slope"
    );
}

#[test]
fn time_varying_equation_fourteen_is_not_constant_equation_twelve() {
    let outcome_on_predictor = 0.35_f64;
    let delta = 1.5_f64;
    let time_varying = recover_discrete_time_varying_predictor_effect(
        outcome_on_predictor,
        delta,
        delta,
        delta,
        LagClock::EventTime,
    )
    .expect("eq 14");
    let constant = recover_discrete_constant_predictor_effect(
        outcome_on_predictor,
        -0.4,
        delta,
        LagClock::EventTime,
    )
    .expect("eq 12");
    assert!(
        (time_varying - constant).abs() > 1e-3,
        "Voelkle et al. (2012, manuscript p. 21): Eq. 14 a_yx Δt must not equal Eq. 12"
    );
    assert!((time_varying - outcome_on_predictor * delta).abs() < 1e-15);
}

#[test]
fn discrete_process_noise_is_not_the_continuous_diffusion() {
    let diffusion = 0.4_f64;
    let drift = -0.5_f64;
    let delta = 1.0_f64;
    let discrete =
        recover_discrete_process_noise(diffusion, drift, delta, LagClock::EventTime).expect("q_dt");
    assert!(
        (discrete - diffusion).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3, p. 4): Q_Δt must not equal continuous G G⊤"
    );
    let expected = diffusion * ((2.0 * drift * delta).exp() - 1.0) / (2.0 * drift);
    assert!((discrete - expected).abs() < 1e-15);
    let twice_rate_overflow =
        recover_discrete_process_noise(1.0, 1e308, 1e-308, LagClock::EventTime)
            .expect("2a overflow");
    let expected_twice_rate = 0.5 * 2.0_f64.exp_m1() / 1e308;
    assert!((twice_rate_overflow - expected_twice_rate).abs() / expected_twice_rate < 1e-12);
    let overflowed_equilibrium =
        recover_discrete_process_noise(1e308, -1e308, 2.0, LagClock::EventTime).expect("2a eq var");
    assert!((overflowed_equilibrium - 0.5).abs() < 1e-15);
    let constant =
        recover_discrete_constant_predictor_effect(diffusion, drift, delta, LagClock::EventTime)
            .expect("eq 12");
    assert!(
        (discrete - constant).abs() > 1e-3,
        "Driver Eq. 3 Q_Δt is not Voelkle Eq. 12"
    );
}
