//! Scientific claim boundaries for compositional coordinates and posterior draws.

use psychometric_core::{
    ClusteredEventScore, ClusteredScore, IndicatorKind, LagClock, LaggedWithinResidual,
    center_within_cluster_event_lags, ordinary_least_squares_slope,
    posterior_draw_point_estimate_mean, recover_asymptotic_continuous_intercept,
    recover_asymptotic_time_independent_predictor_effect,
    recover_asymptotic_time_independent_predictor_variance,
    recover_cluster_mean_within_between_slopes, recover_discrete_constant_predictor_effect,
    recover_discrete_continuous_intercept_effect, recover_discrete_lagged_latent_covariance,
    recover_discrete_latent_mean, recover_discrete_latent_mean_with_extra_process,
    recover_discrete_latent_mean_with_extra_process_after,
    recover_discrete_latent_mean_with_impulse, recover_discrete_latent_mean_with_impulse_carry,
    recover_discrete_latent_mean_with_initial_time_dependent_predictor,
    recover_discrete_latent_mean_with_initial_time_independent_predictor,
    recover_discrete_latent_mean_with_time_independent_predictor, recover_discrete_latent_variance,
    recover_discrete_observed_mean, recover_discrete_observed_mean_with_extra_process,
    recover_discrete_observed_mean_with_extra_process_after,
    recover_discrete_observed_mean_with_impulse, recover_discrete_observed_mean_with_impulse_carry,
    recover_discrete_observed_mean_with_initial_time_dependent_predictor,
    recover_discrete_observed_mean_with_initial_time_independent_predictor,
    recover_discrete_observed_mean_with_time_independent_predictor, recover_discrete_process_noise,
    recover_discrete_time_independent_predictor_effect,
    recover_discrete_time_varying_predictor_effect, recover_initial_time_dependent_predictor_carry,
    recover_initial_time_dependent_predictor_effect,
    recover_initial_time_independent_predictor_carry,
    recover_initial_time_independent_predictor_effect,
    recover_irregular_centered_residual_log_rate, recover_level_change_continuous_intercept,
    recover_level_change_discrete_increment, recover_level_change_extra_process_contribution,
    recover_level_change_extra_process_contribution_after, recover_loading_point_estimate_mean,
    recover_manifest_lagged_observed_covariance, recover_manifest_observed_mean,
    recover_manifest_observed_variance, recover_manifest_trait_plus_state_observed_variance,
    recover_standardised_asymptotic_continuous_intercept,
    recover_standardised_asymptotic_diffusion, recover_standardised_continuous_intercept,
    recover_standardised_discrete_continuous_intercept, recover_standardised_initial_latent_mean,
    recover_standardised_initial_latent_variance, recover_standardised_manifest_mean,
    recover_standardised_manifest_trait_variance, recover_standardised_manifest_variance,
    recover_standardised_trait_variance, recover_stationary_initial_latent_mean,
    recover_stationary_initial_latent_variance, recover_stationary_initial_observed_mean,
    recover_stationary_initial_observed_variance, recover_stationary_lagged_latent_covariance,
    recover_stationary_lagged_observed_covariance, recover_stationary_latent_variance,
    recover_stationary_later_latent_variance, recover_stationary_later_observed_variance,
    recover_time_dependent_predictor_impulse, recover_time_dependent_predictor_impulse_carry,
    recover_trait_plus_state_lagged_covariance, recover_trait_plus_state_latent_variance,
    recover_within_cluster_irregular_residual_log_rate,
    recover_within_residual_event_time_log_rate,
    refuse_after_extra_process_contribution_as_observed_mean,
    refuse_after_extra_process_latent_mean_as_observed_mean,
    refuse_asymptotic_continuous_intercept_as_asymptotic_time_independent_effect,
    refuse_asymptotic_continuous_intercept_as_continuous_intercept,
    refuse_asymptotic_continuous_intercept_as_discrete_increment,
    refuse_asymptotic_continuous_intercept_as_initial_latent_mean,
    refuse_asymptotic_continuous_intercept_observed_mean_as_stationary_initial_observed_mean,
    refuse_asymptotic_standardised_continuous_intercept_as_standardised_continuous_intercept,
    refuse_asymptotic_standardised_continuous_intercept_as_standardised_discrete_continuous_intercept,
    refuse_asymptotic_time_independent_effect_as_coefficient,
    refuse_asymptotic_time_independent_effect_as_continuous_intercept,
    refuse_asymptotic_time_independent_effect_as_discrete_effect,
    refuse_asymptotic_time_independent_effect_as_time_dependent_impulse,
    refuse_asymptotic_time_independent_variance_as_asymptotic_effect,
    refuse_asymptotic_time_independent_variance_as_stationary_within_subject,
    refuse_asymptotic_time_independent_variance_as_trait_variance,
    refuse_continuous_intercept_as_discrete_mean_increment,
    refuse_continuous_intercept_as_initial_latent_mean,
    refuse_continuous_intercept_as_manifest_means,
    refuse_cwc_residual_log_rate_as_raw_process_drift,
    refuse_discrete_standardised_continuous_intercept_as_standardised_asymptotic_continuous_intercept,
    refuse_discrete_standardised_continuous_intercept_as_standardised_continuous_intercept,
    refuse_evolved_observed_mean_as_after_extra_process_observed_mean,
    refuse_evolved_observed_mean_as_extra_process_observed_mean,
    refuse_evolved_observed_mean_as_impulse_carry_observed_mean,
    refuse_evolved_observed_mean_as_impulse_observed_mean,
    refuse_evolved_observed_mean_as_initial_time_dependent_observed_mean,
    refuse_evolved_observed_mean_as_initial_time_independent_observed_mean,
    refuse_evolved_observed_mean_as_stationary_initial_observed_mean,
    refuse_evolved_observed_mean_as_time_independent_observed_mean,
    refuse_evolved_observed_variance_as_stationary_initial_observed_variance,
    refuse_extra_process_contribution_as_observed_mean,
    refuse_extra_process_latent_mean_as_observed_mean,
    refuse_extra_process_observed_mean_as_after_extra_process_observed_mean,
    refuse_finite_interval_process_noise_as_stationary_variance,
    refuse_impulse_carry_observed_mean_as_after_extra_process_observed_mean,
    refuse_impulse_carry_observed_mean_as_initial_time_dependent_observed_mean,
    refuse_impulse_carry_observed_mean_as_initial_time_independent_observed_mean,
    refuse_impulse_carry_observed_mean_as_time_independent_observed_mean,
    refuse_impulse_observed_mean_as_extra_process_observed_mean,
    refuse_impulse_observed_mean_as_impulse_carry_observed_mean,
    refuse_impulse_observed_mean_as_initial_time_dependent_observed_mean,
    refuse_impulse_observed_mean_as_initial_time_independent_observed_mean,
    refuse_impulse_observed_mean_as_time_independent_observed_mean,
    refuse_initial_latent_mean_as_evolved_mean,
    refuse_initial_observed_mean_as_evolved_observed_mean,
    refuse_initial_observed_mean_as_stationary_initial_observed_mean,
    refuse_initial_observed_variance_as_stationary_initial_observed_variance,
    refuse_initial_time_dependent_carry_as_impulse_carry,
    refuse_initial_time_dependent_carry_as_initial_effect,
    refuse_initial_time_dependent_coefficient_as_initial_effect,
    refuse_initial_time_dependent_effect_as_contemporaneous_impulse,
    refuse_initial_time_dependent_effect_as_continuous_intercept,
    refuse_initial_time_dependent_effect_as_initial_time_independent_effect,
    refuse_initial_time_dependent_effect_as_process_increment,
    refuse_initial_time_independent_carry_as_initial_effect,
    refuse_initial_time_independent_coefficient_as_initial_effect,
    refuse_initial_time_independent_effect_as_continuous_intercept,
    refuse_initial_time_independent_effect_as_process_increment,
    refuse_initial_time_independent_effect_as_time_dependent_impulse,
    refuse_initial_time_independent_observed_mean_as_initial_time_dependent_observed_mean,
    refuse_initial_time_independent_variance_as_standardised_trait_variance,
    refuse_latent_lagged_covariance_as_observed_covariance, refuse_latent_mean_as_observed_mean,
    refuse_latent_variance_as_observed_variance, refuse_level_change_extra_process_as_impulse,
    refuse_level_change_extra_process_as_increment, refuse_level_change_extra_process_as_intercept,
    refuse_level_change_increment_as_impulse, refuse_level_change_increment_as_intercept,
    refuse_level_change_increment_as_process_increment,
    refuse_level_change_intercept_as_free_continuous_intercept,
    refuse_level_change_intercept_as_impulse, refuse_level_change_intercept_as_process_increment,
    refuse_manifest_means_as_observed_mean, refuse_manifest_trait_variance_as_measurement_error,
    refuse_measurement_error_as_lagged_observed_covariance,
    refuse_measurement_error_as_observed_variance,
    refuse_measurement_error_as_standardised_manifest_trait_variance,
    refuse_measurement_error_as_stationary_lagged_observed_covariance,
    refuse_measurement_error_as_stationary_later_observed_variance,
    refuse_observed_scaled_manifest_mean_as_standardised_manifest_mean,
    refuse_observed_variance_as_standardised_manifest_variance,
    refuse_process_noise_as_unconditional_variance,
    refuse_standardised_asymptotic_diffusion_as_standardised_initial_latent_variance,
    refuse_standardised_continuous_diffusion_as_standardised_asymptotic_diffusion,
    refuse_standardised_continuous_intercept_as_standardised_asymptotic_continuous_intercept,
    refuse_standardised_continuous_intercept_as_standardised_discrete_continuous_intercept,
    refuse_standardised_initial_latent_mean_as_standardised_initial_latent_variance,
    refuse_standardised_initial_latent_variance_as_standardised_asymptotic_diffusion,
    refuse_standardised_initial_latent_variance_as_standardised_initial_latent_mean,
    refuse_standardised_initial_latent_variance_as_standardised_trait_variance,
    refuse_standardised_manifest_trait_variance_as_standardised_manifest_variance,
    refuse_standardised_manifest_variance_as_standardised_manifest_mean,
    refuse_standardised_time_independent_predictor_variance_as_standardised_asymptotic_diffusion,
    refuse_standardised_trait_variance_as_standardised_manifest_trait_variance,
    refuse_stationary_initial_latent_mean_as_asymptotic_continuous_intercept,
    refuse_stationary_initial_latent_mean_as_asymptotic_time_independent_effect,
    refuse_stationary_initial_latent_mean_as_discrete_mean,
    refuse_stationary_initial_latent_mean_as_initial_latent_mean,
    refuse_stationary_initial_latent_mean_as_observed_mean,
    refuse_stationary_initial_latent_variance_as_asymptotic_time_independent_variance,
    refuse_stationary_initial_latent_variance_as_discrete_variance,
    refuse_stationary_initial_latent_variance_as_initial_latent_variance,
    refuse_stationary_initial_latent_variance_as_observed_variance,
    refuse_stationary_initial_latent_variance_as_stationary_within_subject,
    refuse_stationary_initial_latent_variance_as_trait_variance,
    refuse_stationary_initial_observed_mean_as_manifest_means,
    refuse_stationary_initial_observed_variance_as_measurement_error,
    refuse_stationary_initial_observed_variance_as_stationary_lagged_observed_covariance,
    refuse_stationary_lagged_latent_covariance_as_decayed_stationary_variance,
    refuse_stationary_lagged_latent_covariance_as_observed_covariance,
    refuse_stationary_lagged_latent_covariance_as_stationary_initial_latent_variance,
    refuse_stationary_lagged_observed_covariance_as_stationary_later_observed_variance,
    refuse_stationary_later_latent_variance_as_discrete_variance,
    refuse_stationary_later_latent_variance_as_lagged_covariance,
    refuse_stationary_later_latent_variance_as_observed_variance,
    refuse_stationary_later_latent_variance_as_process_noise,
    refuse_stationary_within_subject_observed_variance_as_stationary_initial_observed_variance,
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
    refuse_time_independent_observed_mean_as_initial_time_dependent_observed_mean,
    refuse_time_independent_observed_mean_as_initial_time_independent_observed_mean,
    refuse_trait_plus_state_lagged_covariance_as_stationary_lagged_latent_covariance,
    refuse_trait_scaled_continuous_intercept_as_standardised_continuous_intercept,
    refuse_trait_variance_as_process_noise, refuse_trait_variance_as_stationary_within_subject,
    refuse_unstandardised_asymptotic_continuous_intercept_as_standardised_asymptotic_continuous_intercept,
    refuse_unstandardised_asymptotic_diffusion_as_standardised_asymptotic_diffusion,
    refuse_unstandardised_continuous_intercept_as_standardised_continuous_intercept,
    refuse_unstandardised_discrete_continuous_intercept_as_standardised_discrete_continuous_intercept,
    refuse_unstandardised_initial_latent_mean_as_standardised_initial_latent_mean,
    refuse_unstandardised_initial_latent_variance_as_standardised_initial_latent_variance,
    refuse_unstandardised_manifest_mean_as_standardised_manifest_mean,
    refuse_unstandardised_manifest_trait_variance_as_standardised_manifest_trait_variance,
    refuse_unstandardised_manifest_variance_as_standardised_manifest_variance,
    refuse_unstandardised_trait_variance_as_standardised_trait_variance,
    refuse_within_subject_scaled_initial_latent_mean_as_standardised_initial_latent_mean,
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
    let pairwise = recover_within_cluster_irregular_residual_log_rate(&raw, LagClock::EventTime)
        .expect("pair");
    let extracted = center_within_cluster_event_lags(&raw, LagClock::EventTime).expect("extract");
    assert_eq!(
        recover_irregular_centered_residual_log_rate(&extracted, LagClock::EventTime),
        Err(psychometric_core::PsychometricError::InvalidNumericInput),
        "unfiltered CWC pairs straddle zero and have no real logarithm"
    );
    let admissible: Vec<_> = extracted
        .iter()
        .copied()
        .filter(|pair| {
            pair.earlier_residual != 0.0
                && pair.later_residual != 0.0
                && pair.earlier_residual.is_sign_positive()
                    == pair.later_residual.is_sign_positive()
        })
        .collect();
    let from_pairs = recover_irregular_centered_residual_log_rate(&admissible, LagClock::EventTime)
        .expect("from pairs");
    assert!((pairwise - from_pairs).abs() < 1e-15);
    assert!(
        (pairwise - drift).abs() > 1e-6,
        "Curran & Bauer (2011, pp. 607–608): CWC pairwise-mean {pairwise} must not equal drift {drift}"
    );
    assert_eq!(
        refuse_cwc_residual_log_rate_as_raw_process_drift(cwc, drift),
        Err(psychometric_core::PsychometricError::CwcResidualLogRateIsNotRawProcessDrift)
    );
    assert_eq!(
        refuse_cwc_residual_log_rate_as_raw_process_drift(pairwise, drift),
        Err(psychometric_core::PsychometricError::CwcResidualLogRateIsNotRawProcessDrift)
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
    assert_eq!(
        recover_discrete_process_noise(0.4, -0.5, f64::NAN, LagClock::EventTime),
        Err(psychometric_core::PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_process_noise(-0.1, -0.5, 1.0, LagClock::EventTime),
        Err(psychometric_core::PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_process_noise(f64::NAN, -0.5, 1.0, LagClock::EventTime),
        Err(psychometric_core::PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_process_noise(0.4, f64::NAN, 1.0, LagClock::EventTime),
        Err(psychometric_core::PsychometricError::InvalidNumericInput)
    );
    let constant =
        recover_discrete_constant_predictor_effect(diffusion, drift, delta, LagClock::EventTime)
            .expect("eq 12");
    assert!(
        (discrete - constant).abs() > 1e-3,
        "Driver Eq. 3 Q_Δt is not Voelkle Eq. 12"
    );
}

#[test]
fn process_noise_is_not_the_unconditional_latent_variance() {
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
    assert!(
        (process_noise - latent).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3-4, pp. 4-5): Q_Δt is cov(η_t | η_{{t-1}}), not Var(η_t)"
    );
    assert!((lagged - (drift * delta).exp() * prior).abs() < 1e-15);
    assert!((latent - ((2.0 * drift * delta).exp() * prior + process_noise)).abs() < 1e-15);
    assert_eq!(
        refuse_process_noise_as_unconditional_variance(process_noise, prior),
        Err(psychometric_core::PsychometricError::ProcessNoiseIsConditionalVariance)
    );
}

#[test]
fn finite_interval_process_noise_is_not_the_stationary_variance() {
    let diffusion = 0.4_f64;
    let drift = -0.5_f64;
    let delta = 1.0_f64;
    let process_noise =
        recover_discrete_process_noise(diffusion, drift, delta, LagClock::EventTime).expect("q_dt");
    let stationary =
        recover_stationary_latent_variance(diffusion, drift, LagClock::EventTime).expect("asym");
    assert!(
        (process_noise - stationary).abs() > 1e-3,
        "Driver et al. (2017, Eq. 4 / p. 16): finite-Δt Q_Δt is not asymDIFFUSION"
    );
    let evolved =
        recover_discrete_latent_variance(stationary, diffusion, drift, delta, LagClock::EventTime)
            .expect("invariant");
    assert!((evolved - stationary).abs() < 1e-12);
    assert_eq!(
        refuse_finite_interval_process_noise_as_stationary_variance(process_noise, delta),
        Err(psychometric_core::PsychometricError::FiniteIntervalProcessNoiseIsNotStationary)
    );
    assert_eq!(
        recover_stationary_latent_variance(diffusion, 0.0, LagClock::EventTime),
        Err(psychometric_core::PsychometricError::StationaryVarianceRequiresStableDrift)
    );
}

#[test]
fn trait_variance_is_not_process_noise_or_stationary_within_subject() {
    let trait_variance = 1.5_f64;
    let diffusion = 0.4_f64;
    let drift = -0.5_f64;
    let delta = 1.0_f64;
    let state =
        recover_stationary_latent_variance(diffusion, drift, LagClock::EventTime).expect("state");
    let total = recover_trait_plus_state_latent_variance(trait_variance, state).expect("sum");
    let process_noise =
        recover_discrete_process_noise(diffusion, drift, delta, LagClock::EventTime).expect("q_dt");
    assert!(
        (trait_variance - process_noise).abs() > 1e-3,
        "Driver et al. (2017, §4.3, p. 9): TRAITVAR is not Q_Δt"
    );
    assert!(
        (trait_variance - state).abs() > 1e-3,
        "Driver et al. (2017, §4.3, p. 9): TRAITVAR is not asymDIFFUSION"
    );
    let evolved_as_state =
        recover_discrete_latent_variance(total, diffusion, drift, delta, LagClock::EventTime)
            .expect("wrong");
    assert!(
        (evolved_as_state - total).abs() > 1e-3,
        "Driver et al. (2017, §4.3): evolving trait+state as all-state is not the trait map"
    );
    assert_eq!(
        refuse_trait_variance_as_process_noise(trait_variance, process_noise),
        Err(psychometric_core::PsychometricError::TraitVarianceIsNotProcessNoise)
    );
    assert_eq!(
        refuse_trait_variance_as_stationary_within_subject(trait_variance, state),
        Err(psychometric_core::PsychometricError::TraitVarianceIsNotStationaryWithinSubject)
    );
}

#[test]
fn measurement_error_and_latent_variance_are_not_the_observed_variance() {
    let loading = 2.0_f64;
    let latent = 0.4_f64;
    let measurement_error = 0.1_f64;
    let observed =
        recover_manifest_observed_variance(loading, latent, measurement_error).expect("eq5");
    assert!(
        (measurement_error - observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 / Table 2 p. 12): MANIFESTVAR is not Var(y)"
    );
    assert!(
        (latent - observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): Var(η) is not Var(y)"
    );
    assert_eq!(
        refuse_measurement_error_as_observed_variance(measurement_error, observed),
        Err(psychometric_core::PsychometricError::MeasurementErrorIsNotObservedVariance)
    );
    assert_eq!(
        refuse_latent_variance_as_observed_variance(latent, observed),
        Err(psychometric_core::PsychometricError::LatentVarianceIsNotObservedVariance)
    );
}

#[test]
fn manifest_trait_variance_is_not_measurement_error() {
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
    let without_trait =
        recover_manifest_observed_variance(loading, latent, measurement_error).expect("psi0");
    assert!(
        (without_trait - observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 / Table 2 p. 12): MANIFESTTRAITVAR is not dropped"
    );
    let stuffed =
        recover_manifest_observed_variance(loading, latent, manifest_trait).expect("psi-as-theta");
    assert!(
        (stuffed - observed).abs() > 1e-3,
        "Driver et al. (2017, Table 2 p. 12): MANIFESTTRAITVAR is not MANIFESTVAR"
    );
    let latent_trait =
        recover_manifest_observed_variance(loading, latent + manifest_trait, measurement_error)
            .expect("traitvar");
    assert!(
        (latent_trait - observed).abs() > 1e-3,
        "Driver et al. (2017, Table 2 p. 12): TRAITVAR is latent and scaled by λ²; MANIFESTTRAITVAR is not"
    );
    assert_eq!(
        refuse_manifest_trait_variance_as_measurement_error(manifest_trait, measurement_error),
        Err(psychometric_core::PsychometricError::ManifestTraitVarianceIsNotMeasurementError)
    );
}

#[test]
fn lagged_latent_covariance_and_measurement_error_are_not_lagged_observed_covariance() {
    let loading = 2.0_f64;
    let lagged = 0.4_f64;
    let manifest_trait = 0.5_f64;
    let measurement_error = 0.1_f64;
    let observed = recover_manifest_lagged_observed_covariance(loading, lagged, manifest_trait)
        .expect("eq5-lag");
    assert!(
        (lagged - observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): cov(η_t, η_{{t-1}}) is not cov(y_t, y_{{t-1}})"
    );
    assert!(
        (measurement_error - observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): MANIFESTVAR does not enter lagged observed covariance"
    );
    assert_eq!(
        refuse_latent_lagged_covariance_as_observed_covariance(lagged, observed),
        Err(psychometric_core::PsychometricError::LatentLaggedCovarianceIsNotObservedCovariance)
    );
    assert_eq!(
        refuse_measurement_error_as_lagged_observed_covariance(measurement_error, observed),
        Err(psychometric_core::PsychometricError::MeasurementErrorIsNotLaggedObservedCovariance)
    );
}

#[test]
fn manifest_means_and_latent_mean_are_not_observed_mean() {
    let loading = 2.0_f64;
    let latent_mean = 0.4_f64;
    let manifest_mean = 0.5_f64;
    let continuous_intercept = 0.3_f64;
    let observed =
        recover_manifest_observed_mean(loading, latent_mean, manifest_mean).expect("eq5-mean");
    assert!(
        (manifest_mean - observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 / Table 2 p. 12): MANIFESTMEANS is not E(y)"
    );
    assert!(
        (latent_mean - observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): E(η) is not E(y)"
    );
    assert!(
        (continuous_intercept - manifest_mean).abs() > 1e-3,
        "Driver et al. (2017, Table 2 p. 12): CINT is not MANIFESTMEANS"
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, observed),
        Err(psychometric_core::PsychometricError::ManifestMeansIsNotObservedMean)
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(latent_mean, observed),
        Err(psychometric_core::PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_continuous_intercept_as_manifest_means(continuous_intercept, manifest_mean),
        Err(psychometric_core::PsychometricError::ContinuousInterceptIsNotManifestMeans)
    );
}

#[test]
fn initial_latent_mean_and_continuous_intercept_are_not_evolved_mean() {
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let evolved =
        recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
            .expect("eq3-mean");
    let increment =
        recover_discrete_continuous_intercept_effect(intercept, drift, delta, LagClock::EventTime)
            .expect("cint");
    assert!(
        (initial - evolved).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3 / Table 2 p. 12): T0MEANS is not μ_t"
    );
    assert!(
        (intercept - increment).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3 / Table 2 p. 12): CINT is not the discrete mean increment"
    );
    assert!(
        (intercept - initial).abs() > 1e-3,
        "Driver et al. (2017, Table 2 p. 12): CINT is not T0MEANS"
    );
    assert_eq!(
        refuse_initial_latent_mean_as_evolved_mean(initial, evolved),
        Err(psychometric_core::PsychometricError::InitialLatentMeanIsNotEvolvedMean)
    );
    assert_eq!(
        refuse_continuous_intercept_as_discrete_mean_increment(intercept, increment),
        Err(psychometric_core::PsychometricError::ContinuousInterceptIsNotDiscreteMeanIncrement)
    );
    assert_eq!(
        refuse_continuous_intercept_as_initial_latent_mean(intercept, initial),
        Err(psychometric_core::PsychometricError::ContinuousInterceptIsNotInitialLatentMean)
    );
}

#[test]
fn first_occasion_observed_mean_is_not_evolved_observed_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
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
    let first_occasion =
        recover_manifest_observed_mean(loading, initial, manifest_mean).expect("t0");
    let evolved_latent =
        recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
            .expect("mu-t");
    assert!(
        (first_occasion - evolved_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Eq. 3): τ + λ μ_0 is not E(y_t)"
    );
    assert!(
        (manifest_mean - evolved_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 / Table 2 p. 12): MANIFESTMEANS is not E(y_t)"
    );
    assert!(
        (evolved_latent - evolved_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): μ_t is not E(y_t)"
    );
    assert_eq!(
        refuse_initial_observed_mean_as_evolved_observed_mean(first_occasion, evolved_observed),
        Err(psychometric_core::PsychometricError::InitialObservedMeanIsNotEvolvedObservedMean)
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(evolved_latent, evolved_observed),
        Err(psychometric_core::PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, evolved_observed),
        Err(psychometric_core::PsychometricError::ManifestMeansIsNotObservedMean)
    );
}

#[test]
fn evolved_observed_mean_is_not_impulse_observed_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let impulse_observed = recover_discrete_observed_mean_with_impulse(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-impulse-mean");
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
    let composed = recover_discrete_latent_mean_with_impulse(
        initial,
        drift,
        intercept,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("mx");
    let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        1.0,
        LagClock::EventTime,
    )
    .expect("eq5-carry-mean");
    assert!(
        (evolved_observed - impulse_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Eq. 3 impulse): τ + λ μ_t is not contemporaneous-impulse E(y_t)"
    );
    assert!(
        (manifest_mean - impulse_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 / Table 2 p. 12): MANIFESTMEANS is not contemporaneous-impulse E(y_t)"
    );
    assert!(
        (composed - impulse_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): evolved-plus-impulse latent mean is not E(y_t)"
    );
    assert!(
        (carried_observed - impulse_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Eq. 1–2): τ + λ(μ_t + carry) is not contemporaneous-impulse E(y_t)"
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_impulse_observed_mean(evolved_observed, impulse_observed),
        Err(psychometric_core::PsychometricError::EvolvedObservedMeanIsNotImpulseObservedMean)
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_impulse_carry_observed_mean(
            impulse_observed,
            carried_observed
        ),
        Err(psychometric_core::PsychometricError::ImpulseObservedMeanIsNotImpulseCarryObservedMean)
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(composed, impulse_observed),
        Err(psychometric_core::PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, impulse_observed),
        Err(psychometric_core::PsychometricError::ManifestMeansIsNotObservedMean)
    );
}

#[test]
fn time_dependent_impulse_is_not_cint_tipred_or_equation_fourteen() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
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
    let evolved =
        recover_discrete_latent_mean(1.0, drift, 0.3, delta, LagClock::EventTime).expect("mu-t");
    let composed = recover_discrete_latent_mean_with_impulse(
        1.0,
        drift,
        0.3,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-impulse");
    assert!(
        (impulse - effect).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3 / Table 2 p. 12): TDPREDEFFECT is not CINT"
    );
    assert!(
        (impulse - intercept_effect).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): M x is not the time-independent discrete effect"
    );
    assert!(
        (impulse - equation_fourteen).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): M x is not Voelkle Eq. 14"
    );
    assert!(
        (composed - evolved).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): μ_t is not μ_t + M x"
    );
    assert_eq!(
        refuse_time_dependent_impulse_as_continuous_intercept(impulse, effect),
        Err(psychometric_core::PsychometricError::TimeDependentImpulseIsNotContinuousIntercept)
    );
    assert_eq!(
        refuse_time_dependent_impulse_as_time_independent_effect(impulse, intercept_effect),
        Err(psychometric_core::PsychometricError::TimeDependentImpulseIsNotTimeIndependentEffect)
    );
    assert_eq!(
        refuse_time_dependent_impulse_as_time_varying_discrete_effect(impulse, equation_fourteen),
        Err(psychometric_core::PsychometricError::TimeDependentImpulseIsNotTimeVaryingDiscreteEffect)
    );
}

#[test]
fn time_independent_predictor_is_not_cint_impulse_equation_fourteen_or_coefficient() {
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
    let evolved =
        recover_discrete_latent_mean(1.0, drift, 0.3, delta, LagClock::EventTime).expect("mu-t");
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
    assert!(
        (increment - effect).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3 / Table 2 p. 12): TIPREDEFFECT is not the discrete increment"
    );
    assert!(
        (increment - intercept_effect).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): A^{{-1}}[e^{{A Δt}} − I] B z is not CINT"
    );
    assert!(
        (increment - impulse).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): A^{{-1}}[e^{{A Δt}} − I] B z is not M x"
    );
    assert!(
        (increment - equation_fourteen).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): A^{{-1}}[e^{{A Δt}} − I] B z is not Voelkle Eq. 14"
    );
    assert!(
        (composed - evolved).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): μ_t is not μ_t + A^{{-1}}[e^{{A Δt}} − I] B z"
    );
    assert_eq!(
        refuse_time_independent_effect_as_continuous_intercept(increment, effect),
        Err(psychometric_core::PsychometricError::TimeIndependentEffectIsNotContinuousIntercept)
    );
    assert_eq!(
        refuse_time_independent_effect_as_time_dependent_impulse(increment, impulse),
        Err(psychometric_core::PsychometricError::TimeIndependentEffectIsNotTimeDependentImpulse)
    );
    assert_eq!(
        refuse_time_independent_effect_as_time_varying_discrete_effect(increment, equation_fourteen),
        Err(psychometric_core::PsychometricError::TimeIndependentEffectIsNotTimeVaryingDiscreteEffect)
    );
    assert_eq!(
        refuse_time_independent_coefficient_as_discrete_effect(effect, increment),
        Err(psychometric_core::PsychometricError::TimeIndependentCoefficientIsNotDiscreteEffect)
    );
}

#[test]
fn initial_time_independent_predictor_is_not_process_increment_cint_or_impulse() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let shift =
        recover_initial_time_independent_predictor_effect(effect, predictor).expect("t0-tipred");
    let carry = recover_initial_time_independent_predictor_carry(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("t0-carry");
    let increment = recover_discrete_time_independent_predictor_effect(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("tipred");
    let intercept_effect =
        recover_discrete_continuous_intercept_effect(effect, drift, delta, LagClock::EventTime)
            .expect("cint");
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
    let evolved =
        recover_discrete_latent_mean(1.0, drift, 0.3, delta, LagClock::EventTime).expect("mu-t");
    let composed = recover_discrete_latent_mean_with_initial_time_independent_predictor(
        1.0,
        drift,
        0.3,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-t0tipred");
    assert!(
        (shift - effect).abs() > 1e-3,
        "Driver et al. (2017, Table 3 p. 13): T0TIPREDEFFECT is not t0_b z"
    );
    assert!(
        (shift - increment).abs() > 1e-3,
        "Driver et al. (2017, Table 3 / Eq. 3): t0_b z is not A^{{-1}}[e^{{A Δt}} − I] B z"
    );
    assert!(
        (carry - shift).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): e^{{A Δt}} t0_b z is not t0_b z"
    );
    assert!(
        (carry - increment).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): e^{{A Δt}} t0_b z is not A^{{-1}}[e^{{A Δt}} − I] B z"
    );
    assert!(
        (shift - intercept_effect).abs() > 1e-3,
        "Driver et al. (2017, Table 3): t0_b z is not CINT"
    );
    assert!(
        (composed - evolved).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): μ_t is not μ_t + e^{{A Δt}} t0_b z"
    );
    assert_eq!(
        refuse_initial_time_independent_effect_as_process_increment(shift, increment),
        Err(
            psychometric_core::PsychometricError::InitialTimeIndependentEffectIsNotProcessIncrement
        )
    );
    assert_eq!(
        refuse_initial_time_independent_carry_as_initial_effect(carry, shift),
        Err(psychometric_core::PsychometricError::InitialTimeIndependentCarryIsNotInitialEffect)
    );
    assert_eq!(
        refuse_initial_time_independent_effect_as_continuous_intercept(shift, effect),
        Err(
            psychometric_core::PsychometricError::InitialTimeIndependentEffectIsNotContinuousIntercept
        )
    );
    assert_eq!(
        refuse_initial_time_independent_effect_as_time_dependent_impulse(shift, impulse),
        Err(
            psychometric_core::PsychometricError::InitialTimeIndependentEffectIsNotTimeDependentImpulse
        )
    );
    assert_eq!(
        refuse_initial_time_independent_coefficient_as_initial_effect(effect, shift),
        Err(
            psychometric_core::PsychometricError::InitialTimeIndependentCoefficientIsNotInitialEffect
        )
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn initial_time_dependent_predictor_is_not_impulse_cint_process_or_t0_tipred() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let shift =
        recover_initial_time_dependent_predictor_effect(effect, predictor).expect("t0-tdpred");
    let carry = recover_initial_time_dependent_predictor_carry(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("t0-td-carry");
    let increment = recover_discrete_time_independent_predictor_effect(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("tipred");
    let intercept_effect =
        recover_discrete_continuous_intercept_effect(effect, drift, delta, LagClock::EventTime)
            .expect("cint");
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
    let tipred_shift =
        recover_initial_time_independent_predictor_effect(effect, predictor).expect("t0-tipred");
    let impulse_carry = recover_time_dependent_predictor_impulse_carry(
        effect,
        predictor,
        drift,
        delta,
        1.0,
        LagClock::EventTime,
    )
    .expect("td-carry");
    let evolved =
        recover_discrete_latent_mean(1.0, drift, 0.3, delta, LagClock::EventTime).expect("mu-t");
    let composed = recover_discrete_latent_mean_with_initial_time_dependent_predictor(
        1.0,
        drift,
        0.3,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-t0tdpred");
    assert!(
        (shift - effect).abs() > 1e-3,
        "Driver et al. (2017, Table 3 p. 13): T0TDPREDEFFECT is not t0_m x0"
    );
    assert!(
        (shift - increment).abs() > 1e-3,
        "Driver et al. (2017, Table 3 / Eq. 3): t0_m x0 is not A^{{-1}}[e^{{A Δt}} − I] B z"
    );
    assert!(
        (carry - shift).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): e^{{A Δt}} t0_m x0 is not t0_m x0"
    );
    assert!(
        (carry - impulse_carry).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): e^{{A Δt}} t0_m x0 is not e^{{A(t−u)}} M x"
    );
    assert!(
        (shift - intercept_effect).abs() > 1e-3,
        "Driver et al. (2017, Table 3): t0_m x0 is not CINT"
    );
    assert!(
        (composed - evolved).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3): μ_t is not μ_t + e^{{A Δt}} t0_m x0"
    );
    assert_eq!(
        refuse_initial_time_dependent_effect_as_contemporaneous_impulse(shift, impulse),
        Err(
            psychometric_core::PsychometricError::InitialTimeDependentEffectIsNotContemporaneousImpulse
        )
    );
    assert_eq!(
        refuse_initial_time_dependent_carry_as_initial_effect(carry, shift),
        Err(psychometric_core::PsychometricError::InitialTimeDependentCarryIsNotInitialEffect)
    );
    assert_eq!(
        refuse_initial_time_dependent_effect_as_continuous_intercept(shift, effect),
        Err(
            psychometric_core::PsychometricError::InitialTimeDependentEffectIsNotContinuousIntercept
        )
    );
    assert_eq!(
        refuse_initial_time_dependent_effect_as_process_increment(shift, increment),
        Err(psychometric_core::PsychometricError::InitialTimeDependentEffectIsNotProcessIncrement)
    );
    assert_eq!(
        refuse_initial_time_dependent_effect_as_initial_time_independent_effect(shift, tipred_shift),
        Err(
            psychometric_core::PsychometricError::InitialTimeDependentEffectIsNotInitialTimeIndependentEffect
        )
    );
    assert_eq!(
        refuse_initial_time_dependent_coefficient_as_initial_effect(effect, shift),
        Err(
            psychometric_core::PsychometricError::InitialTimeDependentCoefficientIsNotInitialEffect
        )
    );
    assert_eq!(
        refuse_initial_time_dependent_carry_as_impulse_carry(carry, impulse_carry),
        Err(psychometric_core::PsychometricError::InitialTimeDependentCarryIsNotImpulseCarry)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn evolved_and_process_observed_mean_are_not_initial_time_independent_observed_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let initial_observed = recover_discrete_observed_mean_with_initial_time_independent_predictor(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-t0tipred-mean");
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
    let process_observed = recover_discrete_observed_mean_with_time_independent_predictor(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-tipred-mean");
    let impulse_observed = recover_discrete_observed_mean_with_impulse(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-impulse-mean");
    let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        1.0,
        LagClock::EventTime,
    )
    .expect("eq5-carry-mean");
    let composed = recover_discrete_latent_mean_with_initial_time_independent_predictor(
        initial,
        drift,
        intercept,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-t0tipred");
    assert!(
        (evolved_observed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Table 3 T0TIPREDEFFECT): τ + λ μ_t is not T0TIPREDEFFECT E(y_t)"
    );
    assert!(
        (process_observed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): TIPREDEFFECT E(y_t) is not T0TIPREDEFFECT E(y_t)"
    );
    assert!(
        (impulse_observed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): τ + λ(μ_t + m x) is not T0TIPREDEFFECT E(y_t)"
    );
    assert!(
        (carried_observed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): τ + λ(μ_t + carry) is not T0TIPREDEFFECT E(y_t)"
    );
    assert!(
        (composed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): evolved-plus-T0TIPRED latent mean is not E(y_t)"
    );
    assert!(
        (manifest_mean - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 / Table 2 p. 12): MANIFESTMEANS is not T0TIPREDEFFECT E(y_t)"
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_initial_time_independent_observed_mean(
            evolved_observed,
            initial_observed
        ),
        Err(
            psychometric_core::PsychometricError::EvolvedObservedMeanIsNotInitialTimeIndependentObservedMean
        )
    );
    assert_eq!(
        refuse_time_independent_observed_mean_as_initial_time_independent_observed_mean(
            process_observed,
            initial_observed
        ),
        Err(
            psychometric_core::PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeIndependentObservedMean
        )
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_initial_time_independent_observed_mean(
            impulse_observed,
            initial_observed
        ),
        Err(
            psychometric_core::PsychometricError::ImpulseObservedMeanIsNotInitialTimeIndependentObservedMean
        )
    );
    assert_eq!(
        refuse_impulse_carry_observed_mean_as_initial_time_independent_observed_mean(
            carried_observed,
            initial_observed
        ),
        Err(
            psychometric_core::PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeIndependentObservedMean
        )
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(composed, initial_observed),
        Err(psychometric_core::PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, initial_observed),
        Err(psychometric_core::PsychometricError::ManifestMeansIsNotObservedMean)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn evolved_and_process_observed_mean_are_not_initial_time_dependent_observed_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let initial_observed = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-t0tdpred-mean");
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
    let process_observed = recover_discrete_observed_mean_with_time_independent_predictor(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-tipred-mean");
    let impulse_observed = recover_discrete_observed_mean_with_impulse(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-impulse-mean");
    let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        1.0,
        LagClock::EventTime,
    )
    .expect("eq5-carry-mean");
    let tipred_observed = recover_discrete_observed_mean_with_initial_time_independent_predictor(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-t0tipred-mean");
    let composed = recover_discrete_latent_mean_with_initial_time_dependent_predictor(
        initial,
        drift,
        intercept,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-t0tdpred");
    assert!(
        (evolved_observed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Table 3 T0TDPREDEFFECT): τ + λ μ_t is not T0TDPREDEFFECT E(y_t)"
    );
    assert!(
        (process_observed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): TIPREDEFFECT E(y_t) is not T0TDPREDEFFECT E(y_t)"
    );
    assert!(
        (impulse_observed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): τ + λ(μ_t + m x) is not T0TDPREDEFFECT E(y_t)"
    );
    assert!(
        (carried_observed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): τ + λ(μ_t + carry) is not T0TDPREDEFFECT E(y_t)"
    );
    assert!(
        (composed - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): evolved-plus-T0TDPRED latent mean is not E(y_t)"
    );
    assert!(
        (manifest_mean - initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 / Table 2 p. 12): MANIFESTMEANS is not T0TDPREDEFFECT E(y_t)"
    );
    // Same numbers as T0TIPRED yield the same product; Table 3 names a different matrix.
    assert!((tipred_observed - initial_observed).abs() < 1e-15);
    assert_eq!(
        refuse_evolved_observed_mean_as_initial_time_dependent_observed_mean(
            evolved_observed,
            initial_observed
        ),
        Err(
            psychometric_core::PsychometricError::EvolvedObservedMeanIsNotInitialTimeDependentObservedMean
        )
    );
    assert_eq!(
        refuse_time_independent_observed_mean_as_initial_time_dependent_observed_mean(
            process_observed,
            initial_observed
        ),
        Err(
            psychometric_core::PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean
        )
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_initial_time_dependent_observed_mean(
            impulse_observed,
            initial_observed
        ),
        Err(
            psychometric_core::PsychometricError::ImpulseObservedMeanIsNotInitialTimeDependentObservedMean
        )
    );
    assert_eq!(
        refuse_impulse_carry_observed_mean_as_initial_time_dependent_observed_mean(
            carried_observed,
            initial_observed
        ),
        Err(
            psychometric_core::PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeDependentObservedMean
        )
    );
    assert_eq!(
        refuse_initial_time_independent_observed_mean_as_initial_time_dependent_observed_mean(
            tipred_observed,
            initial_observed
        ),
        Err(
            psychometric_core::PsychometricError::InitialTimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean
        )
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(composed, initial_observed),
        Err(psychometric_core::PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, initial_observed),
        Err(psychometric_core::PsychometricError::ManifestMeansIsNotObservedMean)
    );
}

#[test]
fn time_dependent_impulse_carry_is_not_contemporaneous_cint_tipred_or_equation_fourteen() {
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
    let evolved =
        recover_discrete_latent_mean(1.0, drift, 0.3, delta, LagClock::EventTime).expect("mu-t");
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
    assert!(
        (carry - impulse).abs() > 1e-3,
        "Driver et al. (2017, Eq. 1–2 / §7.2): e^{{A(t−u)}} M x is not the contemporaneous Dirac"
    );
    assert!(
        (carry - intercept_effect).abs() > 1e-3,
        "Driver et al. (2017, Eq. 1–2): e^{{A(t−u)}} M x is not CINT"
    );
    assert!(
        (carry - time_independent).abs() > 1e-3,
        "Driver et al. (2017, Eq. 1–2): e^{{A(t−u)}} M x is not TIPREDEFFECT"
    );
    assert!(
        (carry - equation_fourteen).abs() > 1e-3,
        "Driver et al. (2017, Eq. 1–2): e^{{A(t−u)}} M x is not Voelkle Eq. 14"
    );
    assert!(
        (composed - evolved).abs() > 1e-3,
        "Driver et al. (2017, Eq. 1–2): μ_t is not μ_t + e^{{A(t−u)}} M x"
    );
    assert_eq!(
        refuse_time_dependent_impulse_carry_as_contemporaneous_impulse(carry, impulse),
        Err(psychometric_core::PsychometricError::TimeDependentImpulseCarryIsNotContemporaneousImpulse)
    );
    assert_eq!(
        refuse_time_dependent_impulse_carry_as_continuous_intercept(carry, effect),
        Err(
            psychometric_core::PsychometricError::TimeDependentImpulseCarryIsNotContinuousIntercept
        )
    );
    assert_eq!(
        refuse_time_dependent_impulse_carry_as_time_independent_effect(carry, time_independent),
        Err(psychometric_core::PsychometricError::TimeDependentImpulseCarryIsNotTimeIndependentEffect)
    );
    assert_eq!(
        refuse_time_dependent_impulse_carry_as_time_varying_discrete_effect(
            carry,
            equation_fourteen
        ),
        Err(psychometric_core::PsychometricError::TimeDependentImpulseCarryIsNotTimeVaryingDiscreteEffect)
    );
}

#[test]
fn evolved_observed_mean_is_not_impulse_carry_observed_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let elapsed = 1.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let impulse_carry_observed = recover_discrete_observed_mean_with_impulse_carry(
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
    let contemporaneous = recover_discrete_observed_mean_with_impulse(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-mx");
    assert!(
        (evolved_observed - impulse_carry_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Eq. 1–2): τ + λ μ_t is not impulse-carry E(y_t)"
    );
    assert!(
        (manifest_mean - impulse_carry_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 / Table 2 p. 12): MANIFESTMEANS is not impulse-carry E(y_t)"
    );
    assert!(
        (carried - impulse_carry_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): carried latent mean is not E(y_t)"
    );
    assert!(
        (contemporaneous - impulse_carry_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Eq. 1–2): τ + λ(μ_t + m x) is not impulse-carry E(y_t)"
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_impulse_carry_observed_mean(
            evolved_observed,
            impulse_carry_observed
        ),
        Err(psychometric_core::PsychometricError::EvolvedObservedMeanIsNotImpulseCarryObservedMean)
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_impulse_carry_observed_mean(
            contemporaneous,
            impulse_carry_observed
        ),
        Err(psychometric_core::PsychometricError::ImpulseObservedMeanIsNotImpulseCarryObservedMean)
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(carried, impulse_carry_observed),
        Err(psychometric_core::PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, impulse_carry_observed),
        Err(psychometric_core::PsychometricError::ManifestMeansIsNotObservedMean)
    );
}

#[test]
fn evolved_observed_mean_is_not_time_independent_observed_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let time_independent_observed = recover_discrete_observed_mean_with_time_independent_predictor(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-tipred-mean");
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
    let composed = recover_discrete_latent_mean_with_time_independent_predictor(
        initial,
        drift,
        intercept,
        effect,
        predictor,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-tipred");
    assert!(
        (evolved_observed - time_independent_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Eq. 3 TIPREDEFFECT): τ + λ μ_t is not TIPREDEFFECT E(y_t)"
    );
    assert!(
        (manifest_mean - time_independent_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 / Table 2 p. 12): MANIFESTMEANS is not TIPREDEFFECT E(y_t)"
    );
    assert!(
        (composed - time_independent_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): evolved-plus-increment latent mean is not E(y_t)"
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_time_independent_observed_mean(
            evolved_observed,
            time_independent_observed
        ),
        Err(
            psychometric_core::PsychometricError::EvolvedObservedMeanIsNotTimeIndependentObservedMean
        )
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(composed, time_independent_observed),
        Err(psychometric_core::PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, time_independent_observed),
        Err(psychometric_core::PsychometricError::ManifestMeansIsNotObservedMean)
    );
}

#[test]
fn impulse_and_carry_observed_mean_are_not_time_independent_observed_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let time_independent_observed = recover_discrete_observed_mean_with_time_independent_predictor(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-tipred-mean");
    let impulse_observed = recover_discrete_observed_mean_with_impulse(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-impulse-mean");
    let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
        loading,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        1.0,
        LagClock::EventTime,
    )
    .expect("eq5-carry-mean");
    assert!(
        (impulse_observed - time_independent_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Eq. 3 impulse): τ + λ(μ_t + m x) is not TIPREDEFFECT E(y_t)"
    );
    assert!(
        (carried_observed - time_independent_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of Eq. 1–2): τ + λ(μ_t + carry) is not TIPREDEFFECT E(y_t)"
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_time_independent_observed_mean(
            impulse_observed,
            time_independent_observed
        ),
        Err(
            psychometric_core::PsychometricError::ImpulseObservedMeanIsNotTimeIndependentObservedMean
        )
    );
    assert_eq!(
        refuse_impulse_carry_observed_mean_as_time_independent_observed_mean(
            carried_observed,
            time_independent_observed
        ),
        Err(
            psychometric_core::PsychometricError::ImpulseCarryObservedMeanIsNotTimeIndependentObservedMean
        )
    );
}

#[test]
fn level_change_cint_is_not_impulse_free_cint_or_process_increment() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let intercept =
        recover_level_change_continuous_intercept(effect, predictor, drift).expect("level-change");
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("impulse");
    let increment = recover_discrete_time_independent_predictor_effect(
        effect,
        predictor,
        drift,
        2.0,
        LagClock::EventTime,
    )
    .expect("tipred");
    assert!(
        (intercept - impulse).abs() > 1e-3,
        "Driver et al. (2017, §7.2): −a m x is not the dissipating Dirac m x"
    );
    assert!(
        (intercept - 0.3).abs() > 1e-3,
        "Driver et al. (2017, §7.2): −a m x is not a free CINT"
    );
    assert!(
        (intercept - increment).abs() > 1e-3,
        "Driver et al. (2017, §7.2): −a m x is not TIPREDEFFECT increment"
    );
    assert_eq!(
        refuse_level_change_intercept_as_impulse(intercept, impulse),
        Err(psychometric_core::PsychometricError::LevelChangeInterceptIsNotImpulse)
    );
    assert_eq!(
        refuse_level_change_intercept_as_free_continuous_intercept(intercept, 0.3),
        Err(psychometric_core::PsychometricError::LevelChangeInterceptIsNotFreeContinuousIntercept)
    );
    assert_eq!(
        refuse_level_change_intercept_as_process_increment(intercept, increment),
        Err(psychometric_core::PsychometricError::LevelChangeInterceptIsNotProcessIncrement)
    );
}

#[test]
fn level_change_increment_is_not_impulse_intercept_or_process_increment() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let intercept =
        recover_level_change_continuous_intercept(effect, predictor, drift).expect("level-change");
    let increment = recover_level_change_discrete_increment(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("level-change-increment");
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("impulse");
    let tipred = recover_discrete_time_independent_predictor_effect(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("tipred");
    assert!(
        (increment - impulse).abs() > 1e-3,
        "Driver et al. (2017, §7.2 / Eq. 3): (1 − e^{{aΔt}}) m x is not the dissipating Dirac m x"
    );
    assert!(
        (increment - intercept).abs() > 1e-3,
        "Driver et al. (2017, §7.2 / Eq. 3): (1 − e^{{aΔt}}) m x is not κ"
    );
    assert!(
        (increment - tipred).abs() > 1e-3,
        "Driver et al. (2017, §7.2 / Eq. 3): (1 − e^{{aΔt}}) m x is not TIPREDEFFECT increment"
    );
    assert_eq!(
        refuse_level_change_increment_as_impulse(increment, impulse),
        Err(psychometric_core::PsychometricError::LevelChangeIncrementIsNotImpulse)
    );
    assert_eq!(
        refuse_level_change_increment_as_intercept(increment, intercept),
        Err(psychometric_core::PsychometricError::LevelChangeIncrementIsNotIntercept)
    );
    assert_eq!(
        refuse_level_change_increment_as_process_increment(increment, tipred),
        Err(psychometric_core::PsychometricError::LevelChangeIncrementIsNotProcessIncrement)
    );
}

#[test]
fn extra_process_contribution_is_not_cint_rewrite_increment_or_impulse() {
    let coupling = 0.4_f64;
    let predictor = 3.0_f64;
    let original = -0.5_f64;
    let extra = -0.05_f64;
    let delta = 2.0_f64;
    let recovered = recover_level_change_extra_process_contribution(
        coupling,
        predictor,
        original,
        extra,
        delta,
        LagClock::EventTime,
    )
    .expect("extra-process");
    let intercept =
        recover_level_change_continuous_intercept(coupling, predictor, original).expect("cint");
    let increment = recover_level_change_discrete_increment(
        coupling,
        predictor,
        original,
        delta,
        LagClock::EventTime,
    )
    .expect("increment");
    let impulse = recover_time_dependent_predictor_impulse(coupling, predictor).expect("impulse");
    assert!(
        (recovered - intercept).abs() > 1e-3,
        "Driver et al. (2017, §7.2 pp. 22–23): extra-process contribution is not κ = −a m x"
    );
    assert!(
        (recovered - increment).abs() > 1e-3,
        "Driver et al. (2017, §7.2 pp. 22–23): extra-process contribution is not (1 − e^{{aΔt}}) m x"
    );
    assert!(
        (recovered - impulse).abs() > 1e-3,
        "Driver et al. (2017, §7.2 pp. 22–23): extra-process contribution is not the dissipating Dirac m x"
    );
    assert_eq!(
        refuse_level_change_extra_process_as_impulse(recovered, impulse),
        Err(psychometric_core::PsychometricError::LevelChangeExtraProcessIsNotImpulse)
    );
    assert_eq!(
        refuse_level_change_extra_process_as_intercept(recovered, intercept),
        Err(psychometric_core::PsychometricError::LevelChangeExtraProcessIsNotIntercept)
    );
    assert_eq!(
        refuse_level_change_extra_process_as_increment(recovered, increment),
        Err(psychometric_core::PsychometricError::LevelChangeExtraProcessIsNotIncrement)
    );
}

#[test]
fn extra_process_observed_mean_is_not_evolved_mean_impulse_mean_or_contribution() {
    let loading = 2.0_f64;
    let coupling = 0.4_f64;
    let predictor = 3.0_f64;
    let original = -0.5_f64;
    let extra = -0.05_f64;
    let delta = 2.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_extra_process(
        loading,
        initial,
        original,
        intercept,
        coupling,
        predictor,
        extra,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-extra-process-mean");
    let composed = recover_discrete_latent_mean_with_extra_process(
        initial,
        original,
        intercept,
        coupling,
        predictor,
        extra,
        delta,
        LagClock::EventTime,
    )
    .expect("extra-latent");
    let contribution = recover_level_change_extra_process_contribution(
        coupling,
        predictor,
        original,
        extra,
        delta,
        LagClock::EventTime,
    )
    .expect("extra-process");
    let evolved_observed = recover_discrete_observed_mean(
        loading,
        initial,
        original,
        intercept,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-eq5-mean");
    let impulse_observed = recover_discrete_observed_mean_with_impulse(
        loading,
        initial,
        original,
        intercept,
        coupling,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-impulse-mean");
    assert!(
        (observed - evolved_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §7.2): extra-process E(y_t) is not τ + λ μ_t"
    );
    assert!(
        (observed - impulse_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §7.2): extra-process E(y_t) is not τ + λ(μ_t + m x)"
    );
    assert!(
        (observed - contribution).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §7.2): extra-process contribution is not E(y_t)"
    );
    assert!(
        (observed - composed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §7.2): evolved-plus-contribution latent mean is not E(y_t)"
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_extra_process_observed_mean(evolved_observed, observed),
        Err(psychometric_core::PsychometricError::EvolvedObservedMeanIsNotExtraProcessObservedMean)
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_extra_process_observed_mean(impulse_observed, observed),
        Err(psychometric_core::PsychometricError::ImpulseObservedMeanIsNotExtraProcessObservedMean)
    );
    assert_eq!(
        refuse_extra_process_contribution_as_observed_mean(contribution, observed),
        Err(psychometric_core::PsychometricError::ExtraProcessContributionIsNotObservedMean)
    );
    assert_eq!(
        refuse_extra_process_latent_mean_as_observed_mean(composed, observed),
        Err(psychometric_core::PsychometricError::ExtraProcessLatentMeanIsNotObservedMean)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn after_extra_process_observed_mean_is_not_t0_extra_evolved_or_impulse_carry() {
    let loading = 2.0_f64;
    let coupling = 0.4_f64;
    let predictor = 3.0_f64;
    let original = -0.5_f64;
    let extra = -0.05_f64;
    let delta = 2.0_f64;
    let elapsed = 1.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_extra_process_after(
        loading,
        initial,
        original,
        intercept,
        coupling,
        predictor,
        extra,
        manifest_mean,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("eq5-after-extra-process-mean");
    let composed = recover_discrete_latent_mean_with_extra_process_after(
        initial,
        original,
        intercept,
        coupling,
        predictor,
        extra,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("after-extra-latent");
    let contribution = recover_level_change_extra_process_contribution_after(
        coupling,
        predictor,
        original,
        extra,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("after-extra-process");
    let first_occasion = recover_discrete_observed_mean_with_extra_process(
        loading,
        initial,
        original,
        intercept,
        coupling,
        predictor,
        extra,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq5-t0-extra-process-mean");
    let evolved_observed = recover_discrete_observed_mean(
        loading,
        initial,
        original,
        intercept,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-eq5-mean");
    let carry_observed = recover_discrete_observed_mean_with_impulse_carry(
        loading,
        initial,
        original,
        intercept,
        coupling,
        predictor,
        manifest_mean,
        delta,
        elapsed,
        LagClock::EventTime,
    )
    .expect("eq5-impulse-carry-mean");
    assert!(
        (observed - first_occasion).abs() > 1e-3,
        "Driver et al. (2017, §7.2): T0TDPREDEFFECT extra E(y_t) is not after-t0 E(y_t)"
    );
    assert!(
        (observed - evolved_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §7.2 after t0): after-t0 E(y_t) is not τ + λ μ_t"
    );
    assert!(
        (observed - carry_observed).abs() > 1e-3,
        "Driver et al. (2017, §7.2): e^{{a(t-u)}} m x is not extra-process DRIFT drive"
    );
    assert!((observed - contribution).abs() > 1e-3);
    assert!((observed - composed).abs() > 1e-3);
    assert_eq!(
        refuse_extra_process_observed_mean_as_after_extra_process_observed_mean(
            first_occasion,
            observed
        ),
        Err(
            psychometric_core::PsychometricError::ExtraProcessObservedMeanIsNotAfterExtraProcessObservedMean
        )
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_after_extra_process_observed_mean(
            evolved_observed,
            observed
        ),
        Err(
            psychometric_core::PsychometricError::EvolvedObservedMeanIsNotAfterExtraProcessObservedMean
        )
    );
    assert_eq!(
        refuse_impulse_carry_observed_mean_as_after_extra_process_observed_mean(
            carry_observed,
            observed
        ),
        Err(
            psychometric_core::PsychometricError::ImpulseCarryObservedMeanIsNotAfterExtraProcessObservedMean
        )
    );
    assert_eq!(
        refuse_after_extra_process_contribution_as_observed_mean(contribution, observed),
        Err(psychometric_core::PsychometricError::AfterExtraProcessContributionIsNotObservedMean)
    );
    assert_eq!(
        refuse_after_extra_process_latent_mean_as_observed_mean(composed, observed),
        Err(psychometric_core::PsychometricError::AfterExtraProcessLatentMeanIsNotObservedMean)
    );
}

#[test]
fn asymptotic_time_independent_effect_is_not_coefficient_discrete_cint_or_impulse() {
    let effect = -0.225_f64;
    let predictor = 2.0_f64;
    let log_rate = -0.134_488_942_f64;
    let recovered = recover_asymptotic_time_independent_predictor_effect(
        effect,
        predictor,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECT");
    let discrete = recover_discrete_time_independent_predictor_effect(
        effect,
        predictor,
        log_rate,
        1.0,
        LagClock::EventTime,
    )
    .expect("discreteTIPREDEFFECT");
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("impulse");
    assert!(
        (recovered - effect).abs() > 1e-3,
        "Driver et al. (2017, §7.2, pp. 20–21): asymTIPREDEFFECT is not TIPREDEFFECT B"
    );
    assert!(
        (recovered - discrete).abs() > 1e-3,
        "Driver et al. (2017, §7.2): -B z / a is not A^{{-1}}[e^{{A Δt}} − I] B z"
    );
    assert!(
        (recovered - impulse).abs() > 1e-3,
        "Driver et al. (2017, §7.2): -B z / a is not M x"
    );
    assert_eq!(
        refuse_asymptotic_time_independent_effect_as_coefficient(recovered, effect),
        Err(psychometric_core::PsychometricError::AsymptoticTimeIndependentEffectIsNotCoefficient)
    );
    assert_eq!(
        refuse_asymptotic_time_independent_effect_as_discrete_effect(recovered, discrete),
        Err(
            psychometric_core::PsychometricError::AsymptoticTimeIndependentEffectIsNotDiscreteEffect
        )
    );
    assert_eq!(
        refuse_asymptotic_time_independent_effect_as_continuous_intercept(recovered, 0.3),
        Err(
            psychometric_core::PsychometricError::AsymptoticTimeIndependentEffectIsNotContinuousIntercept
        )
    );
    assert_eq!(
        refuse_asymptotic_time_independent_effect_as_time_dependent_impulse(recovered, impulse),
        Err(
            psychometric_core::PsychometricError::AsymptoticTimeIndependentEffectIsNotTimeDependentImpulse
        )
    );
}

#[test]
fn asymptotic_time_independent_variance_is_not_trait_stationary_or_mean_effect() {
    let effect = -0.225_f64;
    let log_rate = -0.134_488_942_f64;
    let recovered = recover_asymptotic_time_independent_predictor_variance(
        effect,
        2.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let mean_effect = recover_asymptotic_time_independent_predictor_effect(
        effect,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECT");
    let stationary = recover_stationary_latent_variance(0.4, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let trait_plus = recover_trait_plus_state_latent_variance(0.8, 0.3).expect("trait");
    assert!(
        (recovered - mean_effect).abs() > 1e-3,
        "Driver et al. (2017, §7.2, pp. 20–21): addedTIPREDVAR is not asymTIPREDEFFECT"
    );
    assert!(
        (recovered - stationary).abs() > 1e-3,
        "Driver et al. (2017, §7.2): addedTIPREDVAR is not asymDIFFUSION"
    );
    assert!(
        (recovered - trait_plus).abs() > 1e-3,
        "Driver et al. (2017, §7.2): addedTIPREDVAR is not TRAITVAR"
    );
    assert_eq!(
        refuse_asymptotic_time_independent_variance_as_trait_variance(recovered, trait_plus),
        Err(
            psychometric_core::PsychometricError::AsymptoticTimeIndependentVarianceIsNotTraitVariance
        )
    );
    assert_eq!(
        refuse_asymptotic_time_independent_variance_as_stationary_within_subject(
            recovered,
            stationary
        ),
        Err(
            psychometric_core::PsychometricError::AsymptoticTimeIndependentVarianceIsNotStationaryWithinSubject
        )
    );
    assert_eq!(
        refuse_asymptotic_time_independent_variance_as_asymptotic_effect(recovered, mean_effect),
        Err(
            psychometric_core::PsychometricError::AsymptoticTimeIndependentVarianceIsNotAsymptoticEffect
        )
    );
}

#[test]
fn asymptotic_continuous_intercept_is_not_cint_increment_t0_or_tipred() {
    let intercept = 0.3_f64;
    let log_rate = -0.134_488_942_f64;
    let recovered =
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
            .expect("asymCINT");
    let discrete =
        recover_discrete_continuous_intercept_effect(intercept, log_rate, 1.0, LagClock::EventTime)
            .expect("dtCINT");
    let tipred = recover_asymptotic_time_independent_predictor_effect(
        -0.225,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECT");
    assert!(
        (recovered - intercept).abs() > 1e-3,
        "Driver et al. (2017, Table 2, p. 12): asymCINT is not CINT"
    );
    assert!(
        (recovered - discrete).abs() > 1e-3,
        "Driver et al. (2017, Table 2): -κ / a is not A^{{-1}}[e^{{A Δt}} − I] κ"
    );
    assert!(
        (recovered - 2.823).abs() > 1e-3,
        "Driver et al. (2017, Table 2): -κ / a is not T0MEANS"
    );
    assert!(
        (recovered - tipred).abs() > 1e-3,
        "Driver et al. (2017, Table 2): -κ / a is not -B z / a"
    );
    assert_eq!(
        refuse_asymptotic_continuous_intercept_as_continuous_intercept(recovered, intercept),
        Err(
            psychometric_core::PsychometricError::AsymptoticContinuousInterceptIsNotContinuousIntercept
        )
    );
    assert_eq!(
        refuse_asymptotic_continuous_intercept_as_discrete_increment(recovered, discrete),
        Err(
            psychometric_core::PsychometricError::AsymptoticContinuousInterceptIsNotDiscreteIncrement
        )
    );
    assert_eq!(
        refuse_asymptotic_continuous_intercept_as_initial_latent_mean(recovered, 2.823),
        Err(
            psychometric_core::PsychometricError::AsymptoticContinuousInterceptIsNotInitialLatentMean
        )
    );
    assert_eq!(
        refuse_asymptotic_continuous_intercept_as_asymptotic_time_independent_effect(
            recovered, tipred
        ),
        Err(
            psychometric_core::PsychometricError::AsymptoticContinuousInterceptIsNotAsymptoticTimeIndependentEffect
        )
    );
}

#[test]
fn stationary_initial_latent_mean_is_not_t0_cint_tipred_or_discrete() {
    let intercept = 0.3_f64;
    let log_rate = -0.134_488_942_f64;
    let recovered = recover_stationary_initial_latent_mean(
        intercept,
        -0.225,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0MEANS");
    let intercept_only =
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
            .expect("asymCINT");
    let tipred = recover_asymptotic_time_independent_predictor_effect(
        -0.225,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECT");
    let discrete =
        recover_discrete_latent_mean(2.823, log_rate, intercept, 1.0, LagClock::EventTime)
            .expect("μ_t");
    assert!(
        (recovered - 2.823).abs() > 1e-3,
        "Driver et al. (2017, p. 16): constrained T0MEANS is not free T0MEANS"
    );
    assert!(
        (recovered - intercept_only).abs() > 1e-3,
        "Driver et al. (2017, p. 16): constrained T0MEANS is not asymCINT"
    );
    assert!(
        (recovered - tipred).abs() > 1e-3,
        "Driver et al. (2017, p. 16): constrained T0MEANS is not asymTIPREDEFFECT"
    );
    assert!(
        (recovered - discrete).abs() > 1e-3,
        "Driver et al. (2017, p. 16): constrained T0MEANS is not μ_t"
    );
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_initial_latent_mean(recovered, 2.823),
        Err(
            psychometric_core::PsychometricError::StationaryInitialLatentMeanIsNotInitialLatentMean
        )
    );
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_asymptotic_continuous_intercept(
            recovered,
            intercept_only
        ),
        Err(
            psychometric_core::PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticContinuousIntercept
        )
    );
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_asymptotic_time_independent_effect(
            recovered, tipred
        ),
        Err(
            psychometric_core::PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticTimeIndependentEffect
        )
    );
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_discrete_mean(recovered, discrete),
        Err(psychometric_core::PsychometricError::StationaryInitialLatentMeanIsNotDiscreteMean)
    );
}

#[test]
fn stationary_initial_observed_mean_is_not_manifest_latent_evolved_or_free() {
    let intercept = 0.3_f64;
    let log_rate = -0.134_488_942_f64;
    let loading = 2.0_f64;
    let manifest_mean = 0.5_f64;
    let recovered = recover_stationary_initial_observed_mean(
        loading,
        intercept,
        -0.225,
        1.0,
        log_rate,
        manifest_mean,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-T0MEANS");
    let latent = recover_stationary_initial_latent_mean(
        intercept,
        -0.225,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0MEANS");
    let intercept_only =
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
            .expect("asymCINT");
    let intercept_only_observed =
        recover_manifest_observed_mean(loading, intercept_only, manifest_mean).expect("τ+λ(−κ/a)");
    let free_initial_observed =
        recover_manifest_observed_mean(loading, 2.823, manifest_mean).expect("τ+λμ_0");
    let evolved = recover_discrete_observed_mean(
        loading,
        2.823,
        log_rate,
        intercept,
        manifest_mean,
        1.0,
        LagClock::EventTime,
    )
    .expect("τ+λμ_t");
    assert!(
        (recovered - manifest_mean).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / Eq. 5): E(y_0) is not MANIFESTMEANS"
    );
    assert!(
        (recovered - latent).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / Eq. 5): E(y_0) is not the constrained latent mean"
    );
    assert!(
        (recovered - intercept_only_observed).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / Eq. 5): E(y_0) is not τ + λ(−κ / a)"
    );
    assert!(
        (recovered - free_initial_observed).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / Eq. 5): E(y_0) is not τ + λ μ_0"
    );
    assert!(
        (recovered - evolved).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / Eq. 5): E(y_0) is not τ + λ μ_t"
    );
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_observed_mean(latent, recovered),
        Err(psychometric_core::PsychometricError::StationaryInitialLatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_stationary_initial_observed_mean_as_manifest_means(recovered, manifest_mean),
        Err(psychometric_core::PsychometricError::StationaryInitialObservedMeanIsNotManifestMeans)
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_stationary_initial_observed_mean(evolved, recovered),
        Err(
            psychometric_core::PsychometricError::EvolvedObservedMeanIsNotStationaryInitialObservedMean
        )
    );
    assert_eq!(
        refuse_asymptotic_continuous_intercept_observed_mean_as_stationary_initial_observed_mean(
            intercept_only_observed,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::AsymptoticContinuousInterceptObservedMeanIsNotStationaryInitialObservedMean
        )
    );
    assert_eq!(
        refuse_initial_observed_mean_as_stationary_initial_observed_mean(
            free_initial_observed,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::InitialObservedMeanIsNotStationaryInitialObservedMean
        )
    );
}

#[test]
fn stationary_initial_latent_variance_is_not_t0_state_trait_tipred_or_discrete() {
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let log_rate = -0.134_488_942_f64;
    let recovered = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0VAR");
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let added = recover_asymptotic_time_independent_predictor_variance(
        -0.225,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let discrete =
        recover_discrete_latent_variance(recovered, diffusion, log_rate, 1.0, LagClock::EventTime)
            .expect("Var(η_t)");
    assert!(
        (recovered - 2.0).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / p. 16): constrained T0VAR is not free T0VAR"
    );
    assert!(
        (recovered - state).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / p. 16): constrained T0VAR is not asymDIFFUSION"
    );
    assert!(
        (recovered - trait_variance).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / p. 16): constrained T0VAR is not TRAITVAR"
    );
    assert!(
        (recovered - added).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / p. 16): constrained T0VAR is not addedTIPREDVAR"
    );
    assert!(
        (recovered - discrete).abs() > 1e-3,
        "Driver et al. (2017, §4.3 / p. 16): constrained T0VAR is not Var(η_t)"
    );
    assert!(
        (recovered - 2.838).abs() > 1e-3,
        "Driver et al. (2017, §7.2): printed 2-latent addedTIPREDVAR is not this scalar map"
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_initial_latent_variance(recovered, 2.0),
        Err(
            psychometric_core::PsychometricError::StationaryInitialLatentVarianceIsNotInitialLatentVariance
        )
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_stationary_within_subject(recovered, state),
        Err(
            psychometric_core::PsychometricError::StationaryInitialLatentVarianceIsNotStationaryWithinSubject
        )
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_trait_variance(recovered, trait_variance),
        Err(
            psychometric_core::PsychometricError::StationaryInitialLatentVarianceIsNotTraitVariance
        )
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_asymptotic_time_independent_variance(
            recovered, added
        ),
        Err(
            psychometric_core::PsychometricError::StationaryInitialLatentVarianceIsNotAsymptoticTimeIndependentVariance
        )
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_discrete_variance(recovered, discrete),
        Err(
            psychometric_core::PsychometricError::StationaryInitialLatentVarianceIsNotDiscreteVariance
        )
    );
}

#[test]
fn stationary_initial_observed_variance_is_not_manifest_latent_evolved_or_free() {
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let log_rate = -0.134_488_942_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let recovered = recover_stationary_initial_observed_variance(
        loading,
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        measurement_error,
        0.1,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-T0VAR");
    let latent = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0VAR");
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let state_only_observed =
        recover_manifest_observed_variance(loading, state, measurement_error).expect("λ²(−q/2a)+θ");
    let free_initial_observed =
        recover_manifest_observed_variance(loading, 2.0, measurement_error).expect("λ²p_0+θ");
    let discrete =
        recover_discrete_latent_variance(latent, diffusion, log_rate, 1.0, LagClock::EventTime)
            .expect("Var(η_t)");
    let evolved = recover_manifest_observed_variance(loading, discrete, measurement_error)
        .expect("λ²Var(η_t)+θ");
    assert!(
        (recovered - measurement_error).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §4.3 T0VAR): Var(y_0) is not MANIFESTVAR"
    );
    assert!(
        (recovered - latent).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §4.3 T0VAR): Var(y_0) is not constrained T0VAR"
    );
    assert!(
        (recovered - state_only_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §4.3 T0VAR): Var(y_0) is not λ² asymDIFFUSION + θ"
    );
    assert!(
        (recovered - free_initial_observed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §4.3 T0VAR): Var(y_0) is not λ² free T0VAR + θ"
    );
    assert!(
        (recovered - evolved).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of §4.3 T0VAR): Var(y_0) is not λ² Var(η_t) + θ"
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_observed_variance(latent, recovered),
        Err(
            psychometric_core::PsychometricError::StationaryInitialLatentVarianceIsNotObservedVariance
        )
    );
    assert_eq!(
        refuse_stationary_initial_observed_variance_as_measurement_error(
            recovered,
            measurement_error
        ),
        Err(
            psychometric_core::PsychometricError::StationaryInitialObservedVarianceIsNotMeasurementError
        )
    );
    assert_eq!(
        refuse_evolved_observed_variance_as_stationary_initial_observed_variance(
            evolved, recovered
        ),
        Err(
            psychometric_core::PsychometricError::EvolvedObservedVarianceIsNotStationaryInitialObservedVariance
        )
    );
    assert_eq!(
        refuse_stationary_within_subject_observed_variance_as_stationary_initial_observed_variance(
            state_only_observed,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::StationaryWithinSubjectObservedVarianceIsNotStationaryInitialObservedVariance
        )
    );
    assert_eq!(
        refuse_initial_observed_variance_as_stationary_initial_observed_variance(
            free_initial_observed,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::InitialObservedVarianceIsNotStationaryInitialObservedVariance
        )
    );
}

#[test]
fn stationary_lagged_latent_covariance_is_not_contemporaneous_decayed_or_trait_state() {
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let log_rate = -0.134_488_942_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_stationary_lagged_latent_covariance(
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary lagged T0VAR");
    let contemporaneous = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0VAR");
    let decayed = recover_discrete_lagged_latent_covariance(
        contemporaneous,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("e^{aΔt} p_stat");
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let trait_plus_state = recover_trait_plus_state_lagged_covariance(
        trait_variance,
        state,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("trait+state lagged");
    assert!(
        (recovered - contemporaneous).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3–4 of §4.3 T0VAR): lagged covariance is not contemporaneous T0VAR"
    );
    assert!(
        (recovered - decayed).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3–4 of §4.3 T0VAR): trait and addedTIPREDVAR do not decay"
    );
    assert!(
        (recovered - trait_plus_state).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3–4 of §4.3 T0VAR): lagged T0VAR is not trait-plus-state lagged"
    );
    assert_eq!(
        refuse_stationary_lagged_latent_covariance_as_stationary_initial_latent_variance(
            recovered,
            contemporaneous
        ),
        Err(
            psychometric_core::PsychometricError::StationaryLaggedLatentCovarianceIsNotStationaryInitialLatentVariance
        )
    );
    assert_eq!(
        refuse_stationary_lagged_latent_covariance_as_decayed_stationary_variance(
            recovered, decayed
        ),
        Err(
            psychometric_core::PsychometricError::StationaryLaggedLatentCovarianceIsNotDecayedStationaryVariance
        )
    );
    assert_eq!(
        refuse_trait_plus_state_lagged_covariance_as_stationary_lagged_latent_covariance(
            trait_plus_state,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::TraitPlusStateLaggedCovarianceIsNotStationaryLaggedLatentCovariance
        )
    );
}

#[test]
fn stationary_lagged_observed_covariance_is_not_manifest_latent_or_contemporaneous() {
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let log_rate = -0.134_488_942_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_stationary_lagged_observed_covariance(
        loading,
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        event_delta,
        0.1,
        LagClock::EventTime,
    )
    .expect("eq5-lagged-stationary-T0VAR");
    let latent = recover_stationary_lagged_latent_covariance(
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary lagged T0VAR");
    let contemporaneous = recover_stationary_initial_observed_variance(
        loading,
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        measurement_error,
        0.1,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-T0VAR");
    assert!(
        (recovered - measurement_error).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of lagged §4.3 T0VAR): lagged cov(y) is not MANIFESTVAR"
    );
    assert!(
        (recovered - latent).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of lagged §4.3 T0VAR): lagged cov(y) is not lagged T0VAR"
    );
    assert!(
        (recovered - contemporaneous).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of lagged §4.3 T0VAR): lagged cov(y) is not Var(y_0)"
    );
    assert_eq!(
        refuse_stationary_lagged_latent_covariance_as_observed_covariance(latent, recovered),
        Err(
            psychometric_core::PsychometricError::StationaryLaggedLatentCovarianceIsNotObservedCovariance
        )
    );
    assert_eq!(
        refuse_measurement_error_as_stationary_lagged_observed_covariance(
            measurement_error,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::MeasurementErrorIsNotStationaryLaggedObservedCovariance
        )
    );
    assert_eq!(
        refuse_stationary_initial_observed_variance_as_stationary_lagged_observed_covariance(
            contemporaneous,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::StationaryInitialObservedVarianceIsNotStationaryLaggedObservedCovariance
        )
    );
}

#[test]
fn stationary_later_latent_variance_is_not_lagged_discrete_or_process_noise() {
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let log_rate = -0.134_488_942_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_stationary_later_latent_variance(
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary later T0VAR");
    let lagged = recover_stationary_lagged_latent_covariance(
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary lagged T0VAR");
    let contemporaneous = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0VAR");
    let free_discrete = recover_discrete_latent_variance(
        contemporaneous,
        diffusion,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("e^{2aΔt} p_stat + Q_Δt");
    let process_noise =
        recover_discrete_process_noise(diffusion, log_rate, event_delta, LagClock::EventTime)
            .expect("Q_Δt");
    assert!(
        (recovered - contemporaneous).abs() < 1e-12,
        "Driver et al. (2017, Eq. 3–4 of §4.3 T0VAR): later-occasion variance equals contemporaneous T0VAR under stationarity"
    );
    assert!(
        (recovered - lagged).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3–4 of §4.3 T0VAR): later-occasion variance is not lagged covariance"
    );
    assert!(
        (recovered - free_discrete).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3–4 of §4.3 T0VAR): trait and addedTIPREDVAR do not enter Q_Δt"
    );
    assert!(
        (recovered - process_noise).abs() > 1e-3,
        "Driver et al. (2017, Eq. 3–4 of §4.3 T0VAR): later-occasion variance is not Q_Δt"
    );
    assert_eq!(
        refuse_stationary_later_latent_variance_as_lagged_covariance(recovered, lagged),
        Err(
            psychometric_core::PsychometricError::StationaryLaterLatentVarianceIsNotLaggedCovariance
        )
    );
    assert_eq!(
        refuse_stationary_later_latent_variance_as_discrete_variance(recovered, free_discrete),
        Err(
            psychometric_core::PsychometricError::StationaryLaterLatentVarianceIsNotDiscreteVariance
        )
    );
    assert_eq!(
        refuse_stationary_later_latent_variance_as_process_noise(recovered, process_noise),
        Err(psychometric_core::PsychometricError::StationaryLaterLatentVarianceIsNotProcessNoise)
    );
}

#[test]
fn stationary_later_observed_variance_is_not_manifest_latent_or_lagged() {
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let log_rate = -0.134_488_942_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_stationary_later_observed_variance(
        loading,
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        event_delta,
        measurement_error,
        0.1,
        LagClock::EventTime,
    )
    .expect("eq5-later-stationary-T0VAR");
    let latent = recover_stationary_later_latent_variance(
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary later T0VAR");
    let lagged = recover_stationary_lagged_observed_covariance(
        loading,
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        event_delta,
        0.1,
        LagClock::EventTime,
    )
    .expect("eq5-lagged-stationary-T0VAR");
    let contemporaneous = recover_stationary_initial_observed_variance(
        loading,
        trait_variance,
        diffusion,
        -0.225,
        1.0,
        log_rate,
        measurement_error,
        0.1,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-T0VAR");
    assert!(
        (recovered - contemporaneous).abs() < 1e-12,
        "Driver et al. (2017, Eq. 5 of later §4.3 T0VAR): Var(y_t) equals Var(y_0) under stationarity"
    );
    assert!(
        (recovered - measurement_error).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of later §4.3 T0VAR): Var(y_t) is not MANIFESTVAR"
    );
    assert!(
        (recovered - latent).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of later §4.3 T0VAR): Var(y_t) is not later T0VAR"
    );
    assert!(
        (recovered - lagged).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5 of later §4.3 T0VAR): Var(y_t) is not lagged cov(y)"
    );
    assert_eq!(
        refuse_stationary_later_latent_variance_as_observed_variance(latent, recovered),
        Err(
            psychometric_core::PsychometricError::StationaryLaterLatentVarianceIsNotObservedVariance
        )
    );
    assert_eq!(
        refuse_measurement_error_as_stationary_later_observed_variance(
            measurement_error,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::MeasurementErrorIsNotStationaryLaterObservedVariance
        )
    );
    assert_eq!(
        refuse_stationary_lagged_observed_covariance_as_stationary_later_observed_variance(
            lagged, recovered
        ),
        Err(
            psychometric_core::PsychometricError::StationaryLaggedObservedCovarianceIsNotStationaryLaterObservedVariance
        )
    );
}

#[test]
fn standardised_continuous_intercept_is_not_unstandardised_asymptotic_or_discrete() {
    let intercept = 0.4_f64;
    let diffusion = 0.8_f64;
    let log_rate = -0.5_f64;
    let recovered = recover_standardised_continuous_intercept(
        intercept,
        diffusion,
        log_rate,
        LagClock::EventTime,
    )
    .expect("CINTstd");
    let stationary =
        recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime).expect("p");
    assert!(
        (recovered - intercept / stationary.sqrt()).abs() < 1e-15,
        "Driver et al. (2017, p. 16 footnote 4): CINTstd is κ / √p"
    );
    assert!(
        (recovered - intercept).abs() > 1e-3,
        "Driver et al. (2017, Table 2): unstandardised CINT is not CINTstd"
    );
    let asymptotic =
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
            .expect("asymCINT")
            / stationary.sqrt();
    assert!(
        (asymptotic - recovered).abs() > 1e-3,
        "Driver et al. (2017, p. 16): asymCINTstd is not CINTstd"
    );
    let discrete =
        recover_discrete_continuous_intercept_effect(intercept, log_rate, 1.0, LagClock::EventTime)
            .expect("discreteCINT")
            / stationary.sqrt();
    assert!(
        (discrete - recovered).abs() > 1e-3,
        "Driver et al. (2017, p. 16): discreteCINTstd is not CINTstd"
    );
    let trait_scaled = intercept / (0.5 + stationary).sqrt();
    assert!(
        (trait_scaled - recovered).abs() > 1e-3,
        "Driver et al. (2017, footnote 4): κ / √(trait + p + added) is not CINTstd"
    );
    assert_eq!(
        refuse_unstandardised_continuous_intercept_as_standardised_continuous_intercept(
            intercept, recovered
        ),
        Err(
            psychometric_core::PsychometricError::UnstandardisedContinuousInterceptIsNotStandardisedContinuousIntercept
        )
    );
    assert_eq!(
        refuse_asymptotic_standardised_continuous_intercept_as_standardised_continuous_intercept(
            asymptotic, recovered
        ),
        Err(
            psychometric_core::PsychometricError::AsymptoticStandardisedContinuousInterceptIsNotStandardisedContinuousIntercept
        )
    );
    assert_eq!(
        refuse_discrete_standardised_continuous_intercept_as_standardised_continuous_intercept(
            discrete, recovered
        ),
        Err(
            psychometric_core::PsychometricError::DiscreteStandardisedContinuousInterceptIsNotStandardisedContinuousIntercept
        )
    );
    assert_eq!(
        refuse_trait_scaled_continuous_intercept_as_standardised_continuous_intercept(
            trait_scaled, recovered
        ),
        Err(
            psychometric_core::PsychometricError::TraitScaledContinuousInterceptIsNotStandardisedContinuousIntercept
        )
    );
    assert_eq!(
        recover_standardised_continuous_intercept(intercept, 0.0, log_rate, LagClock::EventTime),
        Err(
            psychometric_core::PsychometricError::StandardisedContinuousInterceptRequiresPositiveStationaryVariance
        )
    );
    assert_eq!(
        recover_standardised_continuous_intercept(
            intercept,
            diffusion,
            log_rate,
            LagClock::DocumentTime
        ),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
}

#[test]
fn standardised_manifest_mean_is_not_unstandardised_or_total_observed_scale() {
    let mean = 0.8_f64;
    let measurement_error = 1.6_f64;
    let recovered =
        recover_standardised_manifest_mean(mean, measurement_error, LagClock::EventTime)
            .expect("MANIFESTMEANSstd");
    assert!(
        (recovered - mean / measurement_error.sqrt()).abs() < 1e-15,
        "Driver et al. (2017, p. 16 footnote 4): MANIFESTMEANSstd is τ / √θ"
    );
    assert!(
        (recovered - mean).abs() > 1e-3,
        "Driver et al. (2017, Table 2): unstandardised MANIFESTMEANS is not MANIFESTMEANSstd"
    );
    let observed = 1.2_f64 * 1.2_f64 * 0.9_f64 + measurement_error;
    let observed_scaled = mean / observed.sqrt();
    assert!(
        (observed_scaled - recovered).abs() > 1e-3,
        "Driver et al. (2017, footnote 4): τ / √(λ² Var(η) + θ) is not MANIFESTMEANSstd"
    );
    let unit = recover_standardised_manifest_mean(
        measurement_error.sqrt(),
        measurement_error,
        LagClock::EventTime,
    )
    .expect("τ = √θ");
    assert!(
        (unit - 1.0).abs() < 1e-15,
        "Driver et al. (2017, p. 16): τ / √θ equals 1 when τ = √θ"
    );
    assert_eq!(
        refuse_unstandardised_manifest_mean_as_standardised_manifest_mean(mean, recovered),
        Err(
            psychometric_core::PsychometricError::UnstandardisedManifestMeanIsNotStandardisedManifestMean
        )
    );
    assert_eq!(
        refuse_standardised_manifest_variance_as_standardised_manifest_mean(1.0, unit),
        Err(
            psychometric_core::PsychometricError::StandardisedManifestVarianceIsNotStandardisedManifestMean
        )
    );
    assert_eq!(
        refuse_observed_scaled_manifest_mean_as_standardised_manifest_mean(
            observed_scaled,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::ObservedScaledManifestMeanIsNotStandardisedManifestMean
        )
    );
    assert_eq!(
        recover_standardised_manifest_mean(mean, 0.0, LagClock::EventTime),
        Err(
            psychometric_core::PsychometricError::StandardisedManifestMeanRequiresPositiveManifestVariance
        )
    );
    assert_eq!(
        recover_standardised_manifest_mean(mean, measurement_error, LagClock::DocumentTime),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
}

#[test]
fn standardised_initial_latent_mean_is_not_unstandardised_or_t0varstd() {
    let mean = 0.8_f64;
    let initial_variance = 1.6_f64;
    let recovered =
        recover_standardised_initial_latent_mean(mean, initial_variance, LagClock::EventTime)
            .expect("T0MEANSstd");
    let expected = mean / initial_variance.sqrt();
    assert!(
        (recovered - expected).abs() < 1e-15,
        "Driver et al. (2017, p. 16 / footnote 4): T0MEANSstd is μ_0 / √p_0"
    );
    assert!(
        (recovered - mean).abs() > 1e-3,
        "Driver et al. (2017, Table 2): unstandardised T0MEANS is not T0MEANSstd"
    );
    let unit = recover_standardised_initial_latent_mean(
        initial_variance.sqrt(),
        initial_variance,
        LagClock::EventTime,
    )
    .expect("T0MEANSstd μ_0=√p_0");
    assert!(
        (unit - 1.0).abs() < 1e-15,
        "Driver et al. (2017, p. 16): μ_0 / √p_0 equals 1 when μ_0 = √p_0"
    );
    let within =
        recover_stationary_latent_variance(0.4, -0.25, LagClock::EventTime).expect("asymDIFFUSION");
    let within_scaled = mean / within.sqrt();
    assert!(
        (within_scaled - recovered).abs() > 1e-3,
        "Driver et al. (2017, p. 16): μ_0 / √asymDIFFUSION is not T0MEANSstd"
    );
    let larger = recover_standardised_initial_latent_mean(mean, 6.4, LagClock::EventTime)
        .expect("T0MEANSstd p_0=6.4");
    assert!((larger - recovered).abs() > 1e-3);
    let zero = recover_standardised_initial_latent_mean(0.0, initial_variance, LagClock::EventTime)
        .expect("zero T0MEANS");
    assert_eq!(zero.to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        recover_standardised_initial_latent_mean(mean, 0.0, LagClock::EventTime),
        Err(
            psychometric_core::PsychometricError::StandardisedInitialLatentMeanRequiresPositiveInitialLatentVariance
        )
    );
    assert_eq!(
        recover_standardised_initial_latent_mean(mean, initial_variance, LagClock::DocumentTime),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        refuse_unstandardised_initial_latent_mean_as_standardised_initial_latent_mean(
            mean, recovered
        ),
        Err(
            psychometric_core::PsychometricError::UnstandardisedInitialLatentMeanIsNotStandardisedInitialLatentMean
        )
    );
    assert_eq!(
        refuse_standardised_initial_latent_variance_as_standardised_initial_latent_mean(1.0, unit),
        Err(
            psychometric_core::PsychometricError::StandardisedInitialLatentVarianceIsNotStandardisedInitialLatentMean
        )
    );
    assert_eq!(
        refuse_within_subject_scaled_initial_latent_mean_as_standardised_initial_latent_mean(
            within_scaled, recovered
        ),
        Err(
            psychometric_core::PsychometricError::WithinSubjectScaledInitialLatentMeanIsNotStandardisedInitialLatentMean
        )
    );
}

#[test]
fn standardised_initial_latent_variance_is_not_unstandardised_mean_or_asymptotic_correlation() {
    let initial_variance = 1.6_f64;
    let recovered =
        recover_standardised_initial_latent_variance(initial_variance, LagClock::EventTime)
            .expect("T0VARstd");
    assert!(
        (recovered - 1.0).abs() < 1e-15,
        "Driver et al. (2017, p. 16 / 2017-era summary.ctsemFit.R): T0VARstd is p_0/p_0 = 1"
    );
    let larger_p0 = recover_standardised_initial_latent_variance(6.4, LagClock::EventTime)
        .expect("T0VARstd p_0=6.4");
    assert_eq!(
        larger_p0.to_bits(),
        recovered.to_bits(),
        "Driver et al. (2017, p. 16): distinct positive T0VAR recover the same T0VARstd"
    );
    assert!(
        (recovered - initial_variance).abs() > 1e-3,
        "Driver et al. (2017, Table 2): unstandardised T0VAR is not T0VARstd"
    );
    let mean_std = initial_variance.sqrt() / initial_variance.sqrt();
    assert!(
        (mean_std - recovered).abs() < 1e-15,
        "Driver et al. (2017, p. 16): T0MEANSstd equals 1 when μ_0 = √p_0"
    );
    let within_std = 1.0_f64;
    assert_eq!(
        refuse_unstandardised_initial_latent_variance_as_standardised_initial_latent_variance(
            initial_variance,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::UnstandardisedInitialLatentVarianceIsNotStandardisedInitialLatentVariance
        )
    );
    assert_eq!(
        refuse_standardised_initial_latent_mean_as_standardised_initial_latent_variance(
            mean_std, recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedInitialLatentMeanIsNotStandardisedInitialLatentVariance
        )
    );
    assert_eq!(
        refuse_standardised_asymptotic_diffusion_as_standardised_initial_latent_variance(
            within_std, recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedAsymptoticDiffusionIsNotStandardisedInitialLatentVariance
        )
    );
    assert_eq!(
        recover_standardised_initial_latent_variance(0.0, LagClock::EventTime),
        Err(
            psychometric_core::PsychometricError::StandardisedInitialLatentVarianceRequiresPositiveInitialLatentVariance
        )
    );
    assert_eq!(
        recover_standardised_initial_latent_variance(initial_variance, LagClock::DocumentTime),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
}

#[test]
fn standardised_asymptotic_diffusion_is_not_unstandardised_t0var_or_diffusion_ratio() {
    let diffusion = 0.4_f64;
    let log_rate = -0.25_f64;
    let recovered =
        recover_standardised_asymptotic_diffusion(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSIONstd");
    assert!(
        (recovered - 1.0).abs() < 1e-15,
        "Driver et al. (2017, p. 16 / 2017-era summary.ctsemFit.R): asymDIFFUSIONstd is p/p = 1"
    );
    let larger_q = recover_standardised_asymptotic_diffusion(1.6, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSIONstd q=1.6");
    assert!(
        (larger_q - recovered).abs() < 1e-15,
        "Driver et al. (2017, p. 16): distinct positive asymDIFFUSION recover the same asymDIFFUSIONstd"
    );
    let stationary =
        recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime).expect("p");
    assert!(
        (recovered - stationary).abs() > 1e-3,
        "Driver et al. (2017, p. 16): unstandardised asymDIFFUSION is not asymDIFFUSIONstd"
    );
    let t0var_std =
        recover_standardised_initial_latent_variance(1.6, LagClock::EventTime).expect("T0VARstd");
    assert!(
        (t0var_std - recovered).abs() < 1e-15,
        "Driver et al. (2017, p. 16): T0VARstd equals 1 after strictly positive p_0"
    );
    let continuous_diffusion_std = -2.0 * log_rate;
    assert!(
        (continuous_diffusion_std - recovered).abs() > 1e-3,
        "Driver et al. (2017, p. 16): DIFFUSIONstd −2a is not asymDIFFUSIONstd"
    );
    let tipred_std = 1.0_f64;
    assert_eq!(
        refuse_unstandardised_asymptotic_diffusion_as_standardised_asymptotic_diffusion(
            stationary, recovered
        ),
        Err(
            psychometric_core::PsychometricError::UnstandardisedAsymptoticDiffusionIsNotStandardisedAsymptoticDiffusion
        )
    );
    assert_eq!(
        refuse_standardised_initial_latent_variance_as_standardised_asymptotic_diffusion(
            t0var_std, recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedInitialLatentVarianceIsNotStandardisedAsymptoticDiffusion
        )
    );
    assert_eq!(
        refuse_standardised_continuous_diffusion_as_standardised_asymptotic_diffusion(
            continuous_diffusion_std,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedContinuousDiffusionIsNotStandardisedAsymptoticDiffusion
        )
    );
    assert_eq!(
        refuse_standardised_time_independent_predictor_variance_as_standardised_asymptotic_diffusion(
            tipred_std, recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedTimeIndependentPredictorVarianceIsNotStandardisedAsymptoticDiffusion
        )
    );
    assert_eq!(
        recover_standardised_asymptotic_diffusion(0.0, log_rate, LagClock::EventTime),
        Err(
            psychometric_core::PsychometricError::StandardisedAsymptoticDiffusionRequiresPositiveStationaryVariance
        )
    );
    assert_eq!(
        recover_standardised_asymptotic_diffusion(diffusion, log_rate, LagClock::DocumentTime),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_asymptotic_diffusion(diffusion, 0.25, LagClock::EventTime),
        Err(psychometric_core::PsychometricError::StationaryVarianceRequiresStableDrift)
    );
}

#[test]
fn standardised_manifest_trait_variance_is_not_unstandardised_traitstd_or_measurement_error() {
    let manifest_trait = 1.6_f64;
    let recovered =
        recover_standardised_manifest_trait_variance(manifest_trait, LagClock::EventTime)
            .expect("MANIFESTTRAITVARstd");
    assert!(
        (recovered - 1.0).abs() < 1e-15,
        "Driver et al. (2017, p. 16 / 2017-era summary.ctsemFit.R): MANIFESTTRAITVARstd is ψ/ψ = 1"
    );
    let larger_psi = recover_standardised_manifest_trait_variance(6.4, LagClock::EventTime)
        .expect("MANIFESTTRAITVARstd ψ=6.4");
    assert!(
        (larger_psi - recovered).abs() < 1e-15,
        "Driver et al. (2017, p. 16): distinct positive MANIFESTTRAITVAR recover the same MANIFESTTRAITVARstd"
    );
    let trait_std = 1.0_f64;
    assert!(
        (trait_std - recovered).abs() < 1e-15,
        "Driver et al. (2017, p. 16): TRAITVARstd and MANIFESTTRAITVARstd equal 1 and remain distinct named quantities"
    );
    let measurement_error = 0.4_f64;
    assert!(
        (measurement_error - recovered).abs() > 1e-3,
        "Driver et al. (2017, Table 2): MANIFESTVAR θ is not MANIFESTTRAITVARstd"
    );
    assert!((manifest_trait - recovered).abs() > 1e-3);
    assert_eq!(
        recover_standardised_manifest_trait_variance(0.0, LagClock::EventTime),
        Err(
            psychometric_core::PsychometricError::StandardisedManifestTraitVarianceRequiresPositiveManifestTraitVariance
        )
    );
    assert_eq!(
        recover_standardised_manifest_trait_variance(manifest_trait, LagClock::DocumentTime),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        refuse_unstandardised_manifest_trait_variance_as_standardised_manifest_trait_variance(
            manifest_trait,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::UnstandardisedManifestTraitVarianceIsNotStandardisedManifestTraitVariance
        )
    );
    assert_eq!(
        refuse_standardised_trait_variance_as_standardised_manifest_trait_variance(
            trait_std, recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedTraitVarianceIsNotStandardisedManifestTraitVariance
        )
    );
    assert_eq!(
        refuse_measurement_error_as_standardised_manifest_trait_variance(
            measurement_error,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::MeasurementErrorIsNotStandardisedManifestTraitVariance
        )
    );
}

#[test]

fn standardised_trait_variance_is_not_unstandardised_or_t0varstd() {
    let trait_variance = 1.6_f64;
    let recovered = recover_standardised_trait_variance(trait_variance, LagClock::EventTime)
        .expect("TRAITVARstd");
    assert!(
        (recovered - 1.0).abs() < 1e-15,
        "Driver et al. (2017, p. 16 / 2017-era summary.ctsemFit.R): TRAITVARstd is trait/trait = 1"
    );
    let larger_trait = recover_standardised_trait_variance(6.4, LagClock::EventTime)
        .expect("TRAITVARstd trait=6.4");
    assert!(
        (larger_trait - recovered).abs() < 1e-15,
        "Driver et al. (2017, p. 16): distinct positive TRAITVAR recover the same TRAITVARstd"
    );
    let t0var_std =
        recover_standardised_initial_latent_variance(trait_variance, LagClock::EventTime)
            .expect("T0VARstd");
    assert!(
        (t0var_std - recovered).abs() < 1e-15,
        "Driver et al. (2017, p. 16): T0VARstd and TRAITVARstd equal 1 and remain distinct named quantities"
    );
    let extra = 0.3_f64 * 0.3_f64 * 4.0_f64;
    assert!(
        (extra - recovered).abs() > 1e-3,
        "Driver et al. (2017, 2017-era addedT0TIPREDVAR): t0_b² v is not TRAITVARstd"
    );
    assert!((trait_variance - recovered).abs() > 1e-3);
    assert_eq!(
        recover_standardised_trait_variance(0.0, LagClock::EventTime),
        Err(
            psychometric_core::PsychometricError::StandardisedTraitVarianceRequiresPositiveTraitVariance
        )
    );
    assert_eq!(
        recover_standardised_trait_variance(trait_variance, LagClock::SystemTime),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        refuse_unstandardised_trait_variance_as_standardised_trait_variance(
            trait_variance,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::UnstandardisedTraitVarianceIsNotStandardisedTraitVariance
        )
    );
    assert_eq!(
        refuse_standardised_initial_latent_variance_as_standardised_trait_variance(
            t0var_std,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedInitialLatentVarianceIsNotStandardisedTraitVariance
        )
    );
    assert_eq!(
        refuse_initial_time_independent_variance_as_standardised_trait_variance(extra, recovered),
        Err(
            psychometric_core::PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedTraitVariance
        )
    );
}

#[test]
fn standardised_discrete_continuous_intercept_is_not_unstandardised_continuous_or_asymptotic() {
    let intercept = 0.4_f64;
    let diffusion = 0.8_f64;
    let log_rate = -0.5_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_standardised_discrete_continuous_intercept(
        intercept,
        diffusion,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("discreteCINTstd");
    let stationary =
        recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime).expect("p");
    let increment = recover_discrete_continuous_intercept_effect(
        intercept,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("discreteCINT");
    assert!(
        (recovered - increment / stationary.sqrt()).abs() < 1e-15,
        "Driver et al. (2017, p. 16 footnote 4): discreteCINTstd is A^{{-1}}[e^{{A Δt}} − I] κ / √p"
    );
    assert!(
        (recovered - increment).abs() > 1e-3,
        "Driver et al. (2017, Table 2): unstandardised discreteCINT is not discreteCINTstd"
    );
    let continuous = intercept / stationary.sqrt();
    assert!(
        (continuous - recovered).abs() > 1e-3,
        "Driver et al. (2017, p. 16): CINTstd is not discreteCINTstd"
    );
    let asymptotic =
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
            .expect("asymCINT")
            / stationary.sqrt();
    assert!(
        (asymptotic - recovered).abs() > 1e-3,
        "Driver et al. (2017, p. 16): asymCINTstd is not discreteCINTstd"
    );
    assert_eq!(
        refuse_unstandardised_discrete_continuous_intercept_as_standardised_discrete_continuous_intercept(
            increment, recovered
        ),
        Err(
            psychometric_core::PsychometricError::UnstandardisedDiscreteContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept
        )
    );
    assert_eq!(
        refuse_standardised_continuous_intercept_as_standardised_discrete_continuous_intercept(
            continuous, recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept
        )
    );
    assert_eq!(
        refuse_asymptotic_standardised_continuous_intercept_as_standardised_discrete_continuous_intercept(
            asymptotic, recovered
        ),
        Err(
            psychometric_core::PsychometricError::AsymptoticStandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept
        )
    );
    assert_eq!(
        recover_standardised_discrete_continuous_intercept(
            intercept,
            0.0,
            log_rate,
            event_delta,
            LagClock::EventTime
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedDiscreteContinuousInterceptRequiresPositiveStationaryVariance
        )
    );
    assert_eq!(
        recover_standardised_discrete_continuous_intercept(
            intercept,
            diffusion,
            log_rate,
            event_delta,
            LagClock::DocumentTime
        ),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn standardised_asymptotic_continuous_intercept_is_not_unstandardised_or_cintstd() {
    let intercept = 0.4_f64;
    let diffusion = 0.8_f64;
    let log_rate = -0.5_f64;
    let recovered = recover_standardised_asymptotic_continuous_intercept(
        intercept,
        diffusion,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymCINTstd");
    let stationary =
        recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime).expect("p");
    let asymptotic =
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
            .expect("asymCINT");
    assert!(
        (recovered - asymptotic / stationary.sqrt()).abs() < 1e-15,
        "Driver et al. (2017, p. 16 footnote 4): asymCINTstd is (−κ / a) / √p"
    );
    assert!(
        (recovered - asymptotic).abs() > 1e-3,
        "Driver et al. (2017, Table 2): unstandardised asymCINT is not asymCINTstd"
    );
    let continuous_std = intercept / stationary.sqrt();
    assert!(
        (continuous_std - recovered).abs() > 1e-3,
        "Driver et al. (2017, p. 16): CINTstd is not asymCINTstd"
    );
    let discrete =
        recover_discrete_continuous_intercept_effect(intercept, log_rate, 1.0, LagClock::EventTime)
            .expect("discreteCINT")
            / stationary.sqrt();
    assert!(
        (discrete - recovered).abs() > 1e-3,
        "Driver et al. (2017, p. 16): discreteCINTstd is not asymCINTstd"
    );
    let later =
        recover_discrete_continuous_intercept_effect(intercept, log_rate, 2.5, LagClock::EventTime)
            .expect("discreteCINT Δt=2.5")
            / stationary.sqrt();
    assert!(
        (later - recovered).abs() > 1e-3,
        "Driver et al. (2017, p. 16): a later event interval changes discreteCINTstd and not asymCINTstd"
    );
    let zero = recover_standardised_asymptotic_continuous_intercept(
        0.0,
        diffusion,
        log_rate,
        LagClock::EventTime,
    )
    .expect("zero CINT");
    assert_eq!(zero.to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        recover_standardised_asymptotic_continuous_intercept(
            intercept,
            0.0,
            log_rate,
            LagClock::EventTime
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedAsymptoticContinuousInterceptRequiresPositiveStationaryVariance
        )
    );
    assert_eq!(
        recover_standardised_asymptotic_continuous_intercept(
            intercept,
            diffusion,
            0.5,
            LagClock::EventTime
        ),
        Err(psychometric_core::PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_standardised_asymptotic_continuous_intercept(
            intercept,
            diffusion,
            log_rate,
            LagClock::DocumentTime
        ),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        refuse_unstandardised_asymptotic_continuous_intercept_as_standardised_asymptotic_continuous_intercept(
            asymptotic, recovered
        ),
        Err(
            psychometric_core::PsychometricError::UnstandardisedAsymptoticContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept
        )
    );
    assert_eq!(
        refuse_standardised_continuous_intercept_as_standardised_asymptotic_continuous_intercept(
            continuous_std, recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept
        )
    );
    assert_eq!(
        refuse_discrete_standardised_continuous_intercept_as_standardised_asymptotic_continuous_intercept(
            discrete, recovered
        ),
        Err(
            psychometric_core::PsychometricError::DiscreteStandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept
        )
    );
}
#[test]
fn standardised_manifest_variance_is_not_unstandardised_traitstd_or_observed_variance() {
    let measurement_error = 0.4_f64;
    let recovered = recover_standardised_manifest_variance(measurement_error, LagClock::EventTime)
        .expect("MANIFESTVARstd");
    assert!(
        (recovered - 1.0).abs() < 1e-15,
        "Driver et al. (2017, p. 16 / 2017-era summary.ctsemFit.R): MANIFESTVARstd is θ/θ = 1"
    );
    let larger_theta = recover_standardised_manifest_variance(1.6, LagClock::EventTime)
        .expect("MANIFESTVARstd θ=1.6");
    assert_eq!(
        larger_theta.to_bits(),
        recovered.to_bits(),
        "Driver et al. (2017, p. 16): distinct positive MANIFESTVAR recover the same MANIFESTVARstd"
    );
    let manifest_trait_std = 1.0_f64;
    assert!(
        (manifest_trait_std - recovered).abs() < 1e-15,
        "Driver et al. (2017, p. 16): MANIFESTTRAITVARstd and MANIFESTVARstd equal 1 and remain distinct named quantities"
    );
    let observed = recover_manifest_observed_variance(2.0, 0.4, measurement_error).expect("Var(y)");
    assert!(
        (observed - recovered).abs() > 1e-3,
        "Driver et al. (2017, Eq. 5): Var(y) is not MANIFESTVARstd"
    );
    assert!((measurement_error - recovered).abs() > 1e-3);
    assert_eq!(
        recover_standardised_manifest_variance(0.0, LagClock::EventTime),
        Err(
            psychometric_core::PsychometricError::StandardisedManifestVarianceRequiresPositiveManifestVariance
        )
    );
    assert_eq!(
        recover_standardised_manifest_variance(measurement_error, LagClock::DocumentTime),
        Err(psychometric_core::PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        refuse_unstandardised_manifest_variance_as_standardised_manifest_variance(
            measurement_error,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::UnstandardisedManifestVarianceIsNotStandardisedManifestVariance
        )
    );
    assert_eq!(
        refuse_standardised_manifest_trait_variance_as_standardised_manifest_variance(
            manifest_trait_std,
            recovered
        ),
        Err(
            psychometric_core::PsychometricError::StandardisedManifestTraitVarianceIsNotStandardisedManifestVariance
        )
    );
    assert_eq!(
        refuse_observed_variance_as_standardised_manifest_variance(observed, recovered),
        Err(
            psychometric_core::PsychometricError::ObservedVarianceIsNotStandardisedManifestVariance
        )
    );
}
