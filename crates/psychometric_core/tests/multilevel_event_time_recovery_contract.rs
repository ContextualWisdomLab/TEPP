//! True-parameter recovery for multilevel OLS, event-time log-rate, and CWC lags.
#![allow(clippy::cast_precision_loss)]

use psychometric_core::{
    ClusteredEventScore, ClusteredScore, EventOccasion, IndicatorKind, LagClock,
    LaggedWithinResidual, PsychometricError, map_discrete_lag_across_event_intervals,
    ordinary_least_squares_slope, recover_cluster_mean_within_between_slopes,
    recover_discrete_constant_predictor_effect, recover_discrete_lag_from_log_rate,
    recover_discrete_process_noise, recover_discrete_time_varying_predictor_effect,
    recover_event_series_mean_log_rate, recover_event_time_discrete_lag_and_log_rate,
    recover_irregular_centered_residual_log_rate, recover_kish_weighted_slope,
    recover_within_residual_event_time_log_rate, refuse_difference_quotient_as_local_rate,
    refuse_pooled_discrete_lag_across_unequal_intervals,
    refuse_unmatched_time_varying_predictor_interval,
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
fn cluster_mean_cwc_recovers_known_within_between_and_contextual() {
    let true_within = 0.5_f64;
    let true_between = 2.0_f64;
    let true_contextual = true_between - true_within;
    let mut rows = Vec::new();
    for cluster in 0..6_u64 {
        let cluster_mean = f64::from(u32::try_from(cluster).expect("tiny")) * 2.0;
        for occasion in 0..4 {
            let within = f64::from(occasion) - 1.5;
            let predictor = cluster_mean + within;
            let outcome = true_between * cluster_mean + true_within * within;
            rows.push(ClusteredScore {
                cluster_key: cluster,
                predictor,
                outcome,
            });
        }
    }
    let recovered = recover_cluster_mean_within_between_slopes(&rows).expect("cwc");
    let within_error = rmse(&[true_within], &[recovered.within_slope]);
    let between_error = rmse(&[true_between], &[recovered.between_slope]);
    let contextual_error = rmse(&[true_contextual], &[recovered.contextual_effect]);
    assert!(within_error < 1e-12, "within RMSE {within_error}");
    assert!(between_error < 1e-12, "between RMSE {between_error}");
    assert!(
        contextual_error < 1e-12,
        "contextual RMSE {contextual_error}"
    );

    let predictors: Vec<f64> = rows.iter().map(|row| row.predictor).collect();
    let outcomes: Vec<f64> = rows.iter().map(|row| row.outcome).collect();
    let pooled = ordinary_least_squares_slope(&predictors, &outcomes).expect("pooled");
    let pooled_within_error = rmse(&[true_within], &[pooled]);
    let pooled_between_error = rmse(&[true_between], &[pooled]);
    let pooled_contextual_error = rmse(&[true_contextual], &[pooled]);
    assert!(
        pooled_within_error > within_error,
        "pooled RMSE {pooled_within_error} should exceed CWC within {within_error}"
    );
    assert!(
        pooled_between_error > between_error,
        "pooled RMSE {pooled_between_error} should exceed CWC between {between_error}"
    );
    assert!(
        pooled_contextual_error > contextual_error,
        "pooled RMSE {pooled_contextual_error} should exceed CWC contextual {contextual_error}"
    );
    assert!(
        (recovered.contextual_effect - recovered.between_slope).abs() > 1e-9,
        "Enders & Tofighi (2007, Table 2): CWC contextual must not equal the between-cluster slope"
    );
}

#[test]
fn kish_weighted_slope_recovers_known_loading() {
    let true_slope = 0.75_f64;
    let predictor = [0.0_f64, 1.0, 2.0, 3.0];
    let outcome = [0.0, true_slope, 2.0 * true_slope, 3.0 * true_slope];
    let weights = [1.0_f64, 2.0, 1.0, 0.5];
    let recovered = recover_kish_weighted_slope(&predictor, &outcome, &weights).expect("wls");
    let error = rmse(&[true_slope], &[recovered]);
    assert!(error < 1e-12, "Kish WLS RMSE {error}");
}

#[test]
fn event_time_log_rate_recovers_known_drift_and_refuses_quotient() {
    let true_drift = -0.4_f64;
    let earlier = 1.25_f64;
    let delta = 1.5_f64;
    let later = earlier * (true_drift * delta).exp();
    let recovered =
        recover_event_time_discrete_lag_and_log_rate(earlier, later, delta, LagClock::EventTime)
            .expect("exact map");
    let error = rmse(&[true_drift], &[recovered.log_rate]);
    assert!(error < 1e-12, "log-rate RMSE {error}");
    assert_eq!(
        refuse_difference_quotient_as_local_rate(earlier, later, delta),
        Err(PsychometricError::DifferenceQuotientForbidden)
    );
    assert_eq!(
        recover_event_time_discrete_lag_and_log_rate(earlier, later, delta, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
}

#[test]
fn discrete_lag_remaps_across_unequal_event_intervals() {
    let true_drift = -0.35_f64;
    let month = 1.0_f64;
    let two_months = 2.0_f64;
    let month_lag =
        recover_discrete_lag_from_log_rate(true_drift, month, LagClock::EventTime).expect("φ(1)");
    let two_month_truth = (true_drift * two_months).exp();
    let remapped =
        map_discrete_lag_across_event_intervals(month_lag, month, two_months, LagClock::EventTime)
            .expect("φ(2)");
    let error = rmse(&[two_month_truth], &[remapped]);
    assert!(error < 1e-12, "interval-remap RMSE {error}");
    let pooled_error = rmse(&[two_month_truth], &[month_lag]);
    assert!(
        pooled_error > error,
        "Voelkle: pooling φ(1) as φ(2) RMSE {pooled_error} must exceed remap {error}"
    );
    assert_eq!(
        refuse_pooled_discrete_lag_across_unequal_intervals(month, two_months),
        Err(PsychometricError::UnequalIntervalPoolingForbidden)
    );
}

#[test]
fn forward_map_underflow_to_zero_fails_closed() {
    assert_eq!(
        recover_discrete_lag_from_log_rate(-800.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    let source_lag =
        recover_discrete_lag_from_log_rate(-0.7, 1.0, LagClock::EventTime).expect("source φ");
    assert!(source_lag > 0.0);
    assert_eq!(
        map_discrete_lag_across_event_intervals(source_lag, 1.0, 2000.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn constant_predictor_discrete_effect_recovers_equation_twelve() {
    let outcome_on_predictor = 0.2_f64;
    let predictor_log_rate = -0.5_f64;
    let delta = 2.0_f64;
    let recovered = recover_discrete_constant_predictor_effect(
        outcome_on_predictor,
        predictor_log_rate,
        delta,
        LagClock::EventTime,
    )
    .expect("eq 12");
    let expected =
        (outcome_on_predictor / predictor_log_rate) * (predictor_log_rate * delta).exp_m1();
    let error = rmse(&[expected], &[recovered]);
    assert!(error < 1e-15, "Eq. 12 RMSE {error}");
    let first_order = outcome_on_predictor * delta;
    let first_order_error = rmse(&[expected], &[first_order]);
    assert!(
        first_order_error > error,
        "Voelkle Eq. 12: first-order a_yx Δt RMSE {first_order_error} must exceed exact {error}"
    );
    assert_eq!(
        recover_discrete_constant_predictor_effect(
            outcome_on_predictor,
            0.0,
            delta,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    let underflowed =
        recover_discrete_constant_predictor_effect(1e308, 1e-308, 1e-308, LagClock::EventTime)
            .expect("eq 12 underflow limit");
    let underflow_error = rmse(&[1.0], &[underflowed]);
    assert!(
        underflow_error < 1e-15,
        "Eq. 12 binary64 underflow limit RMSE {underflow_error}"
    );
    // a_yx Δt overflows; Eq. 12 remains finite.
    let product_overflow =
        recover_discrete_constant_predictor_effect(1e308, -100.0, 10.0, LagClock::EventTime)
            .expect("eq 12 finite after a_yx Δt overflow");
    let product_overflow_truth = (1e308 / -100.0) * (-100.0_f64 * 10.0).exp_m1();
    let product_overflow_error = rmse(&[product_overflow_truth], &[product_overflow]);
    assert!(
        product_overflow_error / 1e306 < 1e-12,
        "Eq. 12 a_yx Δt overflow RMSE {product_overflow_error}"
    );
    assert!(!(1e308_f64 * 10.0).is_finite());
    // z → -∞: expm1(z)/z * Δt is +0; Eq. 12 → -a_yx/a_xx.
    let increment_argument = -1e308_f64 * 2.0;
    assert!(increment_argument.is_infinite() && increment_argument.is_sign_negative());
    assert_eq!(
        (increment_argument.exp_m1() / increment_argument * 2.0).to_bits(),
        0.0_f64.to_bits()
    );
    let negative_overflow =
        recover_discrete_constant_predictor_effect(1.0, -1e308, 2.0, LagClock::EventTime)
            .expect("eq 12 equilibrium increment");
    let negative_overflow_truth = -(1.0 / -1e308);
    let negative_overflow_error = rmse(&[negative_overflow_truth], &[negative_overflow]);
    assert!(
        negative_overflow_error / 1e-308 < 1e-12,
        "Eq. 12 z→-∞ equilibrium RMSE {negative_overflow_error}"
    );
    assert!(negative_overflow > 0.0);
    // expm1(800) is +∞; (1e-308/800)(exp(800)−1) remains finite.
    assert!(!800.0_f64.exp_m1().is_finite());
    let overflowed =
        recover_discrete_constant_predictor_effect(1e-308, 800.0, 1.0, LagClock::EventTime)
            .expect("eq 12 expm1 overflow");
    let overflowed_truth = (1e-308_f64.ln() + 800.0 - 800.0_f64.ln()).exp() - 1e-308 / 800.0;
    let overflowed_error = rmse(&[overflowed_truth], &[overflowed]);
    assert!(
        overflowed_error / overflowed_truth < 1e-12,
        "Eq. 12 expm1-overflow RMSE {overflowed_error}"
    );
    assert_eq!(
        recover_discrete_constant_predictor_effect(0.0, 800.0, 1.0, LagClock::EventTime),
        Ok(0.0)
    );
    assert_eq!(
        recover_discrete_constant_predictor_effect(0.0, 1e308, 2.0, LagClock::EventTime),
        Ok(0.0)
    );
}

#[test]
fn time_varying_predictor_discrete_effect_recovers_equation_fourteen() {
    let outcome_on_predictor = 0.2_f64;
    let delta = 2.0_f64;
    let recovered = recover_discrete_time_varying_predictor_effect(
        outcome_on_predictor,
        delta,
        delta,
        delta,
        LagClock::EventTime,
    )
    .expect("eq 14");
    let expected = outcome_on_predictor * delta;
    let error = rmse(&[expected], &[recovered]);
    assert!(error < 1e-15, "Eq. 14 RMSE {error}");
    let constant = recover_discrete_constant_predictor_effect(
        outcome_on_predictor,
        -0.5,
        delta,
        LagClock::EventTime,
    )
    .expect("eq 12");
    let crossed_error = rmse(&[constant], &[recovered]);
    assert!(
        crossed_error > error,
        "Voelkle Eq. 14 is not Eq. 12: crossed RMSE {crossed_error} must exceed {error}"
    );
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            1.0,
            2.0,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::UnmatchedTimeVaryingInterval)
    );
    assert_eq!(
        refuse_unmatched_time_varying_predictor_interval(1.0, 2.0),
        Err(PsychometricError::UnmatchedTimeVaryingInterval)
    );
}

#[test]
fn discrete_process_noise_recovers_driver_equation_three() {
    let diffusion = 0.4_f64;
    let drift = -0.5_f64;
    let delta = 1.0_f64;
    let recovered =
        recover_discrete_process_noise(diffusion, drift, delta, LagClock::EventTime).expect("q_dt");
    let expected = diffusion * ((2.0 * drift * delta).exp() - 1.0) / (2.0 * drift);
    let error = rmse(&[expected], &[recovered]);
    assert!(error < 1e-15, "Driver Eq. 3 Q_Δt RMSE {error}");
    let collapsed = rmse(&[expected], &[diffusion]);
    assert!(
        collapsed > error,
        "continuous diffusion is not discrete process noise: collapsed RMSE {collapsed} must exceed {error}"
    );
    assert_eq!(
        recover_discrete_process_noise(diffusion, 0.0, 2.5, LagClock::EventTime),
        Ok(diffusion * 2.5)
    );
    let underflowed = recover_discrete_process_noise(1.0, 1e-308, 1e-308, LagClock::EventTime)
        .expect("z underflow");
    assert!(rmse(&[1e-308], &[underflowed]) < 1e-320);
    let equilibrium =
        recover_discrete_process_noise(0.4, -1e300, 2.0, LagClock::EventTime).expect("eq var");
    assert!(rmse(&[0.4 / (2.0 * 1e300)], &[equilibrium]) < 1e-315);
    let overflowed =
        recover_discrete_process_noise(1e-308, 400.0, 1.0, LagClock::EventTime).expect("rewrite");
    let rewrite_scale = 1e-308 / 800.0;
    let rewrite = ((1e-308_f64).ln() + 800.0 - 800.0_f64.ln()).exp() - rewrite_scale;
    assert!(rmse(&[rewrite], &[overflowed]) / rewrite.abs() < 1e-12);
    assert_eq!(
        recover_discrete_process_noise(0.0, 800.0, 1.0, LagClock::EventTime),
        Ok(0.0)
    );
    let twice_rate_overflow =
        recover_discrete_process_noise(1.0, 1e308, 1e-308, LagClock::EventTime)
            .expect("2a overflow");
    let expected_twice_rate = 0.5 * 2.0_f64.exp_m1() / 1e308;
    assert!(rmse(&[expected_twice_rate], &[twice_rate_overflow]) / expected_twice_rate < 1e-12);
    let overflowed_equilibrium =
        recover_discrete_process_noise(1e308, -1e308, 2.0, LagClock::EventTime).expect("2a eq var");
    assert!(rmse(&[0.5], &[overflowed_equilibrium]) < 1e-15);
    assert_eq!(
        recover_discrete_process_noise(1.0, 800.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_process_noise(1e308, 0.1, 4000.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_process_noise(0.4, -0.5, 1.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
}

#[test]
fn within_residual_event_time_log_rate_beats_pooled_levels() {
    let true_drift = -0.3_f64;
    let mut rows = Vec::new();
    for (cluster, person_mean, start) in [(1_u64, 8.0_f64, 1.0_f64), (2, -5.0, 1.4)] {
        for step in 0..6 {
            let time = f64::from(step);
            rows.push(ClusteredEventScore {
                cluster_key: cluster,
                event_time: time,
                score: person_mean + start * (true_drift * time).exp(),
            });
        }
    }
    let recovered =
        recover_within_residual_event_time_log_rate(&rows, LagClock::EventTime).expect("cwc lag");
    let within_error = rmse(&[true_drift], &[recovered]);

    let mut pooled = Vec::new();
    for (cluster, person_mean, start) in [(1_u64, 8.0_f64, 1.0_f64), (2, -5.0, 1.4)] {
        let time = cluster as f64;
        pooled.push(EventOccasion {
            event_time: time,
            score: person_mean + start,
        });
    }
    let pooled_rate = recover_event_series_mean_log_rate(&pooled, LagClock::EventTime);
    match pooled_rate {
        Ok(rate) => {
            let pooled_error = rmse(&[true_drift], &[rate]);
            assert!(
                within_error < pooled_error,
                "CWC lag RMSE {within_error} should beat pooled {pooled_error}"
            );
        }
        Err(PsychometricError::InvalidNumericInput | PsychometricError::NonPositiveInterval) => {}
        Err(other) => panic!("unexpected pooled error {other}"),
    }
    assert!(within_error < 0.25, "CWC lag RMSE {within_error} too large");
}

#[test]
fn irregular_centered_residuals_recover_known_drift_better_than_cwc_of_raw_ar() {
    let true_drift = -0.35_f64;
    let pairs = [
        LaggedWithinResidual {
            earlier_residual: 1.4,
            later_residual: 1.4 * (true_drift * 0.4).exp(),
            event_delta: 0.4,
        },
        LaggedWithinResidual {
            earlier_residual: 0.9,
            later_residual: 0.9 * (true_drift * 1.6).exp(),
            event_delta: 1.6,
        },
        LaggedWithinResidual {
            earlier_residual: -0.7,
            later_residual: -0.7 * (true_drift * 2.2).exp(),
            event_delta: 2.2,
        },
    ];
    let centered = recover_irregular_centered_residual_log_rate(&pairs, LagClock::EventTime)
        .expect("centered residual");
    let centered_error = rmse(&[true_drift], &[centered]);
    assert!(
        centered_error < 1e-12,
        "already-centered irregular RMSE {centered_error}"
    );

    let mut raw_ar = Vec::new();
    for (cluster, person_mean, start) in [(1_u64, 7.5_f64, 1.1_f64), (2, -4.0, 0.8)] {
        for step in 0..6 {
            let time = f64::from(step);
            raw_ar.push(ClusteredEventScore {
                cluster_key: cluster,
                event_time: time,
                score: person_mean + start * (true_drift * time).exp(),
            });
        }
    }
    let cwc =
        recover_within_residual_event_time_log_rate(&raw_ar, LagClock::EventTime).expect("cwc ar");
    let cwc_error = rmse(&[true_drift], &[cwc]);
    assert!(
        cwc_error > centered_error,
        "Curran & Bauer: CWC of raw AR RMSE {cwc_error} must exceed already-centered {centered_error}"
    );
}

#[test]
fn admitted_coordinates_still_required_for_multilevel_weights() {
    assert_eq!(
        recover_kish_weighted_slope(&[0.0, 1.0], &[0.2, 0.3], &[1.0, 1.0]),
        ordinary_least_squares_slope(&[0.0, 1.0], &[0.2, 0.3])
    );
    let _ = IndicatorKind::AdditiveLogRatio;
}
