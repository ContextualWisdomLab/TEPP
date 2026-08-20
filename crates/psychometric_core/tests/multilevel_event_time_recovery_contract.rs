//! True-parameter recovery for multilevel OLS, event-time log-rate, and CWC lags.
#![allow(clippy::cast_precision_loss)]

use psychometric_core::{
    ClusteredEventScore, ClusteredScore, EventOccasion, IndicatorKind, LagClock,
    LaggedWithinResidual, PsychometricError, map_discrete_lag_across_event_intervals,
    ordinary_least_squares_slope, recover_cluster_mean_within_between_slopes,
    recover_discrete_constant_predictor_effect, recover_discrete_continuous_intercept_effect,
    recover_discrete_lag_from_log_rate, recover_discrete_lagged_latent_covariance,
    recover_discrete_latent_mean, recover_discrete_latent_mean_with_impulse,
    recover_discrete_latent_mean_with_impulse_carry,
    recover_discrete_latent_mean_with_time_independent_predictor, recover_discrete_latent_variance,
    recover_discrete_observed_mean, recover_discrete_observed_mean_with_impulse_carry,
    recover_discrete_process_noise, recover_discrete_time_independent_predictor_effect,
    recover_discrete_time_varying_predictor_effect, recover_event_series_mean_log_rate,
    recover_event_time_discrete_lag_and_log_rate, recover_irregular_centered_residual_log_rate,
    recover_kish_weighted_slope, recover_manifest_lagged_observed_covariance,
    recover_manifest_observed_mean, recover_manifest_observed_variance,
    recover_manifest_trait_plus_state_observed_variance, recover_stationary_latent_variance,
    recover_time_dependent_predictor_impulse, recover_time_dependent_predictor_impulse_carry,
    recover_trait_plus_state_lagged_covariance, recover_trait_plus_state_latent_variance,
    recover_within_residual_event_time_log_rate,
    refuse_continuous_intercept_as_discrete_mean_increment,
    refuse_continuous_intercept_as_initial_latent_mean,
    refuse_continuous_intercept_as_manifest_means, refuse_difference_quotient_as_local_rate,
    refuse_evolved_observed_mean_as_impulse_carry_observed_mean,
    refuse_finite_interval_process_noise_as_stationary_variance,
    refuse_initial_latent_mean_as_evolved_mean,
    refuse_initial_observed_mean_as_evolved_observed_mean,
    refuse_latent_lagged_covariance_as_observed_covariance, refuse_latent_mean_as_observed_mean,
    refuse_latent_variance_as_observed_variance, refuse_manifest_means_as_observed_mean,
    refuse_manifest_trait_variance_as_measurement_error,
    refuse_measurement_error_as_lagged_observed_covariance,
    refuse_measurement_error_as_observed_variance,
    refuse_pooled_discrete_lag_across_unequal_intervals,
    refuse_process_noise_as_unconditional_variance,
    refuse_time_dependent_impulse_as_continuous_intercept,
    refuse_time_dependent_impulse_as_time_independent_effect,
    refuse_time_dependent_impulse_as_time_varying_discrete_effect,
    refuse_time_dependent_impulse_carry_as_contemporaneous_impulse,
    refuse_time_dependent_impulse_carry_as_continuous_intercept,
    refuse_time_dependent_impulse_carry_as_time_independent_effect,
    refuse_time_dependent_impulse_carry_as_time_varying_discrete_effect,
    refuse_time_independent_coefficient_as_discrete_effect,
    refuse_time_independent_effect_as_continuous_intercept,
    refuse_time_independent_effect_as_time_dependent_impulse,
    refuse_time_independent_effect_as_time_varying_discrete_effect,
    refuse_trait_variance_as_process_noise, refuse_trait_variance_as_stationary_within_subject,
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
fn time_varying_predictor_equation_fourteen_intervals_fail_closed() {
    let outcome_on_predictor = 0.2_f64;
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            1.0,
            1.0,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            f64::NAN,
            1.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            0.0,
            1.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            1.0,
            f64::NAN,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            1.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            1.0,
            1.0,
            f64::NAN,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            1.0,
            1.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            2.0,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::UnmatchedTimeVaryingInterval)
    );
}

#[test]
fn time_varying_predictor_equation_fourteen_numeric_inputs_fail_closed() {
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            f64::NAN,
            1.0,
            1.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_time_varying_predictor_effect(
            1e308,
            10.0,
            10.0,
            10.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
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
        recover_discrete_process_noise(1.0, 1e308, 2.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_process_noise(0.4, -0.5, 0.0, LagClock::EventTime),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_process_noise(0.4, -0.5, -1.0, LagClock::EventTime),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_process_noise(0.4, -0.5, f64::NAN, LagClock::EventTime),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_process_noise(-0.1, -0.5, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_process_noise(f64::NAN, -0.5, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_process_noise(0.4, f64::NAN, 1.0, LagClock::EventTime),
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
fn discrete_latent_variance_recovers_driver_equations_three_and_four() {
    let prior = 2.0_f64;
    let diffusion = 0.4_f64;
    let drift = -0.5_f64;
    let delta = 1.0_f64;
    let process_noise =
        recover_discrete_process_noise(diffusion, drift, delta, LagClock::EventTime).expect("q_dt");
    let lagged =
        recover_discrete_lagged_latent_covariance(prior, drift, delta, LagClock::EventTime)
            .expect("lagged");
    let latent =
        recover_discrete_latent_variance(prior, diffusion, drift, delta, LagClock::EventTime)
            .expect("var");
    let expected_lagged = (drift * delta).exp() * prior;
    let expected_var = (2.0 * drift * delta).exp() * prior + process_noise;
    let lagged_error = rmse(&[expected_lagged], &[lagged]);
    let var_error = rmse(&[expected_var], &[latent]);
    assert!(
        lagged_error < 1e-15,
        "Driver Eq. 3-4 lagged RMSE {lagged_error}"
    );
    assert!(
        var_error < 1e-15,
        "Driver Eq. 3-4 variance RMSE {var_error}"
    );
    let collapsed = rmse(&[expected_var], &[process_noise]);
    assert!(
        collapsed > var_error,
        "Q_Δt is not Var(η_t): collapsed RMSE {collapsed} must exceed {var_error}"
    );
    assert_eq!(
        refuse_process_noise_as_unconditional_variance(process_noise, prior),
        Err(PsychometricError::ProcessNoiseIsConditionalVariance)
    );
    assert_eq!(
        recover_discrete_lagged_latent_covariance(0.0, 800.0, 1.0, LagClock::EventTime),
        Ok(0.0)
    );
    let rewritten =
        recover_discrete_lagged_latent_covariance(1e-308, 800.0, 1.0, LagClock::EventTime)
            .expect("rewrite");
    assert!(rewritten.is_finite());
    assert!(rewritten > 0.0);
    assert_eq!(
        recover_discrete_lagged_latent_covariance(2.0, 1e308, 2.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_latent_variance(-1.0, diffusion, drift, delta, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_lagged_latent_covariance(2.0, drift, 0.0, LagClock::EventTime),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_lagged_latent_covariance(2.0, drift, 1.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_lagged_latent_covariance(1e308, 700.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_latent_variance(1e308, 1e308, 0.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    // Zero diffusion skips process-noise z→+∞. exp(2 a Δt) p is then
    // non-finite (Driver Eq. 3–4).
    assert_eq!(
        recover_discrete_latent_variance(2.0, 0.0, 1e308, 2.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn stationary_variance_recovers_driver_equation_four_asymptote() {
    let diffusion = 0.4_f64;
    let drift = -0.5_f64;
    let stationary =
        recover_stationary_latent_variance(diffusion, drift, LagClock::EventTime).expect("asym");
    let expected = (diffusion / drift) * -0.5;
    let error = rmse(&[expected], &[stationary]);
    assert!(error < 1e-15, "Driver Eq. 4 asymDIFFUSION RMSE {error}");
    for delta in [0.5_f64, 1.0, 2.0, 10.0] {
        let evolved = recover_discrete_latent_variance(
            stationary,
            diffusion,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("invariant");
        let evolved_error = rmse(&[stationary], &[evolved]);
        assert!(
            evolved_error < 1e-12,
            "stationary variance must be invariant at Δt={delta}: RMSE {evolved_error}"
        );
    }
    let finite_noise =
        recover_discrete_process_noise(diffusion, drift, 1.0, LagClock::EventTime).expect("q_dt");
    let collapsed = rmse(&[stationary], &[finite_noise]);
    assert!(
        collapsed > error,
        "finite-Δt Q_Δt is not asymDIFFUSION: collapsed RMSE {collapsed} must exceed {error}"
    );
    assert_eq!(
        refuse_finite_interval_process_noise_as_stationary_variance(finite_noise, 1.0),
        Err(PsychometricError::FiniteIntervalProcessNoiseIsNotStationary)
    );
    assert_eq!(
        recover_stationary_latent_variance(diffusion, 0.0, LagClock::EventTime),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_latent_variance(diffusion, 0.5, LagClock::EventTime),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_latent_variance(1e308, -1e308, LagClock::EventTime),
        Ok(0.5)
    );
    let min_subnormal = f64::from_bits(1);
    assert_eq!(
        recover_stationary_latent_variance(min_subnormal, -min_subnormal, LagClock::EventTime),
        Ok(0.5)
    );
    assert!(!(f64::MAX / -0.75_f64).is_finite());
    assert_eq!(
        recover_stationary_latent_variance(f64::MAX, -0.75, LagClock::EventTime)
            .expect("q/a overflow")
            .to_bits(),
        (f64::MAX / 1.5).to_bits()
    );
    assert_eq!(
        recover_stationary_latent_variance(diffusion, drift, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_stationary_latent_variance(1e308, -1e-10, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn trait_plus_state_recovers_driver_section_four_point_three() {
    let trait_variance = 1.5_f64;
    let diffusion = 0.4_f64;
    let drift = -0.5_f64;
    let delta = 1.0_f64;
    let state =
        recover_stationary_latent_variance(diffusion, drift, LagClock::EventTime).expect("state");
    let total = recover_trait_plus_state_latent_variance(trait_variance, state).expect("sum");
    let expected_total = trait_variance + state;
    let error = rmse(&[expected_total], &[total]);
    assert!(error < 1e-15, "Driver §4.3 trait+state RMSE {error}");
    let lagged = recover_trait_plus_state_lagged_covariance(
        trait_variance,
        state,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("lagged");
    let state_lagged =
        recover_discrete_lagged_latent_covariance(state, drift, delta, LagClock::EventTime)
            .expect("state lagged");
    let lagged_error = rmse(&[trait_variance + state_lagged], &[lagged]);
    assert!(
        lagged_error < 1e-15,
        "Driver §4.3 trait+state lagged RMSE {lagged_error}"
    );
    let evolved_as_state =
        recover_discrete_latent_variance(total, diffusion, drift, delta, LagClock::EventTime)
            .expect("wrong");
    let evolved_state =
        recover_discrete_latent_variance(state, diffusion, drift, delta, LagClock::EventTime)
            .expect("state evolved");
    let evolved_right =
        recover_trait_plus_state_latent_variance(trait_variance, evolved_state).expect("right");
    let right_error = rmse(&[total], &[evolved_right]);
    assert!(
        right_error < 1e-12,
        "trait + stationary state must stay invariant: RMSE {right_error}"
    );
    let collapsed = rmse(&[evolved_right], &[evolved_as_state]);
    assert!(
        collapsed > error,
        "evolving trait+state as all-state is not Driver §4.3: collapsed RMSE {collapsed} must exceed {error}"
    );
    let process_noise =
        recover_discrete_process_noise(diffusion, drift, delta, LagClock::EventTime).expect("q_dt");
    assert_eq!(
        refuse_trait_variance_as_process_noise(trait_variance, process_noise),
        Err(PsychometricError::TraitVarianceIsNotProcessNoise)
    );
    assert_eq!(
        refuse_trait_variance_as_stationary_within_subject(trait_variance, state),
        Err(PsychometricError::TraitVarianceIsNotStationaryWithinSubject)
    );
    assert_eq!(
        recover_trait_plus_state_latent_variance(0.0, state),
        Ok(state)
    );
    assert_eq!(
        recover_trait_plus_state_latent_variance(trait_variance, 0.0),
        Ok(trait_variance)
    );
    assert_eq!(
        recover_trait_plus_state_lagged_covariance(1e308, 1e308, 0.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_trait_plus_state_lagged_covariance(0.4, 0.4, -0.5, 1.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
}

#[test]
fn manifest_observed_variance_recovers_driver_equation_five() {
    let loading = 2.0_f64;
    let latent = 0.4_f64;
    let measurement_error = 0.1_f64;
    let observed =
        recover_manifest_observed_variance(loading, latent, measurement_error).expect("eq5");
    let expected = (loading * latent) * loading + measurement_error;
    let error = rmse(&[expected], &[observed]);
    assert!(error < 1e-15, "Driver Eq. 5 Var(y) RMSE {error}");
    let collapsed_error = rmse(&[expected], &[measurement_error]);
    let latent_error = rmse(&[expected], &[latent]);
    assert!(
        collapsed_error > error,
        "MANIFESTVAR is not Var(y): collapsed RMSE {collapsed_error} must exceed {error}"
    );
    assert!(
        latent_error > error,
        "Var(η) is not Var(y): latent RMSE {latent_error} must exceed {error}"
    );
    assert_eq!(
        refuse_measurement_error_as_observed_variance(measurement_error, observed),
        Err(PsychometricError::MeasurementErrorIsNotObservedVariance)
    );
    assert_eq!(
        refuse_latent_variance_as_observed_variance(latent, observed),
        Err(PsychometricError::LatentVarianceIsNotObservedVariance)
    );
    assert_eq!(
        recover_manifest_observed_variance(0.0, latent, measurement_error),
        Ok(measurement_error)
    );
    let scaled = recover_manifest_observed_variance(1e308, 1e-308, 0.0).expect("scale");
    assert!(
        (scaled - 1e308).abs() / 1e308 < 1e-15,
        "Driver Eq. 5 (λ p)λ must keep λ=1e308, p=1e-308: got {scaled}"
    );
    assert_eq!(
        recover_manifest_observed_variance(1e308, 1.0, 0.0),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn manifest_trait_plus_state_observed_variance_recovers_driver_equation_five() {
    let loading = 2.0_f64;
    let latent = 0.4_f64;
    let measurement_error = 0.1_f64;
    let manifest_trait = 0.5_f64;
    let observed = recover_manifest_trait_plus_state_observed_variance(
        loading,
        latent,
        measurement_error,
        manifest_trait,
    )
    .expect("eq5-trait");
    let expected = (loading * latent) * loading + measurement_error + manifest_trait;
    let error = rmse(&[expected], &[observed]);
    assert!(error < 1e-15, "Driver Eq. 5 λ²p+θ+ψ RMSE {error}");
    let dropped_trait =
        recover_manifest_observed_variance(loading, latent, measurement_error).expect("psi0");
    let dropped_error = rmse(&[expected], &[dropped_trait]);
    assert!(
        dropped_error > error,
        "MANIFESTTRAITVAR is not dropped: RMSE {dropped_error} must exceed {error}"
    );
    let stuffed =
        recover_manifest_observed_variance(loading, latent, manifest_trait).expect("psi-as-theta");
    let stuffed_error = rmse(&[expected], &[stuffed]);
    assert!(
        stuffed_error > error,
        "MANIFESTTRAITVAR is not MANIFESTVAR: stuffed RMSE {stuffed_error} must exceed {error}"
    );
    let latent_trait =
        recover_manifest_observed_variance(loading, latent + manifest_trait, measurement_error)
            .expect("traitvar");
    let latent_trait_error = rmse(&[expected], &[latent_trait]);
    assert!(
        latent_trait_error > error,
        "TRAITVAR is not MANIFESTTRAITVAR: scaled RMSE {latent_trait_error} must exceed {error}"
    );
    assert_eq!(
        refuse_manifest_trait_variance_as_measurement_error(manifest_trait, measurement_error),
        Err(PsychometricError::ManifestTraitVarianceIsNotMeasurementError)
    );
    assert_eq!(
        recover_manifest_trait_plus_state_observed_variance(
            0.0,
            latent,
            measurement_error,
            manifest_trait
        ),
        Ok(measurement_error + manifest_trait)
    );
    let scaled = recover_manifest_trait_plus_state_observed_variance(1e308, 1e-308, 0.0, 1.0)
        .expect("scale");
    assert!(
        (scaled - 1e308).abs() / 1e308 < 1e-15,
        "Driver Eq. 5 (λ p)λ + ψ must keep λ=1e308, p=1e-308: got {scaled}"
    );
    assert_eq!(
        recover_manifest_trait_plus_state_observed_variance(1e308, 1e-308, 1e308, 1e308),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn manifest_lagged_observed_covariance_recovers_driver_equation_five() {
    let loading = 2.0_f64;
    let lagged = 0.4_f64;
    let manifest_trait = 0.5_f64;
    let observed = recover_manifest_lagged_observed_covariance(loading, lagged, manifest_trait)
        .expect("eq5-lag");
    let expected = (loading * lagged) * loading + manifest_trait;
    let error = rmse(&[expected], &[observed]);
    assert!(error < 1e-15, "Driver Eq. 5 lagged cov RMSE {error}");
    let latent_error = rmse(&[expected], &[lagged]);
    assert!(
        latent_error > error,
        "lagged Var(η) path is not cov(y): RMSE {latent_error} must exceed {error}"
    );
    let without_trait =
        recover_manifest_lagged_observed_covariance(loading, lagged, 0.0).expect("psi0");
    let dropped_error = rmse(&[expected], &[without_trait]);
    assert!(
        dropped_error > error,
        "MANIFESTTRAITVAR is not dropped from lagged cov: RMSE {dropped_error} must exceed {error}"
    );
    assert_eq!(
        refuse_latent_lagged_covariance_as_observed_covariance(lagged, observed),
        Err(PsychometricError::LatentLaggedCovarianceIsNotObservedCovariance)
    );
    assert_eq!(
        refuse_measurement_error_as_lagged_observed_covariance(0.1, observed),
        Err(PsychometricError::MeasurementErrorIsNotLaggedObservedCovariance)
    );
    let scaled = recover_manifest_lagged_observed_covariance(1e308, 1e-308, 0.0).expect("scale");
    assert!(
        (scaled - 1e308).abs() / 1e308 < 1e-15,
        "Driver Eq. 5 (λ c)λ must keep λ=1e308, c=1e-308: got {scaled}"
    );
    assert_eq!(
        recover_manifest_lagged_observed_covariance(1e308, 1.0, 0.0),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn manifest_observed_mean_recovers_driver_equation_five() {
    let loading = 2.0_f64;
    let latent_mean = 0.4_f64;
    let manifest_mean = 0.5_f64;
    let observed =
        recover_manifest_observed_mean(loading, latent_mean, manifest_mean).expect("eq5-mean");
    let expected = loading * latent_mean + manifest_mean;
    let error = rmse(&[expected], &[observed]);
    assert!(error < 1e-15, "Driver Eq. 5 observed mean RMSE {error}");
    let intercept_error = rmse(&[expected], &[manifest_mean]);
    assert!(
        intercept_error > error,
        "MANIFESTMEANS is not E(y): RMSE {intercept_error} must exceed {error}"
    );
    let latent_error = rmse(&[expected], &[latent_mean]);
    assert!(
        latent_error > error,
        "E(η) is not E(y): RMSE {latent_error} must exceed {error}"
    );
    let without_loading =
        recover_manifest_observed_mean(0.0, latent_mean, manifest_mean).expect("lambda0");
    let dropped_error = rmse(&[expected], &[without_loading]);
    assert!(
        dropped_error > error,
        "zero loading is τ, not τ + λμ: RMSE {dropped_error} must exceed {error}"
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, observed),
        Err(PsychometricError::ManifestMeansIsNotObservedMean)
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(latent_mean, observed),
        Err(PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_continuous_intercept_as_manifest_means(0.3, manifest_mean),
        Err(PsychometricError::ContinuousInterceptIsNotManifestMeans)
    );
    let scaled = recover_manifest_observed_mean(1e308, 1e-308, 0.0).expect("scale");
    assert!(
        (scaled - 1.0).abs() < 1e-15,
        "Driver Eq. 5 λμ must keep λ=1e308, μ=1e-308: got {scaled}"
    );
    let finite_loaded = recover_manifest_observed_mean(1e308, 1.0, 0.0).expect("lambda-mu");
    assert!(
        (finite_loaded - 1e308).abs() / 1e308 < 1e-15,
        "Driver Eq. 5 mean is λμ, not λ²: got {finite_loaded}"
    );
    assert_eq!(
        recover_manifest_observed_mean(1e308, 2.0, 0.0),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn discrete_latent_mean_recovers_driver_equation_three() {
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let observed =
        recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
            .expect("eq3-mean");
    let expected = (drift * delta).exp() * initial + intercept * ((drift * delta).exp_m1() / drift);
    let error = rmse(&[expected], &[observed]);
    assert!(error < 1e-15, "Driver Eq. 3 latent mean RMSE {error}");
    let initial_error = rmse(&[expected], &[initial]);
    assert!(
        initial_error > error,
        "T0MEANS is not μ_t: RMSE {initial_error} must exceed {error}"
    );
    let intercept_error = rmse(&[expected], &[intercept]);
    assert!(
        intercept_error > error,
        "CINT is not μ_t: RMSE {intercept_error} must exceed {error}"
    );
    let increment =
        recover_discrete_continuous_intercept_effect(intercept, drift, delta, LagClock::EventTime)
            .expect("cint");
    let increment_error = rmse(&[increment], &[intercept]);
    assert!(
        increment_error > 1e-3,
        "CINT is not the discrete increment: RMSE {increment_error}"
    );
    assert_eq!(
        refuse_initial_latent_mean_as_evolved_mean(initial, observed),
        Err(PsychometricError::InitialLatentMeanIsNotEvolvedMean)
    );
    assert_eq!(
        refuse_continuous_intercept_as_discrete_mean_increment(intercept, increment),
        Err(PsychometricError::ContinuousInterceptIsNotDiscreteMeanIncrement)
    );
    assert_eq!(
        refuse_continuous_intercept_as_initial_latent_mean(intercept, initial),
        Err(PsychometricError::ContinuousInterceptIsNotInitialLatentMean)
    );
    let integrator =
        recover_discrete_latent_mean(initial, 0.0, intercept, delta, LagClock::EventTime)
            .expect("a0");
    assert!(
        (integrator - (initial + intercept * delta)).abs() < 1e-15,
        "Driver Eq. 3 A=0 integral is κ Δt: got {integrator}"
    );
    let equilibrium = recover_discrete_latent_mean(initial, -1e308, 1.0, 2.0, LagClock::EventTime)
        .expect("eq3-equilibrium");
    let equilibrium_expected = -(1.0 / -1e308);
    assert!(
        (equilibrium - equilibrium_expected).abs() / 1e-308 < 1e-12,
        "Driver Eq. 3 z→-∞ drops T0MEANS and keeps -κ/a: got {equilibrium}"
    );
    assert_eq!(
        recover_discrete_latent_mean(1e308, 1.0, 0.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn discrete_observed_mean_recovers_driver_equations_three_and_five() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean(
        loading,
        initial,
        drift,
        intercept,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-eq5-mean");
    let evolved =
        recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
            .expect("mu-t");
    let expected = manifest_mean + loading * evolved;
    let error = rmse(&[expected], &[observed]);
    assert!(
        error < 1e-15,
        "Driver Eq. 5 of Eq. 3 evolved mean RMSE {error}"
    );
    let first_occasion =
        recover_manifest_observed_mean(loading, initial, manifest_mean).expect("t0");
    let first_error = rmse(&[expected], &[first_occasion]);
    assert!(
        first_error > error,
        "τ + λ μ_0 is not E(y_t): RMSE {first_error} must exceed {error}"
    );
    let intercept_error = rmse(&[expected], &[manifest_mean]);
    assert!(
        intercept_error > error,
        "MANIFESTMEANS is not E(y_t): RMSE {intercept_error} must exceed {error}"
    );
    let latent_error = rmse(&[expected], &[evolved]);
    assert!(
        latent_error > error,
        "μ_t is not E(y_t): RMSE {latent_error} must exceed {error}"
    );
    let zero_loading = recover_discrete_observed_mean(
        0.0,
        initial,
        drift,
        intercept,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("lambda0");
    let dropped_error = rmse(&[expected], &[zero_loading]);
    assert!(
        dropped_error > error,
        "zero loading is τ, not τ + λ μ_t: RMSE {dropped_error} must exceed {error}"
    );
}

#[test]
fn discrete_observed_mean_refuses_first_occasion_and_overflow() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean(
        loading,
        initial,
        drift,
        intercept,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-eq5-mean");
    let evolved =
        recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
            .expect("mu-t");
    let first_occasion =
        recover_manifest_observed_mean(loading, initial, manifest_mean).expect("t0");
    assert_eq!(
        refuse_initial_observed_mean_as_evolved_observed_mean(first_occasion, observed),
        Err(PsychometricError::InitialObservedMeanIsNotEvolvedObservedMean)
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(evolved, observed),
        Err(PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, observed),
        Err(PsychometricError::ManifestMeansIsNotObservedMean)
    );
    let integrator = recover_discrete_observed_mean(
        loading,
        initial,
        0.0,
        intercept,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("a0");
    assert!(
        (integrator - (manifest_mean + loading * (initial + intercept * delta))).abs() < 1e-15,
        "Driver Eq. 5 of A=0 mean is τ + λ(μ_0 + κ Δt): got {integrator}"
    );
    let equilibrium = recover_discrete_observed_mean(
        loading,
        initial,
        -1e308,
        1.0,
        manifest_mean,
        2.0,
        LagClock::EventTime,
    )
    .expect("eq3-eq5-equilibrium");
    let equilibrium_expected = manifest_mean + loading * (-(1.0 / -1e308));
    assert!(
        (equilibrium - equilibrium_expected).abs() < 1e-15,
        "Driver Eq. 5 of z→-∞ mean keeps τ + λ(-κ/a): got {equilibrium}"
    );
    let scaled =
        recover_discrete_observed_mean(1e308, 1e-308, 0.0, 0.0, 0.0, 1.0, LagClock::EventTime)
            .expect("scale");
    assert!(
        (scaled - 1.0).abs() < 1e-15,
        "Driver Eq. 5 of Eq. 3 mean must keep λ=1e308, μ_t=1e-308: got {scaled}"
    );
    assert_eq!(
        recover_discrete_observed_mean(1e308, 2.0, 0.0, 0.0, 0.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn time_dependent_impulse_recovers_driver_equation_three_fourth_summand() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
    let error = rmse(&[1.2], &[impulse]);
    assert!(
        error < 1e-15,
        "Driver Eq. 3 fourth summand RMSE {error}: got {impulse}"
    );
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let composed = recover_discrete_latent_mean_with_impulse(
        initial,
        drift,
        intercept,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-impulse");
    let evolved =
        recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
            .expect("mu-t");
    let composed_error = rmse(&[evolved + impulse], &[composed]);
    assert!(
        composed_error < 1e-15,
        "Driver Eq. 3 μ_t + m x RMSE {composed_error}: got {composed}"
    );
    let intercept_effect =
        recover_discrete_continuous_intercept_effect(effect, drift, delta, LagClock::EventTime)
            .expect("cint");
    let equation_fourteen = recover_discrete_time_varying_predictor_effect(
        effect,
        delta,
        delta,
        delta,
        LagClock::EventTime,
    )
    .expect("eq14");
    assert!((impulse - intercept_effect).abs() > 1e-3);
    assert!((impulse - equation_fourteen).abs() > 1e-3);
    assert_eq!(
        refuse_time_dependent_impulse_as_continuous_intercept(impulse, effect),
        Err(PsychometricError::TimeDependentImpulseIsNotContinuousIntercept)
    );
    assert_eq!(
        refuse_time_dependent_impulse_as_time_independent_effect(impulse, intercept_effect),
        Err(PsychometricError::TimeDependentImpulseIsNotTimeIndependentEffect)
    );
    assert_eq!(
        refuse_time_dependent_impulse_as_time_varying_discrete_effect(impulse, equation_fourteen),
        Err(PsychometricError::TimeDependentImpulseIsNotTimeVaryingDiscreteEffect)
    );
}

#[test]
fn time_dependent_impulse_refuses_overflow_and_non_event_clocks() {
    assert_eq!(
        recover_time_dependent_predictor_impulse(1e308, 2.0),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_latent_mean_with_impulse(
            1.0,
            -0.5,
            0.3,
            0.4,
            2.0,
            2.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_latent_mean_with_impulse(
            1e308,
            0.0,
            0.0,
            1e308,
            1.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn time_independent_predictor_recovers_driver_equation_three_second_summand() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let increment = recover_discrete_time_independent_predictor_effect(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("tipred");
    let expected =
        recover_discrete_constant_predictor_effect(1.2, drift, delta, LagClock::EventTime)
            .expect("bz-map");
    let error = rmse(&[expected], &[increment]);
    assert!(
        error < 1e-15,
        "Driver Eq. 3 TIPREDEFFECT RMSE {error}: got {increment}"
    );
    let intercept_effect =
        recover_discrete_continuous_intercept_effect(effect, drift, delta, LagClock::EventTime)
            .expect("cint");
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
    let equation_fourteen = recover_discrete_time_varying_predictor_effect(
        effect,
        delta,
        delta,
        delta,
        LagClock::EventTime,
    )
    .expect("eq14");
    assert!(rmse(&[increment], &[intercept_effect]) > rmse(&[expected], &[increment]));
    assert!(rmse(&[increment], &[impulse]) > rmse(&[expected], &[increment]));
    assert!(rmse(&[increment], &[equation_fourteen]) > rmse(&[expected], &[increment]));
    assert!(rmse(&[increment], &[effect]) > rmse(&[expected], &[increment]));
    let composed = recover_discrete_latent_mean_with_time_independent_predictor(
        1.0,
        drift,
        0.3,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-tipred");
    let evolved =
        recover_discrete_latent_mean(1.0, drift, 0.3, delta, LagClock::EventTime).expect("mu-t");
    let composed_error = rmse(&[evolved + increment], &[composed]);
    assert!(
        composed_error < 1e-15,
        "Driver Eq. 3 μ_t + A^{{-1}}[e^{{A Δt}} − I] B z RMSE {composed_error}: got {composed}"
    );
    assert_eq!(
        refuse_time_independent_effect_as_continuous_intercept(increment, effect),
        Err(PsychometricError::TimeIndependentEffectIsNotContinuousIntercept)
    );
    assert_eq!(
        refuse_time_independent_effect_as_time_dependent_impulse(increment, impulse),
        Err(PsychometricError::TimeIndependentEffectIsNotTimeDependentImpulse)
    );
    assert_eq!(
        refuse_time_independent_effect_as_time_varying_discrete_effect(
            increment,
            equation_fourteen
        ),
        Err(PsychometricError::TimeIndependentEffectIsNotTimeVaryingDiscreteEffect)
    );
    assert_eq!(
        refuse_time_independent_coefficient_as_discrete_effect(effect, increment),
        Err(PsychometricError::TimeIndependentCoefficientIsNotDiscreteEffect)
    );
}

#[test]
fn time_independent_predictor_refuses_overflow_and_non_event_clocks() {
    assert_eq!(
        recover_discrete_time_independent_predictor_effect(
            1e308,
            2.0,
            -0.5,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_latent_mean_with_time_independent_predictor(
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            2.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_latent_mean_with_time_independent_predictor(
            1e308,
            0.0,
            0.0,
            1e308,
            1.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn time_dependent_impulse_carry_recovers_driver_equation_one_two_dissipation() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let elapsed = 1.0_f64;
    let carry = recover_time_dependent_predictor_impulse_carry(
        effect,
        predictor,
        drift,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("tdpred-carry");
    let expected = (-0.5_f64).exp() * 1.2;
    let error = rmse(&[expected], &[carry]);
    assert!(
        error < 1e-15,
        "Driver Eq. 1–2 impulse carry RMSE {error}: got {carry}"
    );
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
    let intercept_effect =
        recover_discrete_continuous_intercept_effect(effect, drift, delta, LagClock::EventTime)
            .expect("cint");
    let time_independent = recover_discrete_time_independent_predictor_effect(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("tipred");
    let equation_fourteen = recover_discrete_time_varying_predictor_effect(
        effect,
        delta,
        delta,
        delta,
        LagClock::EventTime,
    )
    .expect("eq14");
    assert!(rmse(&[carry], &[impulse]) > rmse(&[expected], &[carry]));
    assert!(rmse(&[carry], &[intercept_effect]) > rmse(&[expected], &[carry]));
    assert!(rmse(&[carry], &[time_independent]) > rmse(&[expected], &[carry]));
    assert!(rmse(&[carry], &[equation_fourteen]) > rmse(&[expected], &[carry]));
    let composed = recover_discrete_latent_mean_with_impulse_carry(
        1.0,
        drift,
        0.3,
        effect,
        predictor,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("eq3-carry");
    let evolved =
        recover_discrete_latent_mean(1.0, drift, 0.3, delta, LagClock::EventTime).expect("mu-t");
    let composed_error = rmse(&[evolved + carry], &[composed]);
    assert!(
        composed_error < 1e-15,
        "Driver Eq. 1–2 μ_t + e^{{A(t−u)}} M x RMSE {composed_error}: got {composed}"
    );
    assert_eq!(
        refuse_time_dependent_impulse_carry_as_contemporaneous_impulse(carry, impulse),
        Err(PsychometricError::TimeDependentImpulseCarryIsNotContemporaneousImpulse)
    );
    assert_eq!(
        refuse_time_dependent_impulse_carry_as_continuous_intercept(carry, effect),
        Err(PsychometricError::TimeDependentImpulseCarryIsNotContinuousIntercept)
    );
    assert_eq!(
        refuse_time_dependent_impulse_carry_as_time_independent_effect(carry, time_independent),
        Err(PsychometricError::TimeDependentImpulseCarryIsNotTimeIndependentEffect)
    );
    assert_eq!(
        refuse_time_dependent_impulse_carry_as_time_varying_discrete_effect(
            carry,
            equation_fourteen
        ),
        Err(PsychometricError::TimeDependentImpulseCarryIsNotTimeVaryingDiscreteEffect)
    );
}

#[test]
fn time_dependent_impulse_carry_refuses_overflow_and_non_event_clocks() {
    assert_eq!(
        recover_time_dependent_predictor_impulse_carry(
            1e308,
            2.0,
            -0.5,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_time_dependent_predictor_impulse_carry(
            0.4,
            3.0,
            -0.5,
            2.0,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_time_dependent_predictor_impulse_carry(
            0.4,
            3.0,
            -0.5,
            2.0,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_latent_mean_with_impulse_carry(
            1e308,
            0.0,
            0.0,
            1e308,
            1.0,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn discrete_observed_mean_with_impulse_carry_recovers_driver_equation_five() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let elapsed = 1.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_impulse_carry(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("eq5-carry-mean");
    let carried = recover_discrete_latent_mean_with_impulse_carry(
        initial,
        drift,
        intercept,
        effect,
        predictor,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("carried");
    let expected = manifest_mean + loading * carried;
    let error = rmse(&[expected], &[observed]);
    assert!(
        error < 1e-15,
        "Driver Eq. 5 of Eq. 1–2 impulse carry RMSE {error}"
    );
    let evolved_observed = recover_discrete_observed_mean(
        loading,
        initial,
        drift,
        intercept,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-eq5-mean");
    let evolved_error = rmse(&[expected], &[evolved_observed]);
    assert!(
        evolved_error > error,
        "τ + λ μ_t is not impulse-carry E(y_t): RMSE {evolved_error} must exceed {error}"
    );
    let intercept_error = rmse(&[expected], &[manifest_mean]);
    assert!(
        intercept_error > error,
        "MANIFESTMEANS is not impulse-carry E(y_t): RMSE {intercept_error} must exceed {error}"
    );
    let latent_error = rmse(&[expected], &[carried]);
    assert!(
        latent_error > error,
        "carried latent mean is not E(y_t): RMSE {latent_error} must exceed {error}"
    );
}

#[test]
fn discrete_observed_mean_with_impulse_carry_refuses_evolved_mean_and_contemporaneous() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let elapsed = 1.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_impulse_carry(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("eq5-carry-mean");
    let carried = recover_discrete_latent_mean_with_impulse_carry(
        initial,
        drift,
        intercept,
        effect,
        predictor,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("carried");
    let expected = manifest_mean + loading * carried;
    let error = rmse(&[expected], &[observed]);
    let evolved_observed = recover_discrete_observed_mean(
        loading,
        initial,
        drift,
        intercept,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-eq5-mean");
    let contemporaneous_latent = recover_discrete_latent_mean_with_impulse(
        initial,
        drift,
        intercept,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("mx");
    let contemporaneous =
        recover_manifest_observed_mean(loading, contemporaneous_latent, manifest_mean)
            .expect("eq5-mx");
    let contemporaneous_error = rmse(&[expected], &[contemporaneous]);
    assert!(
        contemporaneous_error > error,
        "τ + λ(μ_t + m x) is not impulse-carry E(y_t): RMSE {contemporaneous_error} must exceed {error}"
    );
    let zero_loading = recover_discrete_observed_mean_with_impulse_carry(
        0.0,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("lambda0");
    let dropped_error = rmse(&[expected], &[zero_loading]);
    assert!(
        dropped_error > error,
        "zero loading is τ, not τ + λ(μ_t + carry): RMSE {dropped_error} must exceed {error}"
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_impulse_carry_observed_mean(evolved_observed, observed),
        Err(PsychometricError::EvolvedObservedMeanIsNotImpulseCarryObservedMean)
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(carried, observed),
        Err(PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, observed),
        Err(PsychometricError::ManifestMeansIsNotObservedMean)
    );
}

#[test]
fn discrete_observed_mean_with_impulse_carry_refuses_overflow_and_non_event_clocks() {
    assert_eq!(
        recover_discrete_observed_mean_with_impulse_carry(
            1e308,
            2.0,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_impulse_carry(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_impulse_carry(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_impulse_carry(
            1e308,
            0.0,
            0.0,
            0.0,
            1e308,
            1.0,
            0.0,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    let scaled = recover_discrete_observed_mean_with_impulse_carry(
        1e308,
        1e-308,
        0.0,
        0.0,
        0.0,
        3.0,
        0.0,
        2.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("scale");
    assert!(
        (scaled - 1.0).abs() < 1e-15,
        "Driver Eq. 5 of Eq. 1–2 carry must keep λ=1e308, μ=1e-308: got {scaled}"
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
