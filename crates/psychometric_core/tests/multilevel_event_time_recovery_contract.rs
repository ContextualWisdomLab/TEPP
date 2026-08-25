//! True-parameter recovery for multilevel OLS, event-time log-rate, and CWC lags.
#![allow(clippy::cast_precision_loss)]

use psychometric_core::{
    ClusteredEventScore, ClusteredScore, EventOccasion, IndicatorKind, LagClock,
    LaggedWithinResidual, PsychometricError, map_discrete_lag_across_event_intervals,
    ordinary_least_squares_slope, recover_asymptotic_continuous_intercept,
    recover_asymptotic_time_independent_observed_variance,
    recover_asymptotic_time_independent_predictor_effect,
    recover_asymptotic_time_independent_predictor_variance,
    recover_cluster_mean_within_between_slopes, recover_discrete_constant_predictor_effect,
    recover_discrete_continuous_intercept_effect, recover_discrete_lag_from_log_rate,
    recover_discrete_lagged_latent_covariance, recover_discrete_latent_mean,
    recover_discrete_latent_mean_with_extra_process,
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
    recover_discrete_time_varying_predictor_effect, recover_event_series_mean_log_rate,
    recover_event_time_discrete_lag_and_log_rate, recover_initial_time_dependent_observed_variance,
    recover_initial_time_dependent_predictor_carry,
    recover_initial_time_dependent_predictor_effect,
    recover_initial_time_dependent_predictor_variance,
    recover_initial_time_independent_observed_variance,
    recover_initial_time_independent_predictor_carry,
    recover_initial_time_independent_predictor_effect,
    recover_initial_time_independent_predictor_variance,
    recover_irregular_centered_residual_log_rate, recover_kish_weighted_slope,
    recover_level_change_continuous_intercept, recover_level_change_discrete_increment,
    recover_level_change_extra_process_contribution,
    recover_level_change_extra_process_contribution_after,
    recover_manifest_lagged_observed_covariance, recover_manifest_observed_mean,
    recover_manifest_observed_variance, recover_manifest_trait_plus_state_observed_variance,
    recover_predetermined_initial_latent_variance, recover_predetermined_initial_observed_variance,
    recover_predetermined_lagged_latent_covariance,
    recover_predetermined_lagged_observed_covariance,
    recover_predetermined_later_lagged_latent_covariance,
    recover_predetermined_later_lagged_observed_covariance,
    recover_predetermined_later_latent_variance, recover_predetermined_later_observed_variance,
    recover_predetermined_later_start_later_latent_variance,
    recover_predetermined_later_start_later_observed_variance,
    recover_standardised_asymptotic_time_independent_predictor_effect,
    recover_standardised_asymptotic_time_independent_predictor_variance,
    recover_standardised_continuous_diffusion, recover_standardised_continuous_drift,
    recover_standardised_continuous_intercept,
    recover_standardised_continuous_time_dependent_predictor_effect,
    recover_standardised_continuous_time_independent_predictor_effect,
    recover_standardised_discrete_diffusion, recover_standardised_discrete_drift,
    recover_standardised_discrete_time_independent_predictor_effect,
    recover_standardised_initial_latent_variance,
    recover_standardised_initial_time_dependent_predictor_effect,
    recover_standardised_initial_time_independent_predictor_effect,
    recover_standardised_manifest_trait_variance, recover_standardised_manifest_variance,
    recover_standardised_trait_variance, recover_stationary_initial_latent_mean,
    recover_stationary_initial_latent_variance, recover_stationary_initial_observed_mean,
    recover_stationary_initial_observed_variance, recover_stationary_lagged_latent_covariance,
    recover_stationary_lagged_observed_covariance, recover_stationary_latent_variance,
    recover_stationary_later_latent_variance, recover_stationary_later_observed_variance,
    recover_time_dependent_predictor_impulse, recover_time_dependent_predictor_impulse_carry,
    recover_trait_plus_state_lagged_covariance, recover_trait_plus_state_latent_variance,
    recover_within_residual_event_time_log_rate,
    refuse_after_extra_process_contribution_as_observed_mean,
    refuse_after_extra_process_latent_mean_as_observed_mean,
    refuse_asymptotic_continuous_intercept_as_asymptotic_time_independent_effect,
    refuse_asymptotic_continuous_intercept_as_continuous_intercept,
    refuse_asymptotic_continuous_intercept_as_discrete_increment,
    refuse_asymptotic_continuous_intercept_as_initial_latent_mean,
    refuse_asymptotic_continuous_intercept_observed_mean_as_stationary_initial_observed_mean,
    refuse_asymptotic_time_independent_effect_as_coefficient,
    refuse_asymptotic_time_independent_effect_as_continuous_intercept,
    refuse_asymptotic_time_independent_effect_as_discrete_effect,
    refuse_asymptotic_time_independent_effect_as_time_dependent_impulse,
    refuse_asymptotic_time_independent_observed_variance_as_asymptotic_time_independent_variance,
    refuse_asymptotic_time_independent_observed_variance_as_initial_time_independent_observed_variance,
    refuse_asymptotic_time_independent_observed_variance_as_measurement_error,
    refuse_asymptotic_time_independent_observed_variance_as_standardised_asymptotic_time_independent_variance,
    refuse_asymptotic_time_independent_observed_variance_as_stationary_observed_variance,
    refuse_asymptotic_time_independent_variance_as_asymptotic_effect,
    refuse_asymptotic_time_independent_variance_as_stationary_within_subject,
    refuse_asymptotic_time_independent_variance_as_trait_variance,
    refuse_continuous_intercept_as_discrete_mean_increment,
    refuse_continuous_intercept_as_initial_latent_mean,
    refuse_continuous_intercept_as_manifest_means, refuse_difference_quotient_as_local_rate,
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
    refuse_initial_time_dependent_observed_variance_as_initial_observed_variance,
    refuse_initial_time_dependent_observed_variance_as_initial_time_dependent_variance,
    refuse_initial_time_dependent_observed_variance_as_initial_time_independent_observed_variance,
    refuse_initial_time_dependent_observed_variance_as_measurement_error,
    refuse_initial_time_dependent_variance_as_initial_latent_variance,
    refuse_initial_time_dependent_variance_as_initial_time_dependent_covariance,
    refuse_initial_time_dependent_variance_as_initial_time_independent_variance,
    refuse_initial_time_dependent_variance_as_standardised_initial_time_dependent_effect,
    refuse_initial_time_dependent_variance_as_trait_variance,
    refuse_initial_time_independent_carry_as_initial_effect,
    refuse_initial_time_independent_coefficient_as_initial_effect,
    refuse_initial_time_independent_effect_as_continuous_intercept,
    refuse_initial_time_independent_effect_as_process_increment,
    refuse_initial_time_independent_effect_as_time_dependent_impulse,
    refuse_initial_time_independent_observed_mean_as_initial_time_dependent_observed_mean,
    refuse_initial_time_independent_observed_variance_as_asymptotic_time_independent_observed_variance,
    refuse_initial_time_independent_observed_variance_as_initial_observed_variance,
    refuse_initial_time_independent_observed_variance_as_initial_time_independent_variance,
    refuse_initial_time_independent_observed_variance_as_measurement_error,
    refuse_initial_time_independent_variance_as_asymptotic_time_independent_variance,
    refuse_initial_time_independent_variance_as_initial_latent_variance,
    refuse_initial_time_independent_variance_as_standardised_asymptotic_time_independent_variance,
    refuse_initial_time_independent_variance_as_standardised_initial_latent_variance,
    refuse_initial_time_independent_variance_as_standardised_initial_time_independent_effect,
    refuse_initial_time_independent_variance_as_standardised_trait_variance,
    refuse_initial_time_independent_variance_as_trait_variance,
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
    refuse_measurement_error_as_predetermined_initial_observed_variance,
    refuse_measurement_error_as_predetermined_lagged_observed_covariance,
    refuse_measurement_error_as_predetermined_later_lagged_observed_covariance,
    refuse_measurement_error_as_predetermined_later_observed_variance,
    refuse_measurement_error_as_predetermined_later_start_later_observed_variance,
    refuse_measurement_error_as_standardised_manifest_trait_variance,
    refuse_measurement_error_as_stationary_lagged_observed_covariance,
    refuse_measurement_error_as_stationary_later_observed_variance,
    refuse_observed_variance_as_standardised_manifest_variance,
    refuse_pooled_discrete_lag_across_unequal_intervals,
    refuse_predetermined_initial_latent_variance_as_initial_latent_variance,
    refuse_predetermined_initial_latent_variance_as_lagged_latent_covariance,
    refuse_predetermined_initial_latent_variance_as_later_latent_variance,
    refuse_predetermined_initial_latent_variance_as_observed_variance,
    refuse_predetermined_initial_latent_variance_as_stationary_initial_latent_variance,
    refuse_predetermined_lagged_latent_covariance_as_decayed_total,
    refuse_predetermined_lagged_latent_covariance_as_initial_latent_variance,
    refuse_predetermined_lagged_latent_covariance_as_later_latent_variance,
    refuse_predetermined_lagged_latent_covariance_as_observed_covariance,
    refuse_predetermined_lagged_latent_covariance_as_stationary_lagged_covariance,
    refuse_predetermined_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance,
    refuse_predetermined_later_lagged_latent_covariance_as_decayed_later_total,
    refuse_predetermined_later_lagged_latent_covariance_as_later_latent_variance,
    refuse_predetermined_later_lagged_latent_covariance_as_observed_covariance,
    refuse_predetermined_later_lagged_latent_covariance_as_predetermined_lagged_covariance,
    refuse_predetermined_later_lagged_latent_covariance_as_stationary_lagged_covariance,
    refuse_predetermined_later_lagged_observed_covariance_as_predetermined_later_start_later_observed_variance,
    refuse_predetermined_later_latent_variance_as_discrete_variance,
    refuse_predetermined_later_latent_variance_as_initial_latent_variance,
    refuse_predetermined_later_latent_variance_as_observed_variance,
    refuse_predetermined_later_latent_variance_as_stationary_later_latent_variance,
    refuse_predetermined_later_observed_variance_as_predetermined_initial_observed_variance,
    refuse_predetermined_later_observed_variance_as_predetermined_lagged_observed_covariance,
    refuse_predetermined_later_observed_variance_as_predetermined_later_lagged_observed_covariance,
    refuse_predetermined_later_observed_variance_as_predetermined_later_start_later_observed_variance,
    refuse_predetermined_later_start_later_latent_variance_as_decayed_later_total,
    refuse_predetermined_later_start_later_latent_variance_as_lag_interval_later_latent_variance,
    refuse_predetermined_later_start_later_latent_variance_as_later_lagged_covariance,
    refuse_predetermined_later_start_later_latent_variance_as_later_latent_variance,
    refuse_predetermined_later_start_later_latent_variance_as_observed_variance,
    refuse_predetermined_later_start_later_latent_variance_as_stationary_later_latent_variance,
    refuse_process_noise_as_unconditional_variance,
    refuse_standardised_asymptotic_time_independent_effect_as_standardised_discrete_time_independent_effect,
    refuse_standardised_asymptotic_continuous_intercept_as_standardised_continuous_intercept,
    refuse_standardised_asymptotic_time_independent_effect_as_standardised_continuous_time_independent_effect,
    refuse_standardised_asymptotic_time_independent_effect_as_standardised_initial_time_independent_effect,
    refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion,
    refuse_standardised_discrete_continuous_intercept_as_standardised_continuous_intercept,
    refuse_standardised_continuous_time_independent_effect_as_standardised_initial_time_dependent_effect,
    refuse_standardised_continuous_time_dependent_effect_as_standardised_initial_time_dependent_effect,
    refuse_standardised_continuous_time_independent_effect_as_standardised_continuous_time_dependent_effect,
    refuse_standardised_continuous_time_independent_effect_as_standardised_initial_time_independent_effect,
    refuse_standardised_discrete_diffusion_as_standardised_continuous_diffusion,
    refuse_standardised_discrete_drift_as_standardised_continuous_drift,
    refuse_standardised_discrete_time_dependent_effect_as_standardised_continuous_time_dependent_effect,
    refuse_standardised_discrete_time_independent_effect_as_standardised_asymptotic_time_independent_effect,
    refuse_standardised_discrete_time_independent_effect_as_standardised_continuous_time_independent_effect,
    refuse_standardised_initial_latent_variance_as_standardised_trait_variance,
    refuse_standardised_initial_time_dependent_effect_as_standardised_initial_latent_variance,
    refuse_standardised_initial_time_independent_effect_as_standardised_initial_time_dependent_effect,
    refuse_standardised_manifest_trait_variance_as_standardised_manifest_variance,
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
    refuse_stationary_initial_observed_variance_as_predetermined_initial_observed_variance,
    refuse_stationary_initial_observed_variance_as_stationary_lagged_observed_covariance,
    refuse_stationary_lagged_latent_covariance_as_decayed_stationary_variance,
    refuse_stationary_lagged_latent_covariance_as_observed_covariance,
    refuse_stationary_lagged_latent_covariance_as_stationary_initial_latent_variance,
    refuse_stationary_lagged_observed_covariance_as_predetermined_lagged_observed_covariance,
    refuse_stationary_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance,
    refuse_stationary_lagged_observed_covariance_as_stationary_later_observed_variance,
    refuse_stationary_later_latent_variance_as_discrete_variance,
    refuse_stationary_later_latent_variance_as_lagged_covariance,
    refuse_stationary_later_latent_variance_as_observed_variance,
    refuse_stationary_later_latent_variance_as_process_noise,
    refuse_stationary_later_observed_variance_as_predetermined_later_observed_variance,
    refuse_stationary_later_observed_variance_as_predetermined_later_start_later_observed_variance,
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
    refuse_trait_contaminated_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect,
    refuse_trait_contaminated_continuous_diffusion_as_standardised_continuous_diffusion,
    refuse_trait_contaminated_continuous_drift_as_standardised_continuous_drift,
    refuse_trait_contaminated_discrete_time_independent_effect_as_standardised_discrete_time_independent_effect,
    refuse_trait_contaminated_continuous_intercept_as_standardised_continuous_intercept,
    refuse_trait_contaminated_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect,
    refuse_trait_contaminated_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect,
    refuse_trait_contaminated_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect,
    refuse_trait_contaminated_initial_time_independent_effect_as_standardised_initial_time_independent_effect,
    refuse_trait_contaminated_process_noise_as_standardised_discrete_diffusion,
    refuse_trait_plus_state_autocorrelation_as_standardised_discrete_drift,
    refuse_trait_plus_state_lagged_covariance_as_stationary_lagged_latent_covariance,
    refuse_trait_variance_as_process_noise, refuse_trait_variance_as_standardisation_variance,
    refuse_trait_variance_as_stationary_within_subject,
    refuse_unmatched_time_varying_predictor_interval,
    refuse_unstandardised_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect,
    refuse_unstandardised_asymptotic_time_independent_variance_as_standardised_asymptotic_time_independent_variance,
    refuse_unstandardised_continuous_diffusion_as_standardised_continuous_diffusion,
    refuse_unstandardised_continuous_drift_as_standardised_continuous_drift,
    refuse_unstandardised_continuous_intercept_as_standardised_continuous_intercept,
    refuse_unstandardised_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect,
    refuse_unstandardised_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect,
    refuse_unstandardised_discrete_diffusion_as_standardised_discrete_diffusion,
    refuse_unstandardised_discrete_drift_as_standardised_discrete_drift,
    refuse_unstandardised_discrete_time_independent_effect_as_standardised_discrete_time_independent_effect,
    refuse_unstandardised_initial_latent_variance_as_standardised_initial_latent_variance,
    refuse_unstandardised_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect,
    refuse_unstandardised_initial_time_independent_effect_as_standardised_initial_time_independent_effect,
    refuse_unstandardised_manifest_trait_variance_as_standardised_manifest_trait_variance,
    refuse_unstandardised_manifest_variance_as_standardised_manifest_variance,
    refuse_unstandardised_trait_variance_as_standardised_trait_variance,
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
    for (cluster, person_mean, start) in [(1_u64, 8.0_f64, 1.0_f64), (2, 5.0, 1.4)] {
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
    for (cluster, person_mean, start) in [(1_u64, 8.0_f64, 1.0_f64), (2, 1.0, 1.4)] {
        let time = cluster as f64;
        pooled.push(EventOccasion {
            event_time: time,
            score: person_mean + start,
        });
    }
    let pooled_rate = recover_event_series_mean_log_rate(&pooled, LagClock::EventTime)
        .expect("pooled positive lag");
    let pooled_error = rmse(&[true_drift], &[pooled_rate]);
    assert!(
        within_error < pooled_error,
        "CWC lag RMSE {within_error} should beat pooled {pooled_error}"
    );
    assert!(within_error < 0.25, "CWC lag RMSE {within_error} too large");
}

#[test]
fn within_residual_event_time_log_rate_rejects_nonfinite_residual_ratios() {
    let rows = [
        ClusteredEventScore {
            cluster_key: 1,
            event_time: 0.0,
            score: 1e-308,
        },
        ClusteredEventScore {
            cluster_key: 1,
            event_time: 1.0,
            score: f64::MAX,
        },
        ClusteredEventScore {
            cluster_key: 1,
            event_time: 2.0,
            score: -f64::MAX,
        },
        ClusteredEventScore {
            cluster_key: 2,
            event_time: 0.0,
            score: 0.0,
        },
        ClusteredEventScore {
            cluster_key: 2,
            event_time: 1.0,
            score: 1.0,
        },
    ];
    assert_eq!(
        recover_within_residual_event_time_log_rate(&rows, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    let zero_residual_rows = [
        ClusteredEventScore {
            cluster_key: 1,
            event_time: 0.0,
            score: 0.0,
        },
        ClusteredEventScore {
            cluster_key: 1,
            event_time: 1.0,
            score: 1.0,
        },
        ClusteredEventScore {
            cluster_key: 1,
            event_time: 2.0,
            score: 2.0,
        },
        ClusteredEventScore {
            cluster_key: 2,
            event_time: 0.0,
            score: 0.0,
        },
        ClusteredEventScore {
            cluster_key: 2,
            event_time: 1.0,
            score: 1.0,
        },
    ];
    assert_eq!(
        recover_within_residual_event_time_log_rate(&zero_residual_rows, LagClock::EventTime,),
        Err(PsychometricError::InvalidNumericInput)
    );
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
    let finite_rewrite =
        recover_discrete_latent_variance(1e-308, 1e-308, 400.0, 1.0, LagClock::EventTime)
            .expect("finite exponential rewrite");
    assert!(finite_rewrite.is_finite());
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
            1.0,
            -0.5,
            0.3,
            1e308,
            2.0,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
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
fn discrete_observed_mean_with_impulse_recovers_driver_equation_five() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_impulse(
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
    let expected = manifest_mean + loading * composed;
    let error = rmse(&[expected], &[observed]);
    assert!(
        error < 1e-15,
        "Driver Eq. 5 of Eq. 3 contemporaneous impulse RMSE {error}"
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
        "τ + λ μ_t is not contemporaneous-impulse E(y_t): RMSE {evolved_error} must exceed {error}"
    );
    let intercept_error = rmse(&[expected], &[manifest_mean]);
    assert!(
        intercept_error > error,
        "MANIFESTMEANS is not contemporaneous-impulse E(y_t): RMSE {intercept_error} must exceed {error}"
    );
    let latent_error = rmse(&[expected], &[composed]);
    assert!(
        latent_error > error,
        "evolved-plus-impulse latent mean is not E(y_t): RMSE {latent_error} must exceed {error}"
    );
}

#[test]
fn discrete_observed_mean_with_impulse_refuses_evolved_mean_and_carry() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_impulse(
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
    let expected = manifest_mean + loading * composed;
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
    let carry_error = rmse(&[expected], &[carried_observed]);
    assert!(
        carry_error > error,
        "τ + λ(μ_t + carry) is not contemporaneous-impulse E(y_t): RMSE {carry_error} must exceed {error}"
    );
    let zero_loading = recover_discrete_observed_mean_with_impulse(
        0.0,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("lambda0");
    let dropped_error = rmse(&[expected], &[zero_loading]);
    assert!(
        dropped_error > error,
        "zero loading is τ, not τ + λ(μ_t + m x): RMSE {dropped_error} must exceed {error}"
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_impulse_observed_mean(evolved_observed, observed),
        Err(PsychometricError::EvolvedObservedMeanIsNotImpulseObservedMean)
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_impulse_carry_observed_mean(observed, carried_observed),
        Err(PsychometricError::ImpulseObservedMeanIsNotImpulseCarryObservedMean)
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(composed, observed),
        Err(PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, observed),
        Err(PsychometricError::ManifestMeansIsNotObservedMean)
    );
}

#[test]
fn discrete_observed_mean_with_impulse_refuses_overflow_and_non_event_clocks() {
    assert_eq!(
        recover_discrete_observed_mean_with_impulse(
            1e308,
            2.0,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_impulse(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_impulse(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_impulse(
            1e308,
            0.0,
            0.0,
            0.0,
            1e308,
            1.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    let scaled = recover_discrete_observed_mean_with_impulse(
        1e308,
        1e-308,
        0.0,
        0.0,
        0.0,
        3.0,
        0.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("scale");
    assert!(
        (scaled - 1.0).abs() < 1e-15,
        "Driver Eq. 5 of Eq. 3 impulse must keep λ=1e308, μ=1e-308: got {scaled}"
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
        recover_discrete_time_independent_predictor_effect(
            0.4,
            3.0,
            1.0,
            1.0,
            f64::NAN,
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
    assert_eq!(
        recover_discrete_latent_mean_with_time_independent_predictor(
            1.0,
            -0.5,
            0.3,
            1e308,
            2.0,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn initial_time_independent_predictor_recovers_driver_table_three_t0_shift() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let shift =
        recover_initial_time_independent_predictor_effect(effect, predictor).expect("t0-tipred");
    let error = rmse(&[1.2], &[shift]);
    assert!(
        error < 1e-15,
        "Driver Table 3 T0TIPREDEFFECT RMSE {error}: got {shift}"
    );
    let carry = recover_initial_time_independent_predictor_carry(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("t0-carry");
    let expected_carry = 1.2 * (drift * delta).exp();
    let carry_error = rmse(&[expected_carry], &[carry]);
    assert!(
        carry_error < 1e-15,
        "Driver Eq. 3 first-summand T0TIPREDEFFECT carry RMSE {carry_error}: got {carry}"
    );
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
    assert!(rmse(&[carry], &[shift]) > rmse(&[expected_carry], &[carry]));
    assert!(rmse(&[carry], &[increment]) > rmse(&[expected_carry], &[carry]));
    assert!(rmse(&[shift], &[increment]) > rmse(&[1.2], &[shift]));
    assert!(rmse(&[shift], &[intercept_effect]) > rmse(&[1.2], &[shift]));
    assert!(rmse(&[shift], &[effect]) > rmse(&[1.2], &[shift]));
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
    let evolved =
        recover_discrete_latent_mean(1.0, drift, 0.3, delta, LagClock::EventTime).expect("mu-t");
    let composed_error = rmse(&[evolved + carry], &[composed]);
    assert!(
        composed_error < 1e-15,
        "Driver Eq. 3 μ_t + e^{{A Δt}} t0_b z RMSE {composed_error}: got {composed}"
    );
    assert_eq!(
        refuse_initial_time_independent_effect_as_process_increment(shift, increment),
        Err(PsychometricError::InitialTimeIndependentEffectIsNotProcessIncrement)
    );
    assert_eq!(
        refuse_initial_time_independent_carry_as_initial_effect(carry, shift),
        Err(PsychometricError::InitialTimeIndependentCarryIsNotInitialEffect)
    );
    assert_eq!(
        refuse_initial_time_independent_effect_as_continuous_intercept(shift, effect),
        Err(PsychometricError::InitialTimeIndependentEffectIsNotContinuousIntercept)
    );
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
    assert_eq!(
        refuse_initial_time_independent_effect_as_time_dependent_impulse(shift, impulse),
        Err(PsychometricError::InitialTimeIndependentEffectIsNotTimeDependentImpulse)
    );
    assert_eq!(
        refuse_initial_time_independent_coefficient_as_initial_effect(effect, shift),
        Err(PsychometricError::InitialTimeIndependentCoefficientIsNotInitialEffect)
    );
}

#[test]
fn initial_time_independent_predictor_refuses_overflow_and_non_event_clocks() {
    assert_eq!(
        recover_initial_time_independent_predictor_effect(1e308, 2.0),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_initial_time_independent_predictor_carry(0.4, 3.0, -0.5, 2.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_initial_time_independent_predictor_carry(0.4, 3.0, 1e308, 2.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_initial_time_independent_predictor_carry(2.0, 0.5, 710.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    let finite_rewrite = recover_initial_time_independent_predictor_carry(
        1e-308,
        1.0,
        700.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("t0-ti-log-rewrite");
    let expected_rewrite = (1e-308_f64.ln() + 700.0).exp();
    assert!((finite_rewrite - expected_rewrite).abs() / expected_rewrite < 1e-12);
    assert_eq!(
        recover_discrete_latent_mean_with_initial_time_independent_predictor(
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
#[allow(clippy::too_many_lines)]
fn initial_time_dependent_predictor_recovers_driver_table_three_t0_shift() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let shift =
        recover_initial_time_dependent_predictor_effect(effect, predictor).expect("t0-tdpred");
    let error = rmse(&[1.2], &[shift]);
    assert!(
        error < 1e-15,
        "Driver Table 3 T0TDPREDEFFECT RMSE {error}: got {shift}"
    );
    let carry = recover_initial_time_dependent_predictor_carry(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("t0-td-carry");
    let expected_carry = 1.2 * (drift * delta).exp();
    let carry_error = rmse(&[expected_carry], &[carry]);
    assert!(
        carry_error < 1e-15,
        "Driver Eq. 3 first-summand T0TDPREDEFFECT carry RMSE {carry_error}: got {carry}"
    );
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
    let impulse_carry = recover_time_dependent_predictor_impulse_carry(
        effect,
        predictor,
        drift,
        delta,
        1.0,
        LagClock::EventTime,
    )
    .expect("td-carry");
    assert!(rmse(&[carry], &[shift]) > rmse(&[expected_carry], &[carry]));
    assert!(rmse(&[carry], &[increment]) > rmse(&[expected_carry], &[carry]));
    assert!(rmse(&[shift], &[increment]) > rmse(&[1.2], &[shift]));
    assert!(rmse(&[shift], &[intercept_effect]) > rmse(&[1.2], &[shift]));
    assert!(rmse(&[shift], &[effect]) > rmse(&[1.2], &[shift]));
    assert!(rmse(&[carry], &[impulse_carry]) > rmse(&[expected_carry], &[carry]));
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
    let evolved =
        recover_discrete_latent_mean(1.0, drift, 0.3, delta, LagClock::EventTime).expect("mu-t");
    let composed_error = rmse(&[evolved + carry], &[composed]);
    assert!(
        composed_error < 1e-15,
        "Driver Eq. 3 μ_t + e^{{A Δt}} t0_m x0 RMSE {composed_error}: got {composed}"
    );
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
    let tipred_shift =
        recover_initial_time_independent_predictor_effect(effect, predictor).expect("t0-tipred");
    assert_eq!(
        refuse_initial_time_dependent_effect_as_contemporaneous_impulse(shift, impulse),
        Err(PsychometricError::InitialTimeDependentEffectIsNotContemporaneousImpulse)
    );
    assert_eq!(
        refuse_initial_time_dependent_carry_as_initial_effect(carry, shift),
        Err(PsychometricError::InitialTimeDependentCarryIsNotInitialEffect)
    );
    assert_eq!(
        refuse_initial_time_dependent_effect_as_continuous_intercept(shift, effect),
        Err(PsychometricError::InitialTimeDependentEffectIsNotContinuousIntercept)
    );
    assert_eq!(
        refuse_initial_time_dependent_effect_as_process_increment(shift, increment),
        Err(PsychometricError::InitialTimeDependentEffectIsNotProcessIncrement)
    );
    assert_eq!(
        refuse_initial_time_dependent_effect_as_initial_time_independent_effect(
            shift,
            tipred_shift
        ),
        Err(PsychometricError::InitialTimeDependentEffectIsNotInitialTimeIndependentEffect)
    );
    assert_eq!(
        refuse_initial_time_dependent_coefficient_as_initial_effect(effect, shift),
        Err(PsychometricError::InitialTimeDependentCoefficientIsNotInitialEffect)
    );
    assert_eq!(
        refuse_initial_time_dependent_carry_as_impulse_carry(carry, impulse_carry),
        Err(PsychometricError::InitialTimeDependentCarryIsNotImpulseCarry)
    );
}

#[test]
fn initial_time_dependent_predictor_refuses_overflow_and_non_event_clocks() {
    assert_eq!(
        recover_initial_time_dependent_predictor_effect(1e308, 2.0),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_initial_time_dependent_predictor_carry(0.4, 3.0, -0.5, 2.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_initial_time_dependent_predictor_carry(0.4, 3.0, 1e308, 2.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_initial_time_dependent_predictor_carry(2.0, 0.5, 710.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    let finite_rewrite = recover_initial_time_dependent_predictor_carry(
        1e-308,
        1.0,
        700.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("t0-td-log-rewrite");
    let expected_rewrite = (1e-308_f64.ln() + 700.0).exp();
    assert!((finite_rewrite - expected_rewrite).abs() / expected_rewrite < 1e-12);
    assert_eq!(
        recover_discrete_latent_mean_with_initial_time_dependent_predictor(
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
fn discrete_observed_mean_with_initial_time_independent_predictor_recovers_driver_equation_five() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_initial_time_independent_predictor(
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
    let expected = manifest_mean + loading * composed;
    let error = rmse(&[expected], &[observed]);
    assert!(
        error < 1e-15,
        "Driver Eq. 5 of Table 3 T0TIPREDEFFECT RMSE {error}"
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
        "τ + λ μ_t is not T0TIPREDEFFECT E(y_t): RMSE {evolved_error} must exceed {error}"
    );
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
    let process_error = rmse(&[expected], &[process_observed]);
    assert!(
        process_error > error,
        "τ + λ(μ_t + increment) is not T0TIPREDEFFECT E(y_t): RMSE {process_error} must exceed {error}"
    );
}

#[test]
fn discrete_observed_mean_with_initial_time_independent_predictor_is_not_impulse_or_carry() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_initial_time_independent_predictor(
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
    let expected = manifest_mean + loading * composed;
    let error = rmse(&[expected], &[observed]);
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
    let impulse_error = rmse(&[expected], &[impulse_observed]);
    assert!(
        impulse_error > error,
        "τ + λ(μ_t + m x) is not T0TIPREDEFFECT E(y_t): RMSE {impulse_error} must exceed {error}"
    );
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
    let carry_error = rmse(&[expected], &[carried_observed]);
    assert!(
        carry_error > error,
        "τ + λ(μ_t + carry) is not T0TIPREDEFFECT E(y_t): RMSE {carry_error} must exceed {error}"
    );
    let intercept_error = rmse(&[expected], &[manifest_mean]);
    assert!(
        intercept_error > error,
        "MANIFESTMEANS is not T0TIPREDEFFECT E(y_t): RMSE {intercept_error} must exceed {error}"
    );
    let latent_error = rmse(&[expected], &[composed]);
    assert!(
        latent_error > error,
        "evolved-plus-T0TIPRED latent mean is not E(y_t): RMSE {latent_error} must exceed {error}"
    );
}

#[test]
fn discrete_observed_mean_with_initial_time_independent_predictor_refuses_evolved_process_impulse_and_carry()
 {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_initial_time_independent_predictor(
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
    assert_eq!(
        refuse_evolved_observed_mean_as_initial_time_independent_observed_mean(
            evolved_observed,
            observed
        ),
        Err(PsychometricError::EvolvedObservedMeanIsNotInitialTimeIndependentObservedMean)
    );
    assert_eq!(
        refuse_time_independent_observed_mean_as_initial_time_independent_observed_mean(
            process_observed,
            observed
        ),
        Err(PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeIndependentObservedMean)
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_initial_time_independent_observed_mean(
            impulse_observed,
            observed
        ),
        Err(PsychometricError::ImpulseObservedMeanIsNotInitialTimeIndependentObservedMean)
    );
    assert_eq!(
        refuse_impulse_carry_observed_mean_as_initial_time_independent_observed_mean(
            carried_observed,
            observed
        ),
        Err(PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeIndependentObservedMean)
    );
}

#[test]
fn discrete_observed_mean_with_initial_time_independent_predictor_zero_loading_is_manifest_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_initial_time_independent_predictor(
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
    assert_eq!(
        refuse_latent_mean_as_observed_mean(composed, observed),
        Err(PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, observed),
        Err(PsychometricError::ManifestMeansIsNotObservedMean)
    );
    let zero_loading = recover_discrete_observed_mean_with_initial_time_independent_predictor(
        0.0,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("lambda0");
    assert!((zero_loading - manifest_mean).abs() < 1e-15);
}

#[test]
fn discrete_observed_mean_with_initial_time_independent_predictor_refuses_overflow_and_non_event_clocks()
 {
    assert_eq!(
        recover_discrete_observed_mean_with_initial_time_independent_predictor(
            1e308,
            2.0,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_initial_time_independent_predictor(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_initial_time_independent_predictor(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let scaled = recover_discrete_observed_mean_with_initial_time_independent_predictor(
        1e308,
        1e-308,
        0.0,
        0.0,
        0.0,
        3.0,
        0.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("scale");
    assert!(
        (scaled - 1.0).abs() < 1e-15,
        "Driver Eq. 5 of Table 3 T0TIPREDEFFECT must keep λ=1e308, μ=1e-308: got {scaled}"
    );
}

#[test]
fn discrete_observed_mean_with_time_independent_predictor_recovers_driver_equation_five() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_time_independent_predictor(
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
    let expected = manifest_mean + loading * composed;
    let error = rmse(&[expected], &[observed]);
    assert!(
        error < 1e-15,
        "Driver Eq. 5 of Eq. 3 TIPREDEFFECT RMSE {error}"
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
        "τ + λ μ_t is not TIPREDEFFECT E(y_t): RMSE {evolved_error} must exceed {error}"
    );
    let intercept_error = rmse(&[expected], &[manifest_mean]);
    assert!(
        intercept_error > error,
        "MANIFESTMEANS is not TIPREDEFFECT E(y_t): RMSE {intercept_error} must exceed {error}"
    );
    let latent_error = rmse(&[expected], &[composed]);
    assert!(
        latent_error > error,
        "evolved-plus-increment latent mean is not E(y_t): RMSE {latent_error} must exceed {error}"
    );
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
    let impulse_error = rmse(&[expected], &[impulse_observed]);
    assert!(
        impulse_error > error,
        "τ + λ(μ_t + m x) is not TIPREDEFFECT E(y_t): RMSE {impulse_error} must exceed {error}"
    );
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
    let carry_error = rmse(&[expected], &[carried_observed]);
    assert!(
        carry_error > error,
        "τ + λ(μ_t + carry) is not TIPREDEFFECT E(y_t): RMSE {carry_error} must exceed {error}"
    );
}

#[test]
fn discrete_observed_mean_with_time_independent_predictor_refuses_evolved_mean_impulse_and_carry() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_time_independent_predictor(
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
    assert_eq!(
        refuse_evolved_observed_mean_as_time_independent_observed_mean(evolved_observed, observed),
        Err(PsychometricError::EvolvedObservedMeanIsNotTimeIndependentObservedMean)
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_time_independent_observed_mean(impulse_observed, observed),
        Err(PsychometricError::ImpulseObservedMeanIsNotTimeIndependentObservedMean)
    );
    assert_eq!(
        refuse_impulse_carry_observed_mean_as_time_independent_observed_mean(
            carried_observed,
            observed
        ),
        Err(PsychometricError::ImpulseCarryObservedMeanIsNotTimeIndependentObservedMean)
    );
}

#[test]
fn discrete_observed_mean_with_time_independent_predictor_zero_loading_is_manifest_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_time_independent_predictor(
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
    let expected = manifest_mean + loading * composed;
    let error = rmse(&[expected], &[observed]);
    let zero_loading = recover_discrete_observed_mean_with_time_independent_predictor(
        0.0,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("lambda0");
    let dropped_error = rmse(&[expected], &[zero_loading]);
    assert!(
        dropped_error > error,
        "zero loading is τ, not τ + λ(μ_t + increment): RMSE {dropped_error} must exceed {error}"
    );
    assert_eq!(
        refuse_latent_mean_as_observed_mean(composed, observed),
        Err(PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, observed),
        Err(PsychometricError::ManifestMeansIsNotObservedMean)
    );
}

#[test]
fn discrete_observed_mean_with_time_independent_predictor_refuses_overflow_and_non_event_clocks() {
    assert_eq!(
        recover_discrete_observed_mean_with_time_independent_predictor(
            1e308,
            2.0,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_time_independent_predictor(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_time_independent_predictor(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_time_independent_predictor(
            1e308,
            0.0,
            0.0,
            0.0,
            1e308,
            1.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    let scaled = recover_discrete_observed_mean_with_time_independent_predictor(
        1e308,
        1e-308,
        0.0,
        0.0,
        0.0,
        3.0,
        0.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("scale");
    assert!(
        (scaled - 1.0).abs() < 1e-15,
        "Driver Eq. 5 of Eq. 3 TIPREDEFFECT must keep λ=1e308, μ=1e-308: got {scaled}"
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
    assert_eq!(
        recover_time_dependent_predictor_impulse_carry(
            0.4,
            3.0,
            1e308,
            3.0,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_time_dependent_predictor_impulse_carry(
            0.4,
            3.0,
            710.0,
            2.0,
            1.0,
            LagClock::EventTime,
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
        refuse_impulse_observed_mean_as_impulse_carry_observed_mean(contemporaneous, observed),
        Err(PsychometricError::ImpulseObservedMeanIsNotImpulseCarryObservedMean)
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

#[test]
fn discrete_observed_mean_with_initial_time_dependent_predictor_recovers_driver_equation_five() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
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
    let expected = manifest_mean + loading * composed;
    let error = rmse(&[expected], &[observed]);
    assert!(
        error < 1e-15,
        "Driver Eq. 5 of Table 3 T0TDPREDEFFECT RMSE {error}"
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
        "τ + λ μ_t is not T0TDPREDEFFECT E(y_t): RMSE {evolved_error} must exceed {error}"
    );
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
    let process_error = rmse(&[expected], &[process_observed]);
    assert!(
        process_error > error,
        "τ + λ(μ_t + increment) is not T0TDPREDEFFECT E(y_t): RMSE {process_error} must exceed {error}"
    );
}

#[test]
fn discrete_observed_mean_with_initial_time_dependent_predictor_is_not_impulse_or_carry() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
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
    let expected = manifest_mean + loading * composed;
    let error = rmse(&[expected], &[observed]);
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
    let impulse_error = rmse(&[expected], &[impulse_observed]);
    assert!(
        impulse_error > error,
        "τ + λ(μ_t + m x) is not T0TDPREDEFFECT E(y_t): RMSE {impulse_error} must exceed {error}"
    );
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
    let carry_error = rmse(&[expected], &[carried_observed]);
    assert!(
        carry_error > error,
        "τ + λ(μ_t + carry) is not T0TDPREDEFFECT E(y_t): RMSE {carry_error} must exceed {error}"
    );
    let intercept_error = rmse(&[expected], &[manifest_mean]);
    assert!(
        intercept_error > error,
        "MANIFESTMEANS is not T0TDPREDEFFECT E(y_t): RMSE {intercept_error} must exceed {error}"
    );
    let latent_error = rmse(&[expected], &[composed]);
    assert!(
        latent_error > error,
        "evolved-plus-T0TDPRED latent mean is not E(y_t): RMSE {latent_error} must exceed {error}"
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn discrete_observed_mean_with_initial_time_dependent_predictor_refuses_evolved_process_impulse_and_carry()
 {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
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
    assert_eq!(
        refuse_evolved_observed_mean_as_initial_time_dependent_observed_mean(
            evolved_observed,
            observed
        ),
        Err(PsychometricError::EvolvedObservedMeanIsNotInitialTimeDependentObservedMean)
    );
    assert_eq!(
        refuse_time_independent_observed_mean_as_initial_time_dependent_observed_mean(
            process_observed,
            observed
        ),
        Err(PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean)
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_initial_time_dependent_observed_mean(
            impulse_observed,
            observed
        ),
        Err(PsychometricError::ImpulseObservedMeanIsNotInitialTimeDependentObservedMean)
    );
    assert_eq!(
        refuse_impulse_carry_observed_mean_as_initial_time_dependent_observed_mean(
            carried_observed,
            observed
        ),
        Err(PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeDependentObservedMean)
    );
    assert_eq!(
        refuse_initial_time_independent_observed_mean_as_initial_time_dependent_observed_mean(
            tipred_observed,
            observed
        ),
        Err(PsychometricError::InitialTimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean)
    );
}

#[test]
fn discrete_observed_mean_with_initial_time_dependent_predictor_zero_loading_is_manifest_mean() {
    let loading = 2.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let observed = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
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
    assert_eq!(
        refuse_latent_mean_as_observed_mean(composed, observed),
        Err(PsychometricError::LatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_manifest_means_as_observed_mean(manifest_mean, observed),
        Err(PsychometricError::ManifestMeansIsNotObservedMean)
    );
    let zero_loading = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
        0.0,
        initial,
        drift,
        intercept,
        effect,
        predictor,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("lambda0");
    assert!((zero_loading - manifest_mean).abs() < 1e-15);
}

#[test]
fn discrete_observed_mean_with_initial_time_dependent_predictor_refuses_overflow_and_non_event_clocks()
 {
    assert_eq!(
        recover_discrete_observed_mean_with_initial_time_dependent_predictor(
            1e308,
            2.0,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_initial_time_dependent_predictor(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_initial_time_dependent_predictor(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let scaled = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
        1e308,
        1e-308,
        0.0,
        0.0,
        0.0,
        3.0,
        0.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("scale");
    assert!(
        (scaled - 1.0).abs() < 1e-15,
        "Driver Eq. 5 of Table 3 T0TDPREDEFFECT must keep λ=1e308, μ=1e-308: got {scaled}"
    );
}

#[test]
fn level_change_continuous_intercept_recovers_driver_section_seven_point_two() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let intercept =
        recover_level_change_continuous_intercept(effect, predictor, drift).expect("level-change");
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("impulse");
    let error = rmse(&[0.6], &[intercept]);
    assert!(
        error < 1e-15,
        "Driver §7.2 level-change CINT RMSE {error}: got {intercept}"
    );
    let equilibrium_error = rmse(&[impulse], &[intercept / (-drift)]);
    assert!(
        equilibrium_error < 1e-15,
        "Driver §7.2 −κ/a must recover m x: RMSE {equilibrium_error}"
    );
    assert!(rmse(&[intercept], &[impulse]) > error);
    let increment = recover_discrete_time_independent_predictor_effect(
        effect,
        predictor,
        drift,
        2.0,
        LagClock::EventTime,
    )
    .expect("tipred");
    assert_eq!(
        refuse_level_change_intercept_as_impulse(intercept, impulse),
        Err(PsychometricError::LevelChangeInterceptIsNotImpulse)
    );
    assert_eq!(
        refuse_level_change_intercept_as_free_continuous_intercept(intercept, 0.3),
        Err(PsychometricError::LevelChangeInterceptIsNotFreeContinuousIntercept)
    );
    assert_eq!(
        refuse_level_change_intercept_as_process_increment(intercept, increment),
        Err(PsychometricError::LevelChangeInterceptIsNotProcessIncrement)
    );
}

#[test]
fn level_change_continuous_intercept_refuses_unstable_drift_and_overflow() {
    assert_eq!(
        recover_level_change_continuous_intercept(0.4, 3.0, 0.0),
        Err(PsychometricError::LevelChangeRequiresStableDrift)
    );
    assert_eq!(
        recover_level_change_continuous_intercept(0.4, 3.0, 0.5),
        Err(PsychometricError::LevelChangeRequiresStableDrift)
    );
    assert_eq!(
        recover_level_change_continuous_intercept(1e308, 2.0, -0.5),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_level_change_continuous_intercept(1.0, 2.0, -1e308),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_level_change_continuous_intercept(0.0, 3.0, 0.0),
        Ok(0.0)
    );
}

#[test]
fn level_change_discrete_increment_recovers_driver_equation_three_of_section_seven_point_two() {
    let effect = 0.4_f64;
    let predictor = 3.0_f64;
    let drift = -0.5_f64;
    let delta = 2.0_f64;
    let increment = recover_level_change_discrete_increment(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("level-change-increment");
    let intercept =
        recover_level_change_continuous_intercept(effect, predictor, drift).expect("level-change");
    let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("impulse");
    let expected = (1.0 - (drift * delta).exp()) * impulse;
    let error = rmse(&[expected], &[increment]);
    assert!(
        error < 1e-15,
        "Driver §7.2 Eq. 3 level-change increment RMSE {error}: got {increment}"
    );
    assert!(rmse(&[increment], &[impulse]) > error);
    assert!(rmse(&[increment], &[intercept]) > error);
    let tipred = recover_discrete_time_independent_predictor_effect(
        effect,
        predictor,
        drift,
        delta,
        LagClock::EventTime,
    )
    .expect("tipred");
    assert_eq!(
        refuse_level_change_increment_as_impulse(increment, impulse),
        Err(PsychometricError::LevelChangeIncrementIsNotImpulse)
    );
    assert_eq!(
        refuse_level_change_increment_as_intercept(increment, intercept),
        Err(PsychometricError::LevelChangeIncrementIsNotIntercept)
    );
    assert_eq!(
        refuse_level_change_increment_as_process_increment(increment, tipred),
        Err(PsychometricError::LevelChangeIncrementIsNotProcessIncrement)
    );
    let equilibrated = recover_level_change_discrete_increment(
        effect,
        predictor,
        -800.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("underflow");
    assert!(
        rmse(&[impulse], &[equilibrated]) < 1e-15,
        "underflow of e^{{aΔt}} must keep m x: got {equilibrated}"
    );
}

#[test]
fn level_change_discrete_increment_refuses_unstable_drift_clock_and_overflow() {
    assert_eq!(
        recover_level_change_discrete_increment(0.4, 3.0, 0.0, 2.0, LagClock::EventTime),
        Err(PsychometricError::LevelChangeRequiresStableDrift)
    );
    assert_eq!(
        recover_level_change_discrete_increment(0.4, 3.0, 0.5, 2.0, LagClock::EventTime),
        Err(PsychometricError::LevelChangeRequiresStableDrift)
    );
    assert_eq!(
        recover_level_change_discrete_increment(0.4, 3.0, -0.5, 2.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_level_change_discrete_increment(0.4, 3.0, -0.5, 0.0, LagClock::EventTime),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_level_change_discrete_increment(1e308, 2.0, -0.5, 2.0, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_level_change_discrete_increment(0.0, 3.0, 0.0, 2.0, LagClock::EventTime),
        Ok(0.0)
    );
}

#[test]
fn extra_process_contribution_recovers_driver_section_seven_point_two() {
    let coupling = 0.569_907_f64;
    let predictor = 1.0_f64;
    let original = -0.1393_f64;
    let extra = -0.000_001_f64;
    let delta = 1.0_f64;
    let recovered = recover_level_change_extra_process_contribution(
        coupling,
        predictor,
        original,
        extra,
        delta,
        LagClock::EventTime,
    )
    .expect("extra-process");
    let expected = coupling * predictor * ((extra * delta).exp() - (original * delta).exp())
        / (extra - original);
    assert!(
        rmse(&[expected], &[recovered]) < 1e-15,
        "Driver et al. (2017, §7.2 pp. 22–23) extra-process map: expected {expected}, got {recovered}"
    );
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
    let distinction = recover_level_change_extra_process_contribution(
        0.4,
        3.0,
        -0.5,
        -0.05,
        2.0,
        LagClock::EventTime,
    )
    .expect("distinction");
    let distinct_intercept =
        recover_level_change_continuous_intercept(0.4, 3.0, -0.5).expect("distinct-cint");
    let distinct_increment =
        recover_level_change_discrete_increment(0.4, 3.0, -0.5, 2.0, LagClock::EventTime)
            .expect("distinct-increment");
    let distinct_impulse = recover_time_dependent_predictor_impulse(0.4, 3.0).expect("dirac");
    assert!(rmse(&[distinct_intercept], &[distinction]) > 1e-3);
    assert!(rmse(&[distinct_increment], &[distinction]) > 1e-3);
    assert!(rmse(&[distinct_impulse], &[distinction]) > 1e-3);
    assert_eq!(
        refuse_level_change_extra_process_as_impulse(recovered, impulse),
        Err(PsychometricError::LevelChangeExtraProcessIsNotImpulse)
    );
    assert_eq!(
        refuse_level_change_extra_process_as_intercept(recovered, intercept),
        Err(PsychometricError::LevelChangeExtraProcessIsNotIntercept)
    );
    assert_eq!(
        refuse_level_change_extra_process_as_increment(recovered, increment),
        Err(PsychometricError::LevelChangeExtraProcessIsNotIncrement)
    );
}

#[test]
fn extra_process_contribution_refuses_nonnegative_extra_drift_clock_and_overflow() {
    assert_eq!(
        recover_level_change_extra_process_contribution(
            0.4,
            3.0,
            -0.5,
            0.0,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::LevelChangeExtraProcessRequiresNegativeDrift)
    );
    assert_eq!(
        recover_level_change_extra_process_contribution(
            0.4,
            3.0,
            -0.5,
            0.5,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::LevelChangeExtraProcessRequiresNegativeDrift)
    );
    assert_eq!(
        recover_level_change_extra_process_contribution(
            0.4,
            3.0,
            -0.5,
            -0.000_001,
            2.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_level_change_extra_process_contribution(
            0.4,
            3.0,
            -0.5,
            -0.000_001,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_level_change_extra_process_contribution(
            1e308,
            2.0,
            -0.5,
            -0.000_001,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_level_change_extra_process_contribution(
            0.0,
            3.0,
            -0.5,
            0.0,
            2.0,
            LagClock::EventTime
        ),
        Ok(0.0)
    );
    let overflow_fallback = recover_level_change_extra_process_contribution(
    let finite_exponential_fallback = recover_level_change_extra_process_contribution(
        0.4,
        3.0,
        -0.8,
        -0.000_001,
        900.0,
        LagClock::EventTime,
    )
    .expect("expm1-overflow-fallback");
    assert!(overflow_fallback.is_finite());
    .expect("finite exponential fallback");
    assert!(finite_exponential_fallback.is_finite());
}

#[test]
fn extra_process_observed_mean_recovers_driver_equation_five_of_section_seven_point_two() {
    let loading = 1.0_f64;
    let coupling = 0.569_907_f64;
    let predictor = 1.0_f64;
    let original = -0.1393_f64;
    let extra = -0.000_001_f64;
    let delta = 1.0_f64;
    let initial = 0.0_f64;
    let intercept = 0.0_f64;
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
    let expected = manifest_mean + loading * composed;
    let error = rmse(&[expected], &[observed]);
    assert!(
        error < 1e-15,
        "Driver Eq. 5 of §7.2 extra-process contribution RMSE {error}: got {observed}"
    );
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
    assert!(
        rmse(&[expected], &[evolved_observed]) > error,
        "τ + λ μ_t is not extra-process E(y_t)"
    );
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
        rmse(&[expected], &[impulse_observed]) > error,
        "τ + λ(μ_t + m x) is not extra-process E(y_t)"
    );
    assert!(rmse(&[expected], &[manifest_mean]) > error);
    assert!(rmse(&[expected], &[composed]) > error);
    assert!(rmse(&[expected], &[contribution]) > error);
    assert_eq!(
        refuse_evolved_observed_mean_as_extra_process_observed_mean(evolved_observed, observed),
        Err(PsychometricError::EvolvedObservedMeanIsNotExtraProcessObservedMean)
    );
    assert_eq!(
        refuse_impulse_observed_mean_as_extra_process_observed_mean(impulse_observed, observed),
        Err(PsychometricError::ImpulseObservedMeanIsNotExtraProcessObservedMean)
    );
    assert_eq!(
        refuse_extra_process_contribution_as_observed_mean(contribution, observed),
        Err(PsychometricError::ExtraProcessContributionIsNotObservedMean)
    );
    assert_eq!(
        refuse_extra_process_latent_mean_as_observed_mean(composed, observed),
        Err(PsychometricError::ExtraProcessLatentMeanIsNotObservedMean)
    );
}

#[test]
fn extra_process_observed_mean_zero_loading_is_manifest_mean() {
    let coupling = 0.4_f64;
    let predictor = 3.0_f64;
    let original = -0.5_f64;
    let extra = -0.05_f64;
    let delta = 2.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    let zero_loading = recover_discrete_observed_mean_with_extra_process(
        0.0,
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
    .expect("zero-loading");
    assert!(
        rmse(&[manifest_mean], &[zero_loading]) < 1e-15,
        "zero original-indicator loading is τ: got {zero_loading}"
    );
    let extra_loading_zero = recover_manifest_observed_mean(0.0, 12.0, manifest_mean)
        .expect("extra-process-lambda-zero");
    let original_observed = recover_discrete_observed_mean_with_extra_process(
        2.0,
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
    .expect("original-indicator");
    assert!(
        rmse(&[extra_loading_zero], &[original_observed]) > 1e-3,
        "printed extra-process LAMBDA 0 is τ, not original-indicator E(y_t)"
    );
    let zero_coupling = recover_discrete_observed_mean_with_extra_process(
        2.0,
        initial,
        original,
        intercept,
        0.0,
        predictor,
        extra,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("zero-coupling");
    let evolved_observed = recover_discrete_observed_mean(
        2.0,
        initial,
        original,
        intercept,
        manifest_mean,
        delta,
        LagClock::EventTime,
    )
    .expect("eq3-eq5-mean");
    assert!(rmse(&[evolved_observed], &[zero_coupling]) < 1e-15);
}

#[test]
fn extra_process_observed_mean_refuses_clock_nonpositive_interval_and_nonnegative_drift() {
    let coupling = 0.4_f64;
    let predictor = 3.0_f64;
    let original = -0.5_f64;
    let extra = -0.05_f64;
    let delta = 2.0_f64;
    let initial = 1.0_f64;
    let intercept = 0.3_f64;
    let manifest_mean = 0.5_f64;
    assert_eq!(
        recover_discrete_observed_mean_with_extra_process(
            2.0,
            initial,
            original,
            intercept,
            coupling,
            predictor,
            extra,
            manifest_mean,
            delta,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_extra_process(
            2.0,
            initial,
            original,
            intercept,
            coupling,
            predictor,
            0.0,
            manifest_mean,
            delta,
            LagClock::EventTime
        ),
        Err(PsychometricError::LevelChangeExtraProcessRequiresNegativeDrift)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_extra_process(
            2.0,
            initial,
            original,
            intercept,
            coupling,
            predictor,
            extra,
            manifest_mean,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_latent_mean_with_extra_process(
            1e308,
            0.0,
            0.0,
            1e308,
            1.0,
            extra,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn after_extra_process_observed_mean_recovers_driver_equation_five_after_t0() {
    let loading = 1.0_f64;
    let coupling = 1.0_f64;
    let predictor = 1.0_f64;
    let original = -0.4_f64;
    let extra = -0.000_001_f64;
    let delta = 2.0_f64;
    let elapsed = 1.0_f64;
    let initial = 0.0_f64;
    let intercept = 0.0_f64;
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
    let expected = manifest_mean + loading * composed;
    let error = rmse(&[expected], &[observed]);
    assert!(
        error < 1e-15,
        "Driver Eq. 5 of §7.2 after-t0 extra-process RMSE {error}: got {observed}"
    );
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
    assert!(
        rmse(&[expected], &[first_occasion]) > error,
        "T0TDPREDEFFECT extra-process E(y_t) is not after-t0 E(y_t)"
    );
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
    assert!(rmse(&[expected], &[evolved_observed]) > error);
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
        rmse(&[expected], &[carry_observed]) > error,
        "e^{{a(t-u)}} m x is not extra-process DRIFT drive"
    );
    assert!(rmse(&[expected], &[manifest_mean]) > error);
    assert!(rmse(&[expected], &[composed]) > error);
    assert!(rmse(&[expected], &[contribution]) > error);
    assert_eq!(
        refuse_extra_process_observed_mean_as_after_extra_process_observed_mean(
            first_occasion,
            observed
        ),
        Err(PsychometricError::ExtraProcessObservedMeanIsNotAfterExtraProcessObservedMean)
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_after_extra_process_observed_mean(
            evolved_observed,
            observed
        ),
        Err(PsychometricError::EvolvedObservedMeanIsNotAfterExtraProcessObservedMean)
    );
    assert_eq!(
        refuse_impulse_carry_observed_mean_as_after_extra_process_observed_mean(
            carry_observed,
            observed
        ),
        Err(PsychometricError::ImpulseCarryObservedMeanIsNotAfterExtraProcessObservedMean)
    );
    assert_eq!(
        refuse_after_extra_process_contribution_as_observed_mean(contribution, observed),
        Err(PsychometricError::AfterExtraProcessContributionIsNotObservedMean)
    );
    assert_eq!(
        refuse_after_extra_process_latent_mean_as_observed_mean(composed, observed),
        Err(PsychometricError::AfterExtraProcessLatentMeanIsNotObservedMean)
    );
}

#[test]
fn after_extra_process_observed_mean_refuses_non_interior_interval_and_clock() {
    let coupling = 1.0_f64;
    let predictor = 1.0_f64;
    let original = -0.4_f64;
    let extra = -0.000_001_f64;
    assert_eq!(
        recover_level_change_extra_process_contribution_after(
            coupling,
            predictor,
            original,
            extra,
            2.0,
            2.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_extra_process_after(
            1.0,
            0.0,
            original,
            0.0,
            coupling,
            predictor,
            extra,
            0.5,
            2.0,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_discrete_observed_mean_with_extra_process_after(
            0.0,
            0.0,
            original,
            0.0,
            coupling,
            predictor,
            extra,
            0.5,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Ok(0.5)
    );
}

#[test]
fn asymptotic_time_independent_effect_recovers_driver_section_seven_point_two() {
    // Driver et al. (2017, §7.2, p. 21) print LeisureTime
    // TIPREDEFFECT = −0.225 and asymTIPREDEFFECT = −1.673 for a unit
    // increase. Reconstruct a = −B / asym.
    let effect = -0.225_f64;
    let predictor = 1.0_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -effect / printed_asym;
    let recovered = recover_asymptotic_time_independent_predictor_effect(
        effect,
        predictor,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECT");
    let expected = -(effect * predictor) / log_rate;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-15,
        "Driver §7.2 asymTIPREDEFFECT RMSE {error}: got {recovered}"
    );
    assert!(
        rmse(&[printed_asym], &[recovered]) < 1e-12,
        "printed LeisureTime asymTIPREDEFFECT"
    );
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
        rmse(&[recovered], &[effect]) > error,
        "TIPREDEFFECT B is not asymTIPREDEFFECT"
    );
    assert!(
        rmse(&[recovered], &[discrete]) > error,
        "A^{{-1}}[e^{{A Δt}} − I] B z is not -B z / a"
    );
    assert!(rmse(&[recovered], &[impulse]) > error);
    let happiness = recover_asymptotic_time_independent_predictor_effect(
        0.549,
        1.0,
        -0.549 / 0.219,
        LagClock::EventTime,
    )
    .expect("happiness-asym");
    assert!(rmse(&[0.219], &[happiness]) < 1e-12);
    assert_eq!(
        refuse_asymptotic_time_independent_effect_as_coefficient(recovered, effect),
        Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotCoefficient)
    );
    assert_eq!(
        refuse_asymptotic_time_independent_effect_as_discrete_effect(recovered, discrete),
        Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotDiscreteEffect)
    );
    assert_eq!(
        refuse_asymptotic_time_independent_effect_as_continuous_intercept(recovered, 0.3),
        Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotContinuousIntercept)
    );
    assert_eq!(
        refuse_asymptotic_time_independent_effect_as_time_dependent_impulse(recovered, impulse),
        Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotTimeDependentImpulse)
    );
}

#[test]
fn asymptotic_time_independent_effect_refuses_unstable_drift_and_non_event_clocks() {
    let effect = -0.225_f64;
    let predictor = 1.0_f64;
    let log_rate = -0.134_488_942_f64;
    assert_eq!(
        recover_asymptotic_time_independent_predictor_effect(
            effect,
            predictor,
            log_rate,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_asymptotic_time_independent_predictor_effect(
            effect,
            predictor,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_asymptotic_time_independent_predictor_effect(
            effect,
            predictor,
            0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_asymptotic_time_independent_predictor_effect(
            1e308,
            2.0,
            log_rate,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_asymptotic_time_independent_predictor_effect(
            0.0,
            predictor,
            0.0,
            LagClock::EventTime
        ),
        Ok(0.0)
    );
}

#[test]
fn asymptotic_time_independent_variance_recovers_driver_section_seven_point_two() {
    let effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -effect / printed_asym;
    let predictor_variance = 1.0_f64;
    let recovered = recover_asymptotic_time_independent_predictor_variance(
        effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let expected = printed_asym * printed_asym * predictor_variance;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §7.2 addedTIPREDVAR RMSE {error}: got {recovered}"
    );
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
        rmse(&[recovered], &[mean_effect]) > error,
        "asymTIPREDEFFECT is not addedTIPREDVAR"
    );
    assert!(rmse(&[recovered], &[stationary]) > error);
    assert!(rmse(&[recovered], &[trait_plus]) > error);
    assert_eq!(
        refuse_asymptotic_time_independent_variance_as_trait_variance(recovered, trait_plus),
        Err(PsychometricError::AsymptoticTimeIndependentVarianceIsNotTraitVariance)
    );
    assert_eq!(
        refuse_asymptotic_time_independent_variance_as_stationary_within_subject(
            recovered, stationary
        ),
        Err(PsychometricError::AsymptoticTimeIndependentVarianceIsNotStationaryWithinSubject)
    );
    assert_eq!(
        refuse_asymptotic_time_independent_variance_as_asymptotic_effect(recovered, mean_effect),
        Err(PsychometricError::AsymptoticTimeIndependentVarianceIsNotAsymptoticEffect)
    );
}

#[test]
fn asymptotic_time_independent_variance_refuses_unstable_drift_and_non_event_clocks() {
    let effect = -0.225_f64;
    let log_rate = -0.134_488_942_f64;
    assert_eq!(
        recover_asymptotic_time_independent_predictor_variance(
            effect,
            1.0,
            log_rate,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_asymptotic_time_independent_predictor_variance(
            effect,
            1.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_asymptotic_time_independent_predictor_variance(
            effect,
            -1.0,
            log_rate,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_asymptotic_time_independent_predictor_variance(0.0, 1.0, 0.0, LagClock::EventTime),
        Ok(0.0)
    );
    assert_eq!(
        recover_asymptotic_time_independent_predictor_variance(
            1.0,
            1.0,
            -1e-308,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn asymptotic_continuous_intercept_recovers_driver_table_two() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let intercept = 0.3_f64;
    let recovered =
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
            .expect("asymCINT");
    let expected = intercept / -log_rate;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver Table 2 asymCINT RMSE {error}: got {recovered}"
    );
    let discrete =
        recover_discrete_continuous_intercept_effect(intercept, log_rate, 1.0, LagClock::EventTime)
            .expect("dtCINT");
    let tipred = recover_asymptotic_time_independent_predictor_effect(
        printed_effect,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECT");
    assert!(
        rmse(&[recovered], &[intercept]) > error,
        "CINT is not asymCINT"
    );
    assert!(rmse(&[recovered], &[discrete]) > error);
    assert!(rmse(&[recovered], &[2.823]) > error);
    assert!(rmse(&[recovered], &[tipred]) > error);
    assert_eq!(
        refuse_asymptotic_continuous_intercept_as_continuous_intercept(recovered, intercept),
        Err(PsychometricError::AsymptoticContinuousInterceptIsNotContinuousIntercept)
    );
    assert_eq!(
        refuse_asymptotic_continuous_intercept_as_discrete_increment(recovered, discrete),
        Err(PsychometricError::AsymptoticContinuousInterceptIsNotDiscreteIncrement)
    );
    assert_eq!(
        refuse_asymptotic_continuous_intercept_as_initial_latent_mean(recovered, 2.823),
        Err(PsychometricError::AsymptoticContinuousInterceptIsNotInitialLatentMean)
    );
    assert_eq!(
        refuse_asymptotic_continuous_intercept_as_asymptotic_time_independent_effect(
            recovered, tipred
        ),
        Err(PsychometricError::AsymptoticContinuousInterceptIsNotAsymptoticTimeIndependentEffect)
    );
}

#[test]
fn asymptotic_continuous_intercept_refuses_unstable_drift_and_non_event_clocks() {
    let intercept = 0.3_f64;
    let log_rate = -0.134_488_942_f64;
    assert_eq!(
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_asymptotic_continuous_intercept(intercept, 0.0, LagClock::EventTime),
        Err(PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift)
    );
    assert_eq!(
        recover_asymptotic_continuous_intercept(intercept, 0.5, LagClock::EventTime),
        Err(PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift)
    );
    assert_eq!(
        recover_asymptotic_continuous_intercept(f64::NAN, log_rate, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_asymptotic_continuous_intercept(0.0, 0.0, LagClock::EventTime),
        Ok(0.0)
    );
}

#[test]
fn stationary_initial_latent_mean_recovers_driver_page_sixteen() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let intercept = 0.3_f64;
    let recovered = recover_stationary_initial_latent_mean(
        intercept,
        printed_effect,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0MEANS");
    let intercept_only =
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
            .expect("asymCINT");
    let tipred = recover_asymptotic_time_independent_predictor_effect(
        printed_effect,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECT");
    let expected = intercept_only + tipred;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver p. 16 stationary T0MEANS RMSE {error}: got {recovered}"
    );
    let discrete =
        recover_discrete_latent_mean(2.823, log_rate, intercept, 1.0, LagClock::EventTime)
            .expect("μ_t");
    assert!(
        rmse(&[recovered], &[2.823]) > error,
        "T0MEANS is not stationary T0MEANS"
    );
    assert!(rmse(&[recovered], &[intercept_only]) > error);
    assert!(rmse(&[recovered], &[tipred]) > error);
    assert!(rmse(&[recovered], &[discrete]) > error);
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_initial_latent_mean(recovered, 2.823),
        Err(PsychometricError::StationaryInitialLatentMeanIsNotInitialLatentMean)
    );
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_asymptotic_continuous_intercept(
            recovered,
            intercept_only
        ),
        Err(PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticContinuousIntercept)
    );
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_asymptotic_time_independent_effect(
            recovered, tipred
        ),
        Err(PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticTimeIndependentEffect)
    );
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_discrete_mean(recovered, discrete),
        Err(PsychometricError::StationaryInitialLatentMeanIsNotDiscreteMean)
    );
}

#[test]
fn stationary_initial_latent_mean_refuses_unstable_drift_and_non_event_clocks() {
    assert_eq!(
        recover_stationary_initial_latent_mean(0.3, -0.225, 1.0, -0.13, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_stationary_initial_latent_mean(0.3, 0.0, 1.0, 0.0, LagClock::EventTime),
        Err(PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_initial_latent_mean(0.0, -0.225, 1.0, 0.5, LagClock::EventTime),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_initial_latent_mean(0.0, 0.0, 1.0, 0.0, LagClock::EventTime),
        Ok(0.0)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn stationary_initial_observed_mean_recovers_driver_equation_five_of_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let intercept = 0.3_f64;
    let loading = 2.0_f64;
    let manifest_mean = 0.5_f64;
    let recovered = recover_stationary_initial_observed_mean(
        loading,
        intercept,
        printed_effect,
        1.0,
        log_rate,
        manifest_mean,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-T0MEANS");
    let latent = recover_stationary_initial_latent_mean(
        intercept,
        printed_effect,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0MEANS");
    let expected = recover_manifest_observed_mean(loading, latent, manifest_mean).expect("τ+λμ");
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 Eq. 5 of stationary T0MEANS RMSE {error}: got {recovered}"
    );
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
        rmse(&[recovered], &[manifest_mean]) > error,
        "MANIFESTMEANS is not E(y_0)"
    );
    assert!(rmse(&[recovered], &[latent]) > error);
    assert!(rmse(&[recovered], &[intercept_only_observed]) > error);
    assert!(rmse(&[recovered], &[free_initial_observed]) > error);
    assert!(rmse(&[recovered], &[evolved]) > error);
    assert_eq!(
        recover_stationary_initial_observed_mean(
            0.0,
            intercept,
            printed_effect,
            1.0,
            log_rate,
            manifest_mean,
            LagClock::EventTime,
        ),
        Ok(manifest_mean)
    );
    let evolved_from_stationary = recover_discrete_observed_mean_with_time_independent_predictor(
        loading,
        latent,
        log_rate,
        intercept,
        printed_effect,
        1.0,
        manifest_mean,
        2.0,
        LagClock::EventTime,
    )
    .expect("invariance");
    assert!(rmse(&[recovered], &[evolved_from_stationary]) < 1e-12);
    assert_eq!(
        refuse_stationary_initial_latent_mean_as_observed_mean(latent, recovered),
        Err(PsychometricError::StationaryInitialLatentMeanIsNotObservedMean)
    );
    assert_eq!(
        refuse_stationary_initial_observed_mean_as_manifest_means(recovered, manifest_mean),
        Err(PsychometricError::StationaryInitialObservedMeanIsNotManifestMeans)
    );
    assert_eq!(
        refuse_evolved_observed_mean_as_stationary_initial_observed_mean(evolved, recovered),
        Err(PsychometricError::EvolvedObservedMeanIsNotStationaryInitialObservedMean)
    );
    assert_eq!(
        refuse_asymptotic_continuous_intercept_observed_mean_as_stationary_initial_observed_mean(
            intercept_only_observed,
            recovered
        ),
        Err(
            PsychometricError::AsymptoticContinuousInterceptObservedMeanIsNotStationaryInitialObservedMean
        )
    );
    assert_eq!(
        refuse_initial_observed_mean_as_stationary_initial_observed_mean(
            free_initial_observed,
            recovered
        ),
        Err(PsychometricError::InitialObservedMeanIsNotStationaryInitialObservedMean)
    );
}

#[test]
fn stationary_initial_observed_mean_refuses_unstable_drift_and_non_event_clocks() {
    assert_eq!(
        recover_stationary_initial_observed_mean(
            2.0,
            0.3,
            -0.225,
            1.0,
            -0.13,
            0.5,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_stationary_initial_observed_mean(2.0, 0.3, 0.0, 1.0, 0.0, 0.5, LagClock::EventTime),
        Err(PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_initial_observed_mean(
            2.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_initial_observed_mean(2.0, 0.0, 0.0, 1.0, 0.0, 0.5, LagClock::EventTime),
        Ok(0.5)
    );
}

#[test]
fn stationary_initial_latent_variance_recovers_driver_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let predictor_variance = 1.0_f64;
    let recovered = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0VAR");
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let trait_plus_state =
        recover_trait_plus_state_latent_variance(trait_variance, state).expect("trait+state");
    let added = recover_asymptotic_time_independent_predictor_variance(
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let expected = trait_plus_state + added;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 stationary T0VAR RMSE {error}: got {recovered}"
    );
    let discrete =
        recover_discrete_latent_variance(recovered, diffusion, log_rate, 1.0, LagClock::EventTime)
            .expect("Var(η_t)");
    let free_t0 = 2.0_f64;
    assert!(rmse(&[recovered], &[free_t0]) > error);
    assert!(rmse(&[recovered], &[state]) > error);
    assert!(rmse(&[recovered], &[trait_variance]) > error);
    assert!(rmse(&[recovered], &[added]) > error);
    assert!(rmse(&[recovered], &[discrete]) > error);
    assert!(rmse(&[recovered], &[2.838]) > error);
    assert_eq!(
        recover_stationary_initial_latent_variance(
            0.0,
            0.0,
            0.0,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        ),
        Ok(0.0)
    );
    assert_eq!(
        recover_stationary_initial_latent_variance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            LagClock::EventTime,
        ),
        Ok(trait_variance)
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_initial_latent_variance(recovered, free_t0),
        Err(PsychometricError::StationaryInitialLatentVarianceIsNotInitialLatentVariance)
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_stationary_within_subject(recovered, state),
        Err(PsychometricError::StationaryInitialLatentVarianceIsNotStationaryWithinSubject)
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_trait_variance(recovered, trait_variance),
        Err(PsychometricError::StationaryInitialLatentVarianceIsNotTraitVariance)
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_asymptotic_time_independent_variance(
            recovered, added
        ),
        Err(
            PsychometricError::StationaryInitialLatentVarianceIsNotAsymptoticTimeIndependentVariance
        )
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_discrete_variance(recovered, discrete),
        Err(PsychometricError::StationaryInitialLatentVarianceIsNotDiscreteVariance)
    );
}

#[test]
fn stationary_initial_latent_variance_refuses_unstable_drift_and_non_event_clocks() {
    assert_eq!(
        recover_stationary_initial_latent_variance(
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_stationary_initial_latent_variance(0.0, 0.4, 0.0, 1.0, 0.0, LagClock::EventTime),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_initial_latent_variance(0.0, 0.0, -0.225, 1.0, 0.5, LagClock::EventTime),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_initial_latent_variance(0.0, 0.0, 0.0, 1.0, 0.0, LagClock::EventTime),
        Ok(0.0)
    );
    assert_eq!(
        recover_stationary_initial_latent_variance(
            f64::NAN,
            0.4,
            0.0,
            0.0,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_stationary_initial_latent_variance(
            f64::MAX,
            f64::MAX,
            0.0,
            0.0,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_stationary_initial_latent_variance(
            f64::MAX,
            0.0,
            1.0,
            f64::MAX,
            -1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn stationary_initial_observed_variance_recovers_driver_equation_five_of_section_four_point_three()
{
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let manifest_trait = 0.1_f64;
    let recovered = recover_stationary_initial_observed_variance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-T0VAR");
    let latent = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0VAR");
    let expected = recover_manifest_trait_plus_state_observed_variance(
        loading,
        latent,
        measurement_error,
        manifest_trait,
    )
    .expect("λ²p+θ+ψ");
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 Eq. 5 of stationary T0VAR RMSE {error}: got {recovered}"
    );
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
        rmse(&[recovered], &[measurement_error]) > error,
        "MANIFESTVAR is not Var(y_0)"
    );
    assert!(rmse(&[recovered], &[latent]) > error);
    assert!(rmse(&[recovered], &[state_only_observed]) > error);
    assert!(rmse(&[recovered], &[free_initial_observed]) > error);
    assert!(rmse(&[recovered], &[evolved]) > error);
    assert_eq!(
        recover_stationary_initial_observed_variance(
            0.0,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        ),
        Ok(measurement_error + manifest_trait)
    );
    assert_eq!(
        refuse_stationary_initial_latent_variance_as_observed_variance(latent, recovered),
        Err(PsychometricError::StationaryInitialLatentVarianceIsNotObservedVariance)
    );
    assert_eq!(
        refuse_stationary_initial_observed_variance_as_measurement_error(
            recovered,
            measurement_error
        ),
        Err(PsychometricError::StationaryInitialObservedVarianceIsNotMeasurementError)
    );
    assert_eq!(
        refuse_evolved_observed_variance_as_stationary_initial_observed_variance(
            evolved, recovered
        ),
        Err(PsychometricError::EvolvedObservedVarianceIsNotStationaryInitialObservedVariance)
    );
    assert_eq!(
        refuse_stationary_within_subject_observed_variance_as_stationary_initial_observed_variance(
            state_only_observed,
            recovered
        ),
        Err(
            PsychometricError::StationaryWithinSubjectObservedVarianceIsNotStationaryInitialObservedVariance
        )
    );
    assert_eq!(
        refuse_initial_observed_variance_as_stationary_initial_observed_variance(
            free_initial_observed,
            recovered
        ),
        Err(PsychometricError::InitialObservedVarianceIsNotStationaryInitialObservedVariance)
    );
}

#[test]
fn stationary_initial_observed_variance_refuses_unstable_drift_and_non_event_clocks() {
    assert_eq!(
        recover_stationary_initial_observed_variance(
            2.0,
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            0.5,
            0.1,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_stationary_initial_observed_variance(
            2.0,
            0.0,
            0.4,
            0.0,
            1.0,
            0.0,
            0.5,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_initial_observed_variance(
            2.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            0.5,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_initial_observed_variance(
            2.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.5,
            0.1,
            LagClock::EventTime
        ),
        Ok(0.6)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn stationary_lagged_latent_covariance_recovers_driver_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let predictor_variance = 1.0_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_stationary_lagged_latent_covariance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary lagged T0VAR");
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
    let added = recover_asymptotic_time_independent_predictor_variance(
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let expected = trait_plus_state + added;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 lagged stationary T0VAR RMSE {error}: got {recovered}"
    );
    let contemporaneous = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
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
    assert!(rmse(&[recovered], &[contemporaneous]) > error);
    assert!(rmse(&[recovered], &[decayed]) > error);
    assert!(rmse(&[recovered], &[trait_plus_state]) > error);
    assert_eq!(
        recover_stationary_lagged_latent_covariance(
            0.0,
            0.0,
            0.0,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        ),
        Ok(0.0)
    );
    assert_eq!(
        recover_stationary_lagged_latent_covariance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            event_delta,
            LagClock::EventTime,
        ),
        Ok(trait_variance)
    );
    let far = recover_stationary_lagged_latent_covariance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        1e8,
        LagClock::EventTime,
    )
    .expect("Δt→∞");
    assert!(rmse(&[far], &[trait_variance + added]) < 1e-12);
    assert_eq!(
        refuse_stationary_lagged_latent_covariance_as_stationary_initial_latent_variance(
            recovered,
            contemporaneous
        ),
        Err(
            PsychometricError::StationaryLaggedLatentCovarianceIsNotStationaryInitialLatentVariance
        )
    );
    assert_eq!(
        refuse_stationary_lagged_latent_covariance_as_decayed_stationary_variance(
            recovered, decayed
        ),
        Err(PsychometricError::StationaryLaggedLatentCovarianceIsNotDecayedStationaryVariance)
    );
    assert_eq!(
        refuse_trait_plus_state_lagged_covariance_as_stationary_lagged_latent_covariance(
            trait_plus_state,
            recovered
        ),
        Err(PsychometricError::TraitPlusStateLaggedCovarianceIsNotStationaryLaggedLatentCovariance)
    );
}

#[test]
fn stationary_lagged_latent_covariance_refuses_unstable_drift_and_non_event_clocks() {
    assert_eq!(
        recover_stationary_lagged_latent_covariance(
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_stationary_lagged_latent_covariance(
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_stationary_lagged_latent_covariance(
            0.0,
            0.4,
            0.0,
            1.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_lagged_latent_covariance(
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_lagged_latent_covariance(
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Ok(0.0)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn stationary_lagged_observed_covariance_recovers_driver_equation_five_of_section_four_point_three()
{
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let manifest_trait = 0.1_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_stationary_lagged_observed_covariance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-lagged-stationary-T0VAR");
    let latent = recover_stationary_lagged_latent_covariance(
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary lagged T0VAR");
    let expected = recover_manifest_lagged_observed_covariance(loading, latent, manifest_trait)
        .expect("λ²c+ψ");
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 Eq. 5 of lagged stationary T0VAR RMSE {error}: got {recovered}"
    );
    let contemporaneous = recover_stationary_initial_observed_variance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-T0VAR");
    assert!(
        rmse(&[recovered], &[measurement_error]) > error,
        "MANIFESTVAR is not lagged cov(y)"
    );
    assert!(rmse(&[recovered], &[latent]) > error);
    assert!(rmse(&[recovered], &[contemporaneous]) > error);
    assert_eq!(
        recover_stationary_lagged_observed_covariance(
            0.0,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            manifest_trait,
            LagClock::EventTime,
        ),
        Ok(manifest_trait)
    );
    assert_eq!(
        refuse_stationary_lagged_latent_covariance_as_observed_covariance(latent, recovered),
        Err(PsychometricError::StationaryLaggedLatentCovarianceIsNotObservedCovariance)
    );
    assert_eq!(
        refuse_measurement_error_as_stationary_lagged_observed_covariance(
            measurement_error,
            recovered
        ),
        Err(PsychometricError::MeasurementErrorIsNotStationaryLaggedObservedCovariance)
    );
    assert_eq!(
        refuse_stationary_initial_observed_variance_as_stationary_lagged_observed_covariance(
            contemporaneous,
            recovered
        ),
        Err(
            PsychometricError::StationaryInitialObservedVarianceIsNotStationaryLaggedObservedCovariance
        )
    );
}

#[test]
fn stationary_lagged_observed_covariance_refuses_unstable_drift_and_non_event_clocks() {
    assert_eq!(
        recover_stationary_lagged_observed_covariance(
            2.0,
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            1.0,
            0.1,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_stationary_lagged_observed_covariance(
            2.0,
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            0.0,
            0.1,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_stationary_lagged_observed_covariance(
            2.0,
            0.0,
            0.4,
            0.0,
            1.0,
            0.0,
            1.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_lagged_observed_covariance(
            2.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            1.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_lagged_observed_covariance(
            2.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            1.0,
            0.1,
            LagClock::EventTime
        ),
        Ok(0.1)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn stationary_later_latent_variance_recovers_driver_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let predictor_variance = 1.0_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_stationary_later_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary later T0VAR");
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let evolved_state = recover_discrete_latent_variance(
        state,
        diffusion,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("e^{2aΔt}p+Q_Δt");
    let added = recover_asymptotic_time_independent_predictor_variance(
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let expected = trait_variance + evolved_state + added;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 later-occasion stationary T0VAR RMSE {error}: got {recovered}"
    );
    let contemporaneous = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0VAR");
    let lagged = recover_stationary_lagged_latent_covariance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary lagged T0VAR");
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
    assert!(rmse(&[recovered], &[contemporaneous]) < 1e-12);
    assert!(rmse(&[recovered], &[lagged]) > error);
    assert!(rmse(&[recovered], &[free_discrete]) > error);
    assert!(rmse(&[recovered], &[process_noise]) > error);
    assert_eq!(
        recover_stationary_later_latent_variance(
            0.0,
            0.0,
            0.0,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        ),
        Ok(0.0)
    );
    assert_eq!(
        recover_stationary_later_latent_variance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            event_delta,
            LagClock::EventTime,
        ),
        Ok(trait_variance)
    );
    let far = recover_stationary_later_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        1e8,
        LagClock::EventTime,
    )
    .expect("Δt→∞");
    assert!(rmse(&[far], &[contemporaneous]) < 1e-12);
    assert_eq!(
        refuse_stationary_later_latent_variance_as_lagged_covariance(recovered, lagged),
        Err(PsychometricError::StationaryLaterLatentVarianceIsNotLaggedCovariance)
    );
    assert_eq!(
        refuse_stationary_later_latent_variance_as_discrete_variance(recovered, free_discrete),
        Err(PsychometricError::StationaryLaterLatentVarianceIsNotDiscreteVariance)
    );
    assert_eq!(
        refuse_stationary_later_latent_variance_as_process_noise(recovered, process_noise),
        Err(PsychometricError::StationaryLaterLatentVarianceIsNotProcessNoise)
    );
}

#[test]
fn stationary_later_latent_variance_refuses_unstable_drift_and_non_event_clocks() {
    assert_eq!(
        recover_stationary_later_latent_variance(
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_stationary_later_latent_variance(
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_stationary_later_latent_variance(0.0, 0.4, 0.0, 1.0, 0.0, 1.0, LagClock::EventTime),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_later_latent_variance(
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_later_latent_variance(0.0, 0.0, 0.0, 1.0, 0.0, 1.0, LagClock::EventTime),
        Ok(0.0)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn stationary_later_observed_variance_recovers_driver_equation_five_of_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let diffusion = 0.4_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let manifest_trait = 0.1_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_stationary_later_observed_variance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later-stationary-T0VAR");
    let latent = recover_stationary_later_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary later T0VAR");
    let expected = recover_manifest_trait_plus_state_observed_variance(
        loading,
        latent,
        measurement_error,
        manifest_trait,
    )
    .expect("λ²p+θ+ψ");
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 Eq. 5 of later-occasion stationary T0VAR RMSE {error}: got {recovered}"
    );
    let contemporaneous = recover_stationary_initial_observed_variance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-T0VAR");
    let lagged = recover_stationary_lagged_observed_covariance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-lagged-stationary-T0VAR");
    assert!(rmse(&[recovered], &[contemporaneous]) < 1e-12);
    assert!(
        rmse(&[recovered], &[measurement_error]) > error,
        "MANIFESTVAR is not later Var(y)"
    );
    assert!(rmse(&[recovered], &[latent]) > error);
    assert!(rmse(&[recovered], &[lagged]) > error);
    assert_eq!(
        recover_stationary_later_observed_variance(
            0.0,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        ),
        Ok(measurement_error + manifest_trait)
    );
    assert_eq!(
        refuse_stationary_later_latent_variance_as_observed_variance(latent, recovered),
        Err(PsychometricError::StationaryLaterLatentVarianceIsNotObservedVariance)
    );
    assert_eq!(
        refuse_measurement_error_as_stationary_later_observed_variance(
            measurement_error,
            recovered
        ),
        Err(PsychometricError::MeasurementErrorIsNotStationaryLaterObservedVariance)
    );
    assert_eq!(
        refuse_stationary_lagged_observed_covariance_as_stationary_later_observed_variance(
            lagged, recovered
        ),
        Err(
            PsychometricError::StationaryLaggedObservedCovarianceIsNotStationaryLaterObservedVariance
        )
    );
}

#[test]
fn stationary_later_observed_variance_refuses_unstable_drift_and_non_event_clocks() {
    assert_eq!(
        recover_stationary_later_observed_variance(
            2.0,
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            1.0,
            0.5,
            0.1,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_stationary_later_observed_variance(
            2.0,
            1.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            0.0,
            0.5,
            0.1,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_stationary_later_observed_variance(
            2.0,
            0.0,
            0.4,
            0.0,
            1.0,
            0.0,
            1.0,
            0.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_later_observed_variance(
            2.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            1.0,
            0.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_stationary_later_observed_variance(
            2.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            1.0,
            0.5,
            0.1,
            LagClock::EventTime
        ),
        Ok(0.6)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_later_latent_variance_recovers_driver_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let predictor_variance = 1.0_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("predetermined later T0VAR");
    let evolved_state = recover_discrete_latent_variance(
        initial_latent_variance,
        diffusion,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("e^{2aΔt}p_0+Q_Δt");
    let added = recover_asymptotic_time_independent_predictor_variance(
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let expected = trait_variance + evolved_state + added;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 later-occasion predetermined T0VAR RMSE {error}: got {recovered}"
    );
    let stationary_later = recover_stationary_later_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary later T0VAR");
    let contemporaneous = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary T0VAR");
    let first_occasion_total = trait_variance + initial_latent_variance + added;
    let free_discrete = recover_discrete_latent_variance(
        first_occasion_total,
        diffusion,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("e^{2aΔt}(trait+p_0+added)+Q_Δt");
    assert!(rmse(&[recovered], &[stationary_later]) > error);
    assert!(rmse(&[recovered], &[free_discrete]) > error);
    assert!(rmse(&[recovered], &[initial_latent_variance]) > error);
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let from_stationary_start = recover_predetermined_later_latent_variance(
        trait_variance,
        state,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("p_0=−q/(2a)");
    assert!(rmse(&[from_stationary_start], &[stationary_later]) < 1e-12);
    assert_eq!(
        recover_predetermined_later_latent_variance(
            0.0,
            0.0,
            0.0,
            0.0,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        ),
        Ok(0.0)
    );
    assert_eq!(
        recover_predetermined_later_latent_variance(
            trait_variance,
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            event_delta,
            LagClock::EventTime,
        ),
        Ok(trait_variance)
    );
    let far = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        1e8,
        LagClock::EventTime,
    )
    .expect("Δt→∞");
    assert!(rmse(&[far], &[contemporaneous]) < 1e-12);
    assert_eq!(
        refuse_predetermined_later_latent_variance_as_stationary_later_latent_variance(
            recovered,
            stationary_later
        ),
        Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotStationaryLaterLatentVariance)
    );
    assert_eq!(
        refuse_predetermined_later_latent_variance_as_discrete_variance(recovered, free_discrete),
        Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotDiscreteVariance)
    );
    assert_eq!(
        refuse_predetermined_later_latent_variance_as_initial_latent_variance(
            recovered,
            initial_latent_variance
        ),
        Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotInitialLatentVariance)
    );
}

#[test]
fn predetermined_later_latent_variance_refuses_non_event_clocks_and_keeps_growing_processes() {
    assert_eq!(
        recover_predetermined_later_latent_variance(
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_predetermined_later_latent_variance(
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing = recover_predetermined_later_latent_variance(
        0.0,
        2.0,
        0.4,
        0.0,
        1.0,
        0.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("Brownian a=0");
    assert!((growing - 2.4).abs() < 1e-12);
    assert_eq!(
        recover_predetermined_later_latent_variance(
            0.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_later_latent_variance(
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Ok(0.0)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_later_observed_variance_recovers_driver_equation_five_of_section_four_point_three()
{
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let manifest_trait = 0.1_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_predetermined_later_observed_variance(
        loading,
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later-predetermined-T0VAR");
    let latent = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("predetermined later T0VAR");
    let expected = recover_manifest_trait_plus_state_observed_variance(
        loading,
        latent,
        measurement_error,
        manifest_trait,
    )
    .expect("λ²p+θ+ψ");
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 Eq. 5 of later-occasion predetermined T0VAR RMSE {error}: got {recovered}"
    );
    let stationary_later = recover_stationary_later_observed_variance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later-stationary-T0VAR");
    assert!(rmse(&[recovered], &[stationary_later]) > error);
    assert!(
        rmse(&[recovered], &[measurement_error]) > error,
        "MANIFESTVAR is not predetermined later Var(y)"
    );
    assert!(rmse(&[recovered], &[latent]) > error);
    assert_eq!(
        recover_predetermined_later_observed_variance(
            0.0,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        ),
        Ok(measurement_error + manifest_trait)
    );
    assert_eq!(
        refuse_predetermined_later_latent_variance_as_observed_variance(latent, recovered),
        Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotObservedVariance)
    );
    assert_eq!(
        refuse_measurement_error_as_predetermined_later_observed_variance(
            measurement_error,
            recovered
        ),
        Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaterObservedVariance)
    );
    assert_eq!(
        refuse_stationary_later_observed_variance_as_predetermined_later_observed_variance(
            stationary_later,
            recovered
        ),
        Err(
            PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterObservedVariance
        )
    );
}

#[test]
fn predetermined_later_observed_variance_refuses_non_event_clocks_and_keeps_growing_processes() {
    assert_eq!(
        recover_predetermined_later_observed_variance(
            2.0,
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            1.0,
            0.5,
            0.1,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_predetermined_later_observed_variance(
            2.0,
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            0.0,
            0.5,
            0.1,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing = recover_predetermined_later_observed_variance(
        1.0,
        0.0,
        2.0,
        0.4,
        0.0,
        1.0,
        0.0,
        1.0,
        0.0,
        0.0,
        LagClock::EventTime,
    )
    .expect("Brownian a=0");
    assert!((growing - 2.4).abs() < 1e-12);
    assert_eq!(
        recover_predetermined_later_observed_variance(
            2.0,
            0.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            1.0,
            0.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_later_observed_variance(
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            1.0,
            0.5,
            0.1,
            LagClock::EventTime
        ),
        Ok(0.6)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_lagged_latent_covariance_recovers_driver_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let predictor_variance = 1.0_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_predetermined_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("predetermined lagged T0VAR");
    let lagged_state = recover_discrete_lagged_latent_covariance(
        initial_latent_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("e^{aΔt}p_0");
    let added = recover_asymptotic_time_independent_predictor_variance(
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let expected = trait_variance + lagged_state + added;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 lagged predetermined T0VAR RMSE {error}: got {recovered}"
    );
    let stationary_lagged = recover_stationary_lagged_latent_covariance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("stationary lagged T0VAR");
    let later = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("predetermined later T0VAR");
    let first_occasion_total = trait_variance + initial_latent_variance + added;
    let decayed_total = recover_discrete_lagged_latent_covariance(
        first_occasion_total,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("e^{aΔt}(trait+p_0+added)");
    assert!(rmse(&[recovered], &[stationary_lagged]) > error);
    assert!(rmse(&[recovered], &[later]) > error);
    assert!(rmse(&[recovered], &[decayed_total]) > error);
    assert!(rmse(&[recovered], &[initial_latent_variance]) > error);
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let from_stationary_start = recover_predetermined_lagged_latent_covariance(
        trait_variance,
        state,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("p_0=−q/(2a)");
    assert!(rmse(&[from_stationary_start], &[stationary_lagged]) < 1e-12);
    assert_eq!(
        recover_predetermined_lagged_latent_covariance(
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            event_delta,
            LagClock::EventTime,
        ),
        Ok(0.0)
    );
    assert_eq!(
        recover_predetermined_lagged_latent_covariance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            event_delta,
            LagClock::EventTime,
        ),
        Ok(trait_variance)
    );
    let far = recover_predetermined_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        printed_effect,
        predictor_variance,
        log_rate,
        1e8,
        LagClock::EventTime,
    )
    .expect("Δt→∞");
    assert!(rmse(&[far], &[trait_variance + added]) < 1e-12);
    let near = recover_predetermined_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        printed_effect,
        predictor_variance,
        log_rate,
        1e-12,
        LagClock::EventTime,
    )
    .expect("Δt→0+");
    assert!(rmse(&[near], &[first_occasion_total]) < 1e-9);
    assert_eq!(
        refuse_predetermined_lagged_latent_covariance_as_stationary_lagged_covariance(
            recovered,
            stationary_lagged
        ),
        Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotStationaryLaggedCovariance)
    );
    assert_eq!(
        refuse_predetermined_lagged_latent_covariance_as_later_latent_variance(recovered, later),
        Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotLaterLatentVariance)
    );
    assert_eq!(
        refuse_predetermined_lagged_latent_covariance_as_decayed_total(recovered, decayed_total),
        Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotDecayedTotal)
    );
    assert_eq!(
        refuse_predetermined_lagged_latent_covariance_as_initial_latent_variance(
            recovered,
            initial_latent_variance
        ),
        Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotInitialLatentVariance)
    );
}

#[test]
fn predetermined_lagged_latent_covariance_refuses_non_event_clocks_and_keeps_growing_processes() {
    assert_eq!(
        recover_predetermined_lagged_latent_covariance(
            1.0,
            2.0,
            -0.225,
            1.0,
            -0.13,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_predetermined_lagged_latent_covariance(
            1.0,
            2.0,
            -0.225,
            1.0,
            -0.13,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing = recover_predetermined_lagged_latent_covariance(
        0.0,
        2.0,
        0.0,
        0.0,
        0.5,
        1.0,
        LagClock::EventTime,
    )
    .expect("growing carry");
    assert!(growing > 2.0);
    let brownian = recover_predetermined_lagged_latent_covariance(
        0.0,
        2.0,
        0.0,
        0.0,
        0.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("Brownian a=0");
    assert!((brownian - 2.0).abs() < 1e-12);
    assert_eq!(
        recover_predetermined_lagged_latent_covariance(
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_lagged_latent_covariance(
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Ok(0.0)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_lagged_observed_covariance_recovers_driver_equation_five_of_section_four_point_three()
 {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let manifest_trait = 0.1_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_predetermined_lagged_observed_covariance(
        loading,
        trait_variance,
        initial_latent_variance,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-lagged-predetermined-T0VAR");
    let latent = recover_predetermined_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("predetermined lagged T0VAR");
    let expected = recover_manifest_lagged_observed_covariance(loading, latent, manifest_trait)
        .expect("λ²c+ψ");
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 Eq. 5 of lagged predetermined T0VAR RMSE {error}: got {recovered}"
    );
    let stationary_lagged = recover_stationary_lagged_observed_covariance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-lagged-stationary-T0VAR");
    let later = recover_predetermined_later_observed_variance(
        loading,
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later-predetermined-T0VAR");
    assert!(rmse(&[recovered], &[stationary_lagged]) > error);
    assert!(
        rmse(&[recovered], &[measurement_error]) > error,
        "MANIFESTVAR is not predetermined lagged cov(y)"
    );
    assert!(rmse(&[recovered], &[latent]) > error);
    assert!(rmse(&[recovered], &[later]) > error);
    assert_eq!(
        recover_predetermined_lagged_observed_covariance(
            0.0,
            trait_variance,
            initial_latent_variance,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            manifest_trait,
            LagClock::EventTime,
        ),
        Ok(manifest_trait)
    );
    assert_eq!(
        refuse_predetermined_lagged_latent_covariance_as_observed_covariance(latent, recovered),
        Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotObservedCovariance)
    );
    assert_eq!(
        refuse_measurement_error_as_predetermined_lagged_observed_covariance(
            measurement_error,
            recovered
        ),
        Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaggedObservedCovariance)
    );
    assert_eq!(
        refuse_predetermined_later_observed_variance_as_predetermined_lagged_observed_covariance(
            later,
            recovered
        ),
        Err(
            PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaggedObservedCovariance
        )
    );
    assert_eq!(
        refuse_stationary_lagged_observed_covariance_as_predetermined_lagged_observed_covariance(
            stationary_lagged,
            recovered
        ),
        Err(
            PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaggedObservedCovariance
        )
    );
}

#[test]
fn predetermined_lagged_observed_covariance_refuses_non_event_clocks_and_keeps_growing_processes() {
    assert_eq!(
        recover_predetermined_lagged_observed_covariance(
            2.0,
            1.0,
            2.0,
            -0.225,
            1.0,
            -0.13,
            1.0,
            0.1,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_predetermined_lagged_observed_covariance(
            2.0,
            1.0,
            2.0,
            -0.225,
            1.0,
            -0.13,
            0.0,
            0.1,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing = recover_predetermined_lagged_observed_covariance(
        1.0,
        0.0,
        2.0,
        0.0,
        1.0,
        0.5,
        1.0,
        0.0,
        LagClock::EventTime,
    )
    .expect("growing carry");
    assert!(growing > 2.0);
    let brownian = recover_predetermined_lagged_observed_covariance(
        1.0,
        0.0,
        2.0,
        0.0,
        1.0,
        0.0,
        1.0,
        0.0,
        LagClock::EventTime,
    )
    .expect("Brownian a=0");
    assert!((brownian - 2.0).abs() < 1e-12);
    assert_eq!(
        recover_predetermined_lagged_observed_covariance(
            2.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            1.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_lagged_observed_covariance(
            2.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            1.0,
            0.1,
            LagClock::EventTime
        ),
        Ok(0.1)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_initial_latent_variance_recovers_driver_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let predictor_variance = 1.0_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_predetermined_initial_latent_variance(
        trait_variance,
        initial_latent_variance,
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("predetermined initial T0VAR");
    let added = recover_asymptotic_time_independent_predictor_variance(
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let expected = trait_variance + initial_latent_variance + added;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 predetermined first-occasion T0VAR RMSE {error}: got {recovered}"
    );
    let stationary = recover_stationary_initial_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("stationary initial T0VAR");
    let lagged = recover_predetermined_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("predetermined lagged T0VAR");
    let later = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("predetermined later T0VAR");
    assert!(rmse(&[recovered], &[stationary]) > error);
    assert!(rmse(&[recovered], &[lagged]) > error);
    assert!(rmse(&[recovered], &[later]) > error);
    assert!(rmse(&[recovered], &[initial_latent_variance]) > error);
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let from_stationary_start = recover_predetermined_initial_latent_variance(
        trait_variance,
        state,
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("p_0=−q/(2a)");
    assert!(rmse(&[from_stationary_start], &[stationary]) < 1e-12);
    let near_lagged = recover_predetermined_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        printed_effect,
        predictor_variance,
        log_rate,
        1e-12,
        LagClock::EventTime,
    )
    .expect("Δt→0+ lagged");
    assert!(rmse(&[near_lagged], &[recovered]) < 1e-9);
    let near_later = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        1e-12,
        LagClock::EventTime,
    )
    .expect("Δt→0+ later");
    assert!(rmse(&[near_later], &[recovered]) < 1e-9);
    assert_eq!(
        recover_predetermined_initial_latent_variance(
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            LagClock::EventTime,
        ),
        Ok(0.0)
    );
    assert_eq!(
        recover_predetermined_initial_latent_variance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            LagClock::EventTime,
        ),
        Ok(trait_variance)
    );
    assert_eq!(
        refuse_predetermined_initial_latent_variance_as_stationary_initial_latent_variance(
            recovered, stationary
        ),
        Err(
            PsychometricError::PredeterminedInitialLatentVarianceIsNotStationaryInitialLatentVariance
        )
    );
    assert_eq!(
        refuse_predetermined_initial_latent_variance_as_initial_latent_variance(
            recovered,
            initial_latent_variance
        ),
        Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotInitialLatentVariance)
    );
    assert_eq!(
        refuse_predetermined_initial_latent_variance_as_lagged_latent_covariance(recovered, lagged),
        Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotLaggedLatentCovariance)
    );
    assert_eq!(
        refuse_predetermined_initial_latent_variance_as_later_latent_variance(recovered, later),
        Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotLaterLatentVariance)
    );
}

#[test]
fn predetermined_initial_latent_variance_refuses_non_event_clocks_and_keeps_unstable_trait() {
    assert_eq!(
        recover_predetermined_initial_latent_variance(
            1.0,
            2.0,
            -0.225,
            1.0,
            -0.13,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    let unstable_trait =
        recover_predetermined_initial_latent_variance(1.0, 0.0, 0.0, 0.0, 0.5, LagClock::EventTime)
            .expect("trait-only a≥0");
    assert!((unstable_trait - 1.0).abs() < 1e-15);
    let brownian =
        recover_predetermined_initial_latent_variance(0.0, 2.0, 0.0, 0.0, 0.0, LagClock::EventTime)
            .expect("Brownian a=0");
    assert!((brownian - 2.0).abs() < 1e-12);
    assert_eq!(
        recover_predetermined_initial_latent_variance(
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_initial_latent_variance(0.0, 0.0, 0.0, 1.0, 0.0, LagClock::EventTime),
        Ok(0.0)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_initial_observed_variance_recovers_driver_equation_five_of_section_four_point_three()
 {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let manifest_trait = 0.1_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_predetermined_initial_observed_variance(
        loading,
        trait_variance,
        initial_latent_variance,
        printed_effect,
        1.0,
        log_rate,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-initial-predetermined-T0VAR");
    let latent = recover_predetermined_initial_latent_variance(
        trait_variance,
        initial_latent_variance,
        printed_effect,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("predetermined initial T0VAR");
    let expected = recover_manifest_trait_plus_state_observed_variance(
        loading,
        latent,
        measurement_error,
        manifest_trait,
    )
    .expect("λ²p+θ+ψ");
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 Eq. 5 of predetermined first-occasion T0VAR RMSE {error}: got {recovered}"
    );
    let stationary = recover_stationary_initial_observed_variance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-initial-stationary-T0VAR");
    let later = recover_predetermined_later_observed_variance(
        loading,
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        event_delta,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later-predetermined-T0VAR");
    assert!(rmse(&[recovered], &[stationary]) > error);
    assert!(
        rmse(&[recovered], &[measurement_error]) > error,
        "MANIFESTVAR is not predetermined first-occasion Var(y)"
    );
    assert!(rmse(&[recovered], &[latent]) > error);
    assert!(rmse(&[recovered], &[later]) > error);
    assert_eq!(
        recover_predetermined_initial_observed_variance(
            0.0,
            trait_variance,
            initial_latent_variance,
            printed_effect,
            1.0,
            log_rate,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        ),
        Ok(measurement_error + manifest_trait)
    );
    assert_eq!(
        refuse_predetermined_initial_latent_variance_as_observed_variance(latent, recovered),
        Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotObservedVariance)
    );
    assert_eq!(
        refuse_measurement_error_as_predetermined_initial_observed_variance(
            measurement_error,
            recovered
        ),
        Err(PsychometricError::MeasurementErrorIsNotPredeterminedInitialObservedVariance)
    );
    assert_eq!(
        refuse_stationary_initial_observed_variance_as_predetermined_initial_observed_variance(
            stationary, recovered
        ),
        Err(
            PsychometricError::StationaryInitialObservedVarianceIsNotPredeterminedInitialObservedVariance
        )
    );
    assert_eq!(
        refuse_predetermined_later_observed_variance_as_predetermined_initial_observed_variance(
            later, recovered
        ),
        Err(
            PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedInitialObservedVariance
        )
    );
}

#[test]
fn predetermined_initial_observed_variance_refuses_non_event_clocks_and_keeps_unstable_trait() {
    assert_eq!(
        recover_predetermined_initial_observed_variance(
            2.0,
            1.0,
            2.0,
            -0.225,
            1.0,
            -0.13,
            0.5,
            0.1,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    let brownian = recover_predetermined_initial_observed_variance(
        1.0,
        0.0,
        2.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        LagClock::EventTime,
    )
    .expect("Brownian a=0");
    assert!((brownian - 2.0).abs() < 1e-12);
    assert_eq!(
        recover_predetermined_initial_observed_variance(
            2.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            0.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_initial_observed_variance(
            2.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.5,
            0.1,
            LagClock::EventTime
        ),
        Ok(0.6)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_later_lagged_latent_covariance_recovers_driver_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let predictor_variance = 1.0_f64;
    let start_delta = 2.0_f64;
    let lag_delta = 1.0_f64;
    let recovered = recover_predetermined_later_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("predetermined later-start lagged T0VAR");
    let later_state = recover_discrete_latent_variance(
        initial_latent_variance,
        diffusion,
        log_rate,
        start_delta,
        LagClock::EventTime,
    )
    .expect("later state");
    let added = recover_asymptotic_time_independent_predictor_variance(
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let expected = recover_trait_plus_state_lagged_covariance(
        trait_variance,
        later_state,
        log_rate,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("trait + e^{as} later-state")
        + added;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 later-start lagged predetermined T0VAR RMSE {error}: got {recovered}"
    );
    let first_lagged = recover_predetermined_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        printed_effect,
        predictor_variance,
        log_rate,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("first-occasion lagged");
    let later = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta,
        LagClock::EventTime,
    )
    .expect("later variance");
    let stationary_lagged = recover_stationary_lagged_latent_covariance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("stationary lagged");
    let decayed_later = later * lag_delta.mul_add(log_rate, 0.0).exp();
    assert!(rmse(&[recovered], &[first_lagged]) > error);
    assert!(rmse(&[recovered], &[later]) > error);
    assert!(rmse(&[recovered], &[stationary_lagged]) > error);
    assert!(rmse(&[recovered], &[decayed_later]) > error);
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let from_stationary_start = recover_predetermined_later_lagged_latent_covariance(
        trait_variance,
        state,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("p_0=−q/(2a)");
    assert!(rmse(&[from_stationary_start], &[stationary_lagged]) < 1e-12);
    let near_first = recover_predetermined_later_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        1e-12,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("u→0+");
    assert!(rmse(&[near_first], &[first_lagged]) < 1e-9);
    let near_later = recover_predetermined_later_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta,
        1e-12,
        LagClock::EventTime,
    )
    .expect("s→0+");
    assert!(rmse(&[near_later], &[later]) < 1e-9);
    assert_eq!(
        recover_predetermined_later_lagged_latent_covariance(
            0.0,
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            start_delta,
            lag_delta,
            LagClock::EventTime
        ),
        Ok(0.0)
    );
    assert_eq!(
        recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            start_delta,
            lag_delta,
            LagClock::EventTime
        ),
        Ok(trait_variance)
    );
    assert_eq!(
        refuse_predetermined_later_lagged_latent_covariance_as_predetermined_lagged_covariance(
            recovered, first_lagged
        ),
        Err(
            PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotPredeterminedLaggedCovariance
        )
    );
    assert_eq!(
        refuse_predetermined_later_lagged_latent_covariance_as_later_latent_variance(
            recovered, later
        ),
        Err(PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotLaterLatentVariance)
    );
    assert_eq!(
        refuse_predetermined_later_lagged_latent_covariance_as_stationary_lagged_covariance(
            recovered,
            stationary_lagged
        ),
        Err(
            PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotStationaryLaggedCovariance
        )
    );
    assert_eq!(
        refuse_predetermined_later_lagged_latent_covariance_as_decayed_later_total(
            recovered,
            decayed_later
        ),
        Err(PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotDecayedLaterTotal)
    );
}

#[test]
fn predetermined_later_lagged_latent_covariance_refuses_non_event_clocks_and_keeps_growing_processes()
 {
    assert_eq!(
        recover_predetermined_later_lagged_latent_covariance(
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            2.0,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_predetermined_later_lagged_latent_covariance(
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_predetermined_later_lagged_latent_covariance(
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            2.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing = recover_predetermined_later_lagged_latent_covariance(
        0.0,
        2.0,
        0.4,
        0.0,
        0.0,
        0.5,
        1.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("growing a>0");
    assert!(growing.is_finite() && growing > 2.0);
    assert_eq!(
        recover_predetermined_later_lagged_latent_covariance(
            0.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_later_lagged_latent_covariance(
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Ok(0.0)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_later_lagged_observed_covariance_recovers_driver_equation_five_of_section_four_point_three()
 {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let manifest_trait = 0.1_f64;
    let start_delta = 2.0_f64;
    let lag_delta = 1.0_f64;
    let recovered = recover_predetermined_later_lagged_observed_covariance(
        loading,
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        start_delta,
        lag_delta,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later-start-lagged-predetermined-T0VAR");
    let latent = recover_predetermined_later_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        start_delta,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("later-start lagged T0VAR");
    let expected = recover_manifest_lagged_observed_covariance(loading, latent, manifest_trait)
        .expect("λ²c+ψ");
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 Eq. 5 of later-start lagged predetermined T0VAR RMSE {error}: got {recovered}"
    );
    let first_lagged = recover_predetermined_lagged_observed_covariance(
        loading,
        trait_variance,
        initial_latent_variance,
        printed_effect,
        1.0,
        log_rate,
        lag_delta,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-first-lagged");
    let later = recover_predetermined_later_observed_variance(
        loading,
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        start_delta,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later");
    let stationary = recover_stationary_lagged_observed_covariance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        lag_delta,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-lagged");
    assert!(rmse(&[recovered], &[first_lagged]) > error);
    assert!(rmse(&[recovered], &[later]) > error);
    assert!(rmse(&[recovered], &[stationary]) > error);
    assert!(
        rmse(&[recovered], &[measurement_error]) > error,
        "MANIFESTVAR is not later-start lagged predetermined cov(y)"
    );
    assert!(rmse(&[recovered], &[latent]) > error);
    assert_eq!(
        recover_predetermined_later_lagged_observed_covariance(
            0.0,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            manifest_trait,
            LagClock::EventTime,
        ),
        Ok(manifest_trait)
    );
    assert_eq!(
        refuse_predetermined_later_lagged_latent_covariance_as_observed_covariance(
            latent, recovered
        ),
        Err(PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotObservedCovariance)
    );
    assert_eq!(
        refuse_measurement_error_as_predetermined_later_lagged_observed_covariance(
            measurement_error,
            recovered
        ),
        Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaterLaggedObservedCovariance)
    );
    assert_eq!(
        refuse_predetermined_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance(
            first_lagged, recovered
        ),
        Err(
            PsychometricError::PredeterminedLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance
        )
    );
    assert_eq!(
        refuse_stationary_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance(
            stationary, recovered
        ),
        Err(
            PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance
        )
    );
    assert_eq!(
        refuse_predetermined_later_observed_variance_as_predetermined_later_lagged_observed_covariance(
            later, recovered
        ),
        Err(
            PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterLaggedObservedCovariance
        )
    );
}

#[test]
fn predetermined_later_lagged_observed_covariance_refuses_non_event_clocks_and_keeps_growing_processes()
 {
    assert_eq!(
        recover_predetermined_later_lagged_observed_covariance(
            2.0,
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            2.0,
            1.0,
            0.1,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_predetermined_later_lagged_observed_covariance(
            2.0,
            1.0,
            2.0,
            0.4,
            0.0,
            1.0,
            -0.5,
            0.0,
            1.0,
            0.1,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing = recover_predetermined_later_lagged_observed_covariance(
        1.0,
        0.0,
        2.0,
        0.4,
        0.0,
        0.0,
        0.5,
        1.0,
        1.0,
        0.0,
        LagClock::EventTime,
    )
    .expect("growing a>0");
    assert!(growing.is_finite() && growing > 2.0);
    assert_eq!(
        recover_predetermined_later_lagged_observed_covariance(
            2.0,
            0.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            2.0,
            1.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_later_lagged_observed_covariance(
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            2.0,
            1.0,
            0.1,
            LagClock::EventTime
        ),
        Ok(0.1)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_later_start_later_latent_variance_recovers_driver_section_four_point_three() {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let predictor_variance = 1.0_f64;
    let start_delta = 2.0_f64;
    let lag_delta = 1.0_f64;
    let recovered = recover_predetermined_later_start_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("predetermined later-start later T0VAR");
    let later_state = recover_discrete_latent_variance(
        initial_latent_variance,
        diffusion,
        log_rate,
        start_delta,
        LagClock::EventTime,
    )
    .expect("later state");
    let evolved_state = recover_discrete_latent_variance(
        later_state,
        diffusion,
        log_rate,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("evolved later state");
    let added = recover_asymptotic_time_independent_predictor_variance(
        printed_effect,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let expected = recover_trait_plus_state_latent_variance(trait_variance, evolved_state)
        .expect("trait + evolved later-state")
        + added;
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 later-start later-occasion predetermined T0VAR RMSE {error}: got {recovered}"
    );
    let later_full = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta + lag_delta,
        LagClock::EventTime,
    )
    .expect("later over u+s");
    assert!(rmse(&[recovered], &[later_full]) < 1e-12);
    let later = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta,
        LagClock::EventTime,
    )
    .expect("later variance");
    let later_lagged = recover_predetermined_later_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("later-start lagged");
    let stationary_later = recover_stationary_later_latent_variance(
        trait_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("stationary later");
    let decayed_later = recover_discrete_latent_variance(
        later,
        diffusion,
        log_rate,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("evolved later total");
    let lag_interval = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("later over s only");
    assert!(rmse(&[recovered], &[later]) > error);
    assert!(rmse(&[recovered], &[later_lagged]) > error);
    assert!(rmse(&[recovered], &[stationary_later]) > error);
    assert!(rmse(&[recovered], &[decayed_later]) > error);
    assert!(rmse(&[recovered], &[lag_interval]) > error);
    let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    let from_stationary_start = recover_predetermined_later_start_later_latent_variance(
        trait_variance,
        state,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("p_0=−q/(2a)");
    assert!(rmse(&[from_stationary_start], &[stationary_later]) < 1e-12);
    let near_later = recover_predetermined_later_start_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        start_delta,
        1e-12,
        LagClock::EventTime,
    )
    .expect("s→0+");
    assert!(rmse(&[near_later], &[later]) < 1e-9);
    let later_over_s = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("later over s");
    let near_first = recover_predetermined_later_start_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        predictor_variance,
        log_rate,
        1e-12,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("u→0+");
    assert!(rmse(&[near_first], &[later_over_s]) < 1e-9);
    assert_eq!(
        recover_predetermined_later_start_later_latent_variance(
            0.0,
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            start_delta,
            lag_delta,
            LagClock::EventTime
        ),
        Ok(0.0)
    );
    assert_eq!(
        recover_predetermined_later_start_later_latent_variance(
            trait_variance,
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            start_delta,
            lag_delta,
            LagClock::EventTime
        ),
        Ok(trait_variance)
    );
    assert_eq!(
        refuse_predetermined_later_start_later_latent_variance_as_later_latent_variance(
            recovered, later
        ),
        Err(PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLatentVariance)
    );
    assert_eq!(
        refuse_predetermined_later_start_later_latent_variance_as_later_lagged_covariance(
            recovered,
            later_lagged
        ),
        Err(
            PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLaggedCovariance
        )
    );
    assert_eq!(
        refuse_predetermined_later_start_later_latent_variance_as_stationary_later_latent_variance(
            recovered,
            stationary_later
        ),
        Err(
            PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotStationaryLaterLatentVariance
        )
    );
    assert_eq!(
        refuse_predetermined_later_start_later_latent_variance_as_decayed_later_total(
            recovered,
            decayed_later
        ),
        Err(PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotDecayedLaterTotal)
    );
    assert_eq!(
        refuse_predetermined_later_start_later_latent_variance_as_lag_interval_later_latent_variance(
            recovered,
            lag_interval
        ),
        Err(
            PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLagIntervalLaterLatentVariance
        )
    );
}

#[test]
fn predetermined_later_start_later_latent_variance_refuses_non_event_clocks_and_keeps_growing_processes()
 {
    assert_eq!(
        recover_predetermined_later_start_later_latent_variance(
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            2.0,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_predetermined_later_start_later_latent_variance(
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            0.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    assert_eq!(
        recover_predetermined_later_start_later_latent_variance(
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            2.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing = recover_predetermined_later_start_later_latent_variance(
        0.0,
        2.0,
        0.4,
        0.0,
        0.0,
        0.5,
        1.0,
        1.0,
        LagClock::EventTime,
    )
    .expect("growing a>0");
    assert!(growing.is_finite() && growing > 2.0);
    assert_eq!(
        recover_predetermined_later_start_later_latent_variance(
            0.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_later_start_later_latent_variance(
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            2.0,
            1.0,
            LagClock::EventTime
        ),
        Ok(0.0)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn predetermined_later_start_later_observed_variance_recovers_driver_equation_five_of_section_four_point_three()
 {
    let printed_effect = -0.225_f64;
    let printed_asym = -1.673_f64;
    let log_rate = -printed_effect / printed_asym;
    let trait_variance = 1.0_f64;
    let initial_latent_variance = 2.0_f64;
    let diffusion = 0.4_f64;
    let loading = 2.0_f64;
    let measurement_error = 0.5_f64;
    let manifest_trait = 0.1_f64;
    let start_delta = 2.0_f64;
    let lag_delta = 1.0_f64;
    let recovered = recover_predetermined_later_start_later_observed_variance(
        loading,
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        start_delta,
        lag_delta,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later-start-later-predetermined-T0VAR");
    let latent = recover_predetermined_later_start_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        start_delta,
        lag_delta,
        LagClock::EventTime,
    )
    .expect("later-start later T0VAR");
    let expected = recover_manifest_trait_plus_state_observed_variance(
        loading,
        latent,
        measurement_error,
        manifest_trait,
    )
    .expect("λ²p+θ+ψ");
    let error = rmse(&[expected], &[recovered]);
    assert!(
        error < 1e-12,
        "Driver §4.3 Eq. 5 of later-start later-occasion predetermined T0VAR RMSE {error}: got {recovered}"
    );
    let later = recover_predetermined_later_observed_variance(
        loading,
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        start_delta,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later");
    let later_lagged = recover_predetermined_later_lagged_observed_covariance(
        loading,
        trait_variance,
        initial_latent_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        start_delta,
        lag_delta,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-later-start-lagged");
    let stationary = recover_stationary_later_observed_variance(
        loading,
        trait_variance,
        diffusion,
        printed_effect,
        1.0,
        log_rate,
        lag_delta,
        measurement_error,
        manifest_trait,
        LagClock::EventTime,
    )
    .expect("eq5-stationary-later");
    assert!(rmse(&[recovered], &[later]) > error);
    assert!(rmse(&[recovered], &[later_lagged]) > error);
    assert!(rmse(&[recovered], &[stationary]) > error);
    assert!(
        rmse(&[recovered], &[measurement_error]) > error,
        "MANIFESTVAR is not later-start later-occasion predetermined Var(y)"
    );
    assert!(rmse(&[recovered], &[latent]) > error);
    assert_eq!(
        recover_predetermined_later_start_later_observed_variance(
            0.0,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        ),
        Ok(measurement_error + manifest_trait)
    );
    assert_eq!(
        refuse_predetermined_later_start_later_latent_variance_as_observed_variance(
            latent, recovered
        ),
        Err(PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotObservedVariance)
    );
    assert_eq!(
        refuse_measurement_error_as_predetermined_later_start_later_observed_variance(
            measurement_error,
            recovered
        ),
        Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaterStartLaterObservedVariance)
    );
    assert_eq!(
        refuse_predetermined_later_observed_variance_as_predetermined_later_start_later_observed_variance(
            later, recovered
        ),
        Err(
            PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance
        )
    );
    assert_eq!(
        refuse_predetermined_later_lagged_observed_covariance_as_predetermined_later_start_later_observed_variance(
            later_lagged, recovered
        ),
        Err(
            PsychometricError::PredeterminedLaterLaggedObservedCovarianceIsNotPredeterminedLaterStartLaterObservedVariance
        )
    );
    assert_eq!(
        refuse_stationary_later_observed_variance_as_predetermined_later_start_later_observed_variance(
            stationary, recovered
        ),
        Err(
            PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance
        )
    );
}

#[test]
fn predetermined_later_start_later_observed_variance_refuses_non_event_clocks_and_keeps_growing_processes()
 {
    assert_eq!(
        recover_predetermined_later_start_later_observed_variance(
            2.0,
            1.0,
            2.0,
            0.4,
            -0.225,
            1.0,
            -0.13,
            2.0,
            1.0,
            0.5,
            0.1,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_predetermined_later_start_later_observed_variance(
            2.0,
            1.0,
            2.0,
            0.4,
            0.0,
            1.0,
            -0.5,
            0.0,
            1.0,
            0.5,
            0.1,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing = recover_predetermined_later_start_later_observed_variance(
        1.0,
        0.0,
        2.0,
        0.4,
        0.0,
        0.0,
        0.5,
        1.0,
        1.0,
        0.0,
        0.0,
        LagClock::EventTime,
    )
    .expect("growing a>0");
    assert!(growing.is_finite() && growing > 2.0);
    assert_eq!(
        recover_predetermined_later_start_later_observed_variance(
            2.0,
            0.0,
            0.0,
            0.0,
            -0.225,
            1.0,
            0.5,
            2.0,
            1.0,
            0.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    assert_eq!(
        recover_predetermined_later_start_later_observed_variance(
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            2.0,
            1.0,
            0.5,
            0.1,
            LagClock::EventTime
        ),
        Ok(0.6)
    );
}

#[test]
fn standardised_discrete_drift_recovers_driver_page_sixteen_footnote_four() {
    let diffusion = 0.4_f64;
    let log_rate = -0.5_f64;
    let event_delta = 1.0_f64;
    let recovered =
        recover_standardised_discrete_drift(diffusion, log_rate, event_delta, LagClock::EventTime)
            .expect("discreteDRIFTstd");
    let unstandardised =
        recover_discrete_lag_from_log_rate(log_rate, event_delta, LagClock::EventTime)
            .expect("discreteDRIFT");
    let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    assert!(within > 0.0);
    assert!((recovered - (log_rate * event_delta).exp()).abs() < 1e-15);
    assert!((recovered - unstandardised).abs() < 1e-15);
    let two_and_a_half =
        recover_standardised_discrete_drift(diffusion, log_rate, 2.5, LagClock::EventTime)
            .expect("discreteDRIFTstd Δt=2.5");
    assert!((two_and_a_half - (log_rate * 2.5).exp()).abs() < 1e-15);
    let trait_variance = 1.0_f64;
    let lagged = recover_trait_plus_state_lagged_covariance(
        trait_variance,
        within,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("trait+state lag");
    let total =
        recover_trait_plus_state_latent_variance(trait_variance, within).expect("trait+state var");
    let contaminated = lagged / total;
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_discrete_drift_as_standardised_discrete_drift(
            unstandardised,
            recovered
        ),
        Err(PsychometricError::UnstandardisedDiscreteDriftIsNotStandardisedDiscreteDrift)
    );
    assert_eq!(
        refuse_trait_plus_state_autocorrelation_as_standardised_discrete_drift(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitPlusStateAutocorrelationIsNotStandardisedDiscreteDrift)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, within),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_discrete_drift_refuses_non_event_clocks_and_does_not_keep_growing_or_zero_diffusion()
 {
    assert_eq!(
        recover_standardised_discrete_drift(0.4, -0.5, 1.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_discrete_drift(0.4, -0.5, 0.0, LagClock::EventTime),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing =
        recover_discrete_lag_from_log_rate(0.5, 1.0, LagClock::EventTime).expect("growing a>0");
    assert!(growing.is_finite() && growing > 1.0);
    assert_eq!(
        recover_standardised_discrete_drift(0.4, 0.5, 1.0, LagClock::EventTime),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    let zero_q =
        recover_discrete_lag_from_log_rate(-0.5, 1.0, LagClock::EventTime).expect("e^{aΔt} at q=0");
    assert!(zero_q.is_finite() && zero_q > 0.0);
    assert_eq!(
        recover_standardised_discrete_drift(0.0, -0.5, 1.0, LagClock::EventTime),
        Err(PsychometricError::StandardisedDiscreteDriftRequiresPositiveWithinSubjectVariance)
    );
}

#[test]
fn standardised_discrete_diffusion_recovers_driver_page_sixteen_footnote_four() {
    let diffusion = 0.4_f64;
    let log_rate = -0.5_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_standardised_discrete_diffusion(
        diffusion,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("discreteDIFFUSIONstd");
    let process_noise =
        recover_discrete_process_noise(diffusion, log_rate, event_delta, LagClock::EventTime)
            .expect("discreteDIFFUSION");
    let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    assert!(within > 0.0);
    assert!((recovered - (process_noise / within)).abs() < 1e-15);
    assert!((recovered - (1.0 - (2.0 * log_rate * event_delta).exp())).abs() < 1e-15);
    let two_and_a_half =
        recover_standardised_discrete_diffusion(diffusion, log_rate, 2.5, LagClock::EventTime)
            .expect("discreteDIFFUSIONstd Δt=2.5");
    assert!((two_and_a_half - (1.0 - (2.0 * log_rate * 2.5).exp())).abs() < 1e-15);
    let continuous_std = diffusion / within;
    assert!((continuous_std - recovered).abs() > 1e-3);
    let trait_variance = 1.0_f64;
    let total =
        recover_trait_plus_state_latent_variance(trait_variance, within).expect("trait+state var");
    let contaminated = process_noise / total;
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_discrete_diffusion_as_standardised_discrete_diffusion(
            process_noise,
            recovered
        ),
        Err(PsychometricError::UnstandardisedDiscreteDiffusionIsNotStandardisedDiscreteDiffusion)
    );
    assert_eq!(
        refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion(
            continuous_std,
            recovered
        ),
        Err(PsychometricError::StandardisedContinuousDiffusionIsNotStandardisedDiscreteDiffusion)
    );
    assert_eq!(
        refuse_trait_contaminated_process_noise_as_standardised_discrete_diffusion(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedProcessNoiseIsNotStandardisedDiscreteDiffusion)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, within),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_discrete_diffusion_refuses_non_event_clocks_and_does_not_keep_growing_or_zero_diffusion()
 {
    assert_eq!(
        recover_standardised_discrete_diffusion(0.4, -0.5, 1.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_discrete_diffusion(0.4, -0.5, 0.0, LagClock::EventTime),
        Err(PsychometricError::NonPositiveInterval)
    );
    let growing =
        recover_discrete_process_noise(0.4, 0.5, 1.0, LagClock::EventTime).expect("growing a>0");
    assert!(growing.is_finite() && growing > 0.0);
    assert_eq!(
        recover_standardised_discrete_diffusion(0.4, 0.5, 1.0, LagClock::EventTime),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    let zero_q =
        recover_discrete_process_noise(0.0, -0.5, 1.0, LagClock::EventTime).expect("Q_Δt at q=0");
    assert!((zero_q - 0.0).abs() < 1e-15);
    assert_eq!(
        recover_standardised_discrete_diffusion(0.0, -0.5, 1.0, LagClock::EventTime),
        Err(PsychometricError::StandardisedDiscreteDiffusionRequiresPositiveWithinSubjectVariance)
    );
}

#[test]
fn standardised_continuous_diffusion_recovers_driver_page_sixteen_footnote_four() {
    let diffusion = 0.4_f64;
    let log_rate = -0.5_f64;
    let recovered =
        recover_standardised_continuous_diffusion(diffusion, log_rate, LagClock::EventTime)
            .expect("DIFFUSIONstd");
    let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    assert!(within > 0.0);
    assert!((recovered - (diffusion / within)).abs() < 1e-15);
    assert!((recovered - (-2.0 * log_rate)).abs() < 1e-15);
    let larger_q = recover_standardised_continuous_diffusion(2.0, log_rate, LagClock::EventTime)
        .expect("DIFFUSIONstd q=2");
    assert!((larger_q - recovered).abs() < 1e-15);
    let discrete =
        recover_standardised_discrete_diffusion(diffusion, log_rate, 1.0, LagClock::EventTime)
            .expect("discreteDIFFUSIONstd");
    assert!((discrete - recovered).abs() > 1e-3);
    let trait_variance = 1.0_f64;
    let total =
        recover_trait_plus_state_latent_variance(trait_variance, within).expect("trait+state var");
    let contaminated = diffusion / total;
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_continuous_diffusion_as_standardised_continuous_diffusion(
            diffusion,
            recovered
        ),
        Err(PsychometricError::UnstandardisedContinuousDiffusionIsNotStandardisedContinuousDiffusion)
    );
    assert_eq!(
        refuse_standardised_discrete_diffusion_as_standardised_continuous_diffusion(
            discrete, recovered
        ),
        Err(PsychometricError::StandardisedDiscreteDiffusionIsNotStandardisedContinuousDiffusion)
    );
    assert_eq!(
        refuse_trait_contaminated_continuous_diffusion_as_standardised_continuous_diffusion(
            contaminated,
            recovered
        ),
        Err(
            PsychometricError::TraitContaminatedContinuousDiffusionIsNotStandardisedContinuousDiffusion
        )
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, within),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_continuous_diffusion_refuses_non_event_clocks_and_does_not_keep_growing_or_zero_diffusion()
 {
    assert_eq!(
        recover_standardised_continuous_diffusion(0.4, -0.5, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    let growing =
        recover_discrete_process_noise(0.4, 0.5, 1.0, LagClock::EventTime).expect("growing a>0");
    assert!(growing.is_finite() && growing > 0.0);
    assert_eq!(
        recover_standardised_continuous_diffusion(0.4, 0.5, LagClock::EventTime),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_standardised_continuous_diffusion(0.0, -0.5, LagClock::EventTime),
        Err(
            PsychometricError::StandardisedContinuousDiffusionRequiresPositiveWithinSubjectVariance
        )
    );
}

#[test]
fn standardised_continuous_drift_recovers_driver_page_sixteen_footnote_four() {
    let diffusion = 0.4_f64;
    let log_rate = -0.5_f64;
    let recovered = recover_standardised_continuous_drift(diffusion, log_rate, LagClock::EventTime)
        .expect("DRIFTstd");
    let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    assert!(within > 0.0);
    assert!((recovered - log_rate).abs() < 1e-15);
    let larger_q = recover_standardised_continuous_drift(2.0, log_rate, LagClock::EventTime)
        .expect("DRIFTstd q=2");
    assert!((larger_q - recovered).abs() < 1e-15);
    let discrete =
        recover_standardised_discrete_drift(diffusion, log_rate, 1.0, LagClock::EventTime)
            .expect("discreteDRIFTstd");
    assert!((discrete - recovered).abs() > 1e-3);
    let trait_variance = 1.0_f64;
    let total =
        recover_trait_plus_state_latent_variance(trait_variance, within).expect("trait+state var");
    let contaminated = log_rate * (within / total);
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_continuous_drift_as_standardised_continuous_drift(
            log_rate, recovered
        ),
        Err(PsychometricError::UnstandardisedContinuousDriftIsNotStandardisedContinuousDrift)
    );
    assert_eq!(
        refuse_standardised_discrete_drift_as_standardised_continuous_drift(discrete, recovered),
        Err(PsychometricError::StandardisedDiscreteDriftIsNotStandardisedContinuousDrift)
    );
    assert_eq!(
        refuse_trait_contaminated_continuous_drift_as_standardised_continuous_drift(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedContinuousDriftIsNotStandardisedContinuousDrift)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, within),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_continuous_drift_refuses_non_event_clocks_and_does_not_keep_growing_or_zero_diffusion()
 {
    assert_eq!(
        recover_standardised_continuous_drift(0.4, -0.5, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_continuous_drift(0.4, 0.5, LagClock::EventTime),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_standardised_continuous_drift(0.0, -0.5, LagClock::EventTime),
        Err(PsychometricError::StandardisedContinuousDriftRequiresPositiveWithinSubjectVariance)
    );
}

#[test]
fn standardised_asymptotic_time_independent_effect_recovers_driver_page_sixteen_footnote_four() {
    let diffusion = 0.4_f64;
    let log_rate = -0.5_f64;
    let coefficient = 0.3_f64;
    let predictor_variance = 1.0_f64;
    let recovered = recover_standardised_asymptotic_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        diffusion,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECTstd");
    let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    assert!(within > 0.0);
    let unit = recover_asymptotic_time_independent_predictor_effect(
        coefficient,
        1.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECT");
    assert!((recovered - unit * predictor_variance.sqrt() / within.sqrt()).abs() < 1e-15);
    let larger_q = recover_standardised_asymptotic_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        2.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECTstd q=2");
    assert!((larger_q - recovered).abs() > 1e-3);
    let discrete_increment = recover_discrete_time_independent_predictor_effect(
        coefficient,
        1.0,
        log_rate,
        1.0,
        LagClock::EventTime,
    )
    .expect("discrete TIPREDEFFECT");
    let discrete_std = discrete_increment * predictor_variance.sqrt() / within.sqrt();
    assert!((discrete_std - recovered).abs() > 1e-3);
    let trait_variance = 1.0_f64;
    let total =
        recover_trait_plus_state_latent_variance(trait_variance, within).expect("trait+state var");
    let contaminated = unit * predictor_variance.sqrt() / total.sqrt();
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect(
            unit, recovered
        ),
        Err(PsychometricError::UnstandardisedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect)
    );
    assert_eq!(
        refuse_standardised_discrete_time_independent_effect_as_standardised_asymptotic_time_independent_effect(
            discrete_std, recovered
        ),
        Err(PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect)
    );
    assert_eq!(
        refuse_trait_contaminated_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, within),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_asymptotic_time_independent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances()
 {
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_effect(
            0.3,
            1.0,
            0.4,
            -0.5,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_effect(
            0.3,
            1.0,
            0.4,
            0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_effect(
            0.3,
            1.0,
            0.0,
            -0.5,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositiveWithinSubjectVariance
        )
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_effect(
            0.3,
            0.0,
            0.4,
            -0.5,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositivePredictorVariance
        )
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_effect(
            f64::NAN,
            1.0,
            0.4,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_effect(
            0.3,
            f64::NAN,
            0.4,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_effect(
            0.3,
            -0.1,
            0.4,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_effect(
            0.3,
            f64::MAX,
            1e-308,
            -1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_effect(
            f64::MAX,
            4.0,
            2.0,
            -1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn standardised_discrete_time_independent_effect_recovers_driver_page_sixteen_footnote_four() {
fn standardised_continuous_time_independent_effect_recovers_driver_page_sixteen_footnote_four() {
    let diffusion = 0.4_f64;
    let log_rate = -0.5_f64;
    let coefficient = 0.3_f64;
    let predictor_variance = 1.0_f64;
    let event_delta = 1.0_f64;
    let recovered = recover_standardised_discrete_time_independent_predictor_effect(
    let recovered = recover_standardised_continuous_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        diffusion,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("finite-interval TIPREDEFFECTstd");
    let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    assert!(within > 0.0);
    let unit = recover_discrete_time_independent_predictor_effect(
        coefficient,
        1.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("discrete TIPREDEFFECT");
    assert!((recovered - unit * predictor_variance.sqrt() / within.sqrt()).abs() < 1e-15);
    let larger_q = recover_standardised_discrete_time_independent_predictor_effect(
        LagClock::EventTime,
    )
    .expect("TIPREDEFFECTstd");
    let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    assert!(within > 0.0);
    assert!((recovered - coefficient * predictor_variance.sqrt() / within.sqrt()).abs() < 1e-15);
    let larger_q = recover_standardised_continuous_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        2.0,
        log_rate,
        event_delta,
        LagClock::EventTime,
    )
    .expect("finite-interval TIPREDEFFECTstd q=2");
    assert!((larger_q - recovered).abs() > 1e-3);
    let later = recover_standardised_discrete_time_independent_predictor_effect(
        LagClock::EventTime,
    )
    .expect("TIPREDEFFECTstd q=2");
    assert!((larger_q - recovered).abs() > 1e-3);
    let asymptotic = recover_standardised_asymptotic_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        diffusion,
        log_rate,
        2.0,
        LagClock::EventTime,
    )
    .expect("finite-interval TIPREDEFFECTstd Δt=2");
    assert!((later - recovered).abs() > 1e-3);
    let asymptotic = recover_standardised_asymptotic_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        diffusion,
        log_rate,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECTstd");
    assert!((asymptotic - recovered).abs() > 1e-3);
    let trait_variance = 1.0_f64;
    let total =
        recover_trait_plus_state_latent_variance(trait_variance, within).expect("trait+state var");
    let contaminated = unit * predictor_variance.sqrt() / total.sqrt();
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_discrete_time_independent_effect_as_standardised_discrete_time_independent_effect(
            unit, recovered
        ),
        Err(PsychometricError::UnstandardisedDiscreteTimeIndependentEffectIsNotStandardisedDiscreteTimeIndependentEffect)
    );
    assert_eq!(
        refuse_standardised_asymptotic_time_independent_effect_as_standardised_discrete_time_independent_effect(
            asymptotic, recovered
        ),
        Err(PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedDiscreteTimeIndependentEffect)
    );
    assert_eq!(
        refuse_trait_contaminated_discrete_time_independent_effect_as_standardised_discrete_time_independent_effect(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedDiscreteTimeIndependentEffectIsNotStandardisedDiscreteTimeIndependentEffect)
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECTstd");
    assert!((asymptotic - recovered).abs() > 1e-3);
    let discrete_increment = recover_discrete_time_independent_predictor_effect(
        coefficient,
        1.0,
        log_rate,
        1.0,
        LagClock::EventTime,
    )
    .expect("discrete TIPREDEFFECT");
    let discrete_std = discrete_increment * predictor_variance.sqrt() / within.sqrt();
    assert!((discrete_std - recovered).abs() > 1e-3);
    let trait_variance = 1.0_f64;
    let total =
        recover_trait_plus_state_latent_variance(trait_variance, within).expect("trait+state var");
    let contaminated = coefficient * predictor_variance.sqrt() / total.sqrt();
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect(
            coefficient, recovered
        ),
        Err(PsychometricError::UnstandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect)
    );
    assert_eq!(
        refuse_standardised_asymptotic_time_independent_effect_as_standardised_continuous_time_independent_effect(
            asymptotic, recovered
        ),
        Err(PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect)
    );
    assert_eq!(
        refuse_standardised_discrete_time_independent_effect_as_standardised_continuous_time_independent_effect(
            discrete_std, recovered
        ),
        Err(PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect)
    );
    assert_eq!(
        refuse_trait_contaminated_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, within),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_discrete_time_independent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances()
 {
    assert_eq!(
        recover_standardised_discrete_time_independent_predictor_effect(
fn standardised_continuous_time_independent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances()
 {
fn standardised_continuous_time_independent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances(
) {
fn standardised_continuous_time_independent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances()
 {
    assert_eq!(
        recover_standardised_continuous_time_independent_predictor_effect(
            0.3,
            1.0,
            0.4,
            -0.5,
            1.0,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_discrete_time_independent_predictor_effect(
        recover_standardised_continuous_time_independent_predictor_effect(
            0.3,
            1.0,
            0.4,
            0.5,
            1.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_standardised_discrete_time_independent_predictor_effect(
        recover_standardised_continuous_time_independent_predictor_effect(
            0.3,
            1.0,
            0.0,
            -0.5,
            1.0,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedDiscreteTimeIndependentEffectRequiresPositiveWithinSubjectVariance
        )
    );
    assert_eq!(
        recover_standardised_discrete_time_independent_predictor_effect(
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositiveWithinSubjectVariance
        )
    );
    assert_eq!(
        recover_standardised_continuous_time_independent_predictor_effect(
            0.3,
            0.0,
            0.4,
            -0.5,
            1.0,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedDiscreteTimeIndependentEffectRequiresPositivePredictorVariance
        )
    );
    assert_eq!(
        recover_standardised_discrete_time_independent_predictor_effect(
            0.3,
            1.0,
            0.4,
            -0.5,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::NonPositiveInterval)
    );
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositivePredictorVariance
        )
    );
    assert_eq!(
        recover_standardised_continuous_time_independent_predictor_effect(
            f64::NAN,
            1.0,
            0.4,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_continuous_time_independent_predictor_effect(
            0.3,
            f64::NAN,
            0.4,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_continuous_time_independent_predictor_effect(
            0.3,
            -0.1,
            0.4,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn standardised_continuous_intercept_recovers_driver_page_sixteen_footnote_four() {
    let diffusion = 0.4_f64;
    let log_rate = -0.5_f64;
    let intercept = 0.3_f64;
    let recovered = recover_standardised_continuous_intercept(
        intercept,
fn standardised_continuous_time_dependent_effect_recovers_driver_page_sixteen_footnote_four() {
    let diffusion = 0.4_f64;
    let log_rate = -0.5_f64;
    let coefficient = 0.3_f64;
    let predictor_variance = 1.0_f64;
    let recovered = recover_standardised_continuous_time_dependent_predictor_effect(
        coefficient,
        predictor_variance,
        diffusion,
        log_rate,
        LagClock::EventTime,
    )
    .expect("CINTstd");
    let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    assert!(within > 0.0);
    assert!((recovered - intercept / within.sqrt()).abs() < 1e-15);
    let larger_q =
        recover_standardised_continuous_intercept(intercept, 2.0, log_rate, LagClock::EventTime)
            .expect("CINTstd q=2");
    assert!((larger_q - recovered).abs() > 1e-3);
    assert!(larger_q.abs() < recovered.abs());
    let asymptotic =
        recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
            .expect("asymCINT");
    let asymptotic_std = asymptotic / within.sqrt();
    assert!((asymptotic_std - recovered).abs() > 1e-3);
    let discrete_increment =
        recover_discrete_continuous_intercept_effect(intercept, log_rate, 1.0, LagClock::EventTime)
            .expect("discrete CINT");
    let discrete_std = discrete_increment / within.sqrt();
    .expect("TDPREDEFFECTstd");
    let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
        .expect("asymDIFFUSION");
    assert!(within > 0.0);
    assert!((recovered - coefficient * predictor_variance.sqrt() / within.sqrt()).abs() < 1e-15);
    let larger_q = recover_standardised_continuous_time_dependent_predictor_effect(
        coefficient,
        predictor_variance,
        2.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("TDPREDEFFECTstd q=2");
    assert!((larger_q - recovered).abs() > 1e-3);
    let tipred = recover_standardised_continuous_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        diffusion,
        log_rate,
        LagClock::EventTime,
    )
    .expect("TIPREDEFFECTstd");
    assert_eq!(tipred.to_bits(), recovered.to_bits());
    let discrete_increment = recover_discrete_time_independent_predictor_effect(
        coefficient,
        1.0,
        log_rate,
        1.0,
        LagClock::EventTime,
    )
    .expect("intercept-style discrete TDPREDEFFECT");
    let discrete_std = discrete_increment * predictor_variance.sqrt() / within.sqrt();
    assert!((discrete_std - recovered).abs() > 1e-3);
    let trait_variance = 1.0_f64;
    let total =
        recover_trait_plus_state_latent_variance(trait_variance, within).expect("trait+state var");
    let contaminated = intercept / total.sqrt();
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_continuous_intercept_as_standardised_continuous_intercept(
            intercept, recovered
        ),
        Err(PsychometricError::UnstandardisedContinuousInterceptIsNotStandardisedContinuousIntercept)
    );
    assert_eq!(
        refuse_standardised_asymptotic_continuous_intercept_as_standardised_continuous_intercept(
            asymptotic_std, recovered
        ),
        Err(PsychometricError::StandardisedAsymptoticContinuousInterceptIsNotStandardisedContinuousIntercept)
    );
    assert_eq!(
        refuse_standardised_discrete_continuous_intercept_as_standardised_continuous_intercept(
            discrete_std, recovered
        ),
        Err(PsychometricError::StandardisedDiscreteContinuousInterceptIsNotStandardisedContinuousIntercept)
    );
    assert_eq!(
        refuse_trait_contaminated_continuous_intercept_as_standardised_continuous_intercept(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedContinuousInterceptIsNotStandardisedContinuousIntercept)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, within),
    let contaminated = coefficient * predictor_variance.sqrt() / total.sqrt();
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect(
            coefficient, recovered
        ),
        Err(PsychometricError::UnstandardisedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect)
    );
    assert_eq!(
        refuse_standardised_continuous_time_independent_effect_as_standardised_continuous_time_dependent_effect(
            tipred, recovered
        ),
        Err(PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeDependentEffect)
    );
    assert_eq!(
        refuse_standardised_discrete_time_dependent_effect_as_standardised_continuous_time_dependent_effect(
            discrete_std, recovered
        ),
        Err(PsychometricError::StandardisedDiscreteTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect)
    );
    assert_eq!(
        refuse_trait_contaminated_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, within),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_continuous_time_dependent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances()
 {
    assert_eq!(
        recover_standardised_continuous_time_dependent_predictor_effect(
            0.3,
            1.0,
            0.4,
            -0.5,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_continuous_time_dependent_predictor_effect(
            0.3,
            1.0,
            0.4,
            0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_standardised_continuous_time_dependent_predictor_effect(
            0.3,
            1.0,
            0.0,
            -0.5,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositiveWithinSubjectVariance
        )
    );
    assert_eq!(
        recover_standardised_continuous_time_dependent_predictor_effect(
            0.3,
            0.0,
            0.4,
            -0.5,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositivePredictorVariance
        )
    );
}

#[test]
fn standardised_initial_time_dependent_effect_recovers_driver_table_three_footnote_four() {
    let initial_variance = 1.6_f64;
    let coefficient = 0.3_f64;
    let predictor_variance = 1.0_f64;
    let recovered = recover_standardised_initial_time_dependent_predictor_effect(
        coefficient,
        predictor_variance,
        initial_variance,
        LagClock::EventTime,
    )
    .expect("T0TDPREDEFFECTstd");
    assert!(
        (recovered - coefficient * predictor_variance.sqrt() / initial_variance.sqrt()).abs()
            < 1e-15
    );
    let larger_p0 = recover_standardised_initial_time_dependent_predictor_effect(
        coefficient,
        predictor_variance,
        6.4,
        LagClock::EventTime,
    )
    .expect("T0TDPREDEFFECTstd p_0=6.4");
    assert!((larger_p0 - recovered).abs() > 1e-3);
    let continuous = recover_standardised_continuous_time_dependent_predictor_effect(
        coefficient,
        predictor_variance,
        0.4,
        -0.5,
        LagClock::EventTime,
    )
    .expect("TDPREDEFFECTstd");
    assert!((continuous - recovered).abs() > 1e-3);
    let t0_tipred = recover_standardised_initial_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        initial_variance,
        LagClock::EventTime,
    )
    .expect("T0TIPREDEFFECTstd");
    assert_eq!(t0_tipred.to_bits(), recovered.to_bits());
    let trait_variance = 1.0_f64;
    let total = recover_trait_plus_state_latent_variance(trait_variance, initial_variance)
        .expect("trait+state var");
    let contaminated = coefficient * predictor_variance.sqrt() / total.sqrt();
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect(
            coefficient, recovered
        ),
        Err(PsychometricError::UnstandardisedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect)
    );
    assert_eq!(
        refuse_standardised_continuous_time_dependent_effect_as_standardised_initial_time_dependent_effect(
            continuous, recovered
        ),
        Err(PsychometricError::StandardisedContinuousTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect)
    );
    assert_eq!(
        refuse_standardised_initial_time_independent_effect_as_standardised_initial_time_dependent_effect(
            t0_tipred, recovered
        ),
        Err(PsychometricError::StandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeDependentEffect)
    );
    assert_eq!(
        refuse_trait_contaminated_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, initial_variance),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_initial_time_dependent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances()
 {
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            1.0,
            1.6,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            1.0,
            0.0,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositiveInitialLatentVariance
        )
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            0.0,
            1.6,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositivePredictorVariance
        )
    );
}

#[test]
fn standardised_initial_latent_variance_recovers_driver_table_two_correlation() {
    let initial_variance = 1.6_f64;
    let recovered =
        recover_standardised_initial_latent_variance(initial_variance, LagClock::EventTime)
            .expect("T0VARstd");
    assert!((recovered - 1.0).abs() < 1e-15);
    let larger_p0 = recover_standardised_initial_latent_variance(6.4, LagClock::EventTime)
        .expect("T0VARstd p_0=6.4");
    assert_eq!(larger_p0.to_bits(), recovered.to_bits());
    let standardised_effect = recover_standardised_initial_time_dependent_predictor_effect(
        0.3,
        1.0,
        initial_variance,
        LagClock::EventTime,
    )
    .expect("T0TDPREDEFFECTstd");
    let effect_error = (standardised_effect - 1.0).abs();
    let recovered_error = (recovered - 1.0).abs();
    assert!(
        recovered_error < effect_error,
        "Driver et al. (2017, p. 16): T0TDPREDEFFECTstd RMSE {effect_error} must exceed T0VARstd RMSE {recovered_error}"
    );
    let extra = recover_initial_time_independent_predictor_variance(0.3, 4.0, LagClock::EventTime)
        .expect("addedT0TIPREDVAR");
    let extra_error = (extra - 1.0).abs();
    assert!(
        recovered_error < extra_error,
        "Driver et al. (2017, 2017-era addedT0TIPREDVAR): extra RMSE {extra_error} must exceed T0VARstd RMSE {recovered_error}"
    );
    assert_eq!(
        refuse_unstandardised_initial_latent_variance_as_standardised_initial_latent_variance(
            initial_variance,
            recovered
        ),
        Err(
            PsychometricError::UnstandardisedInitialLatentVarianceIsNotStandardisedInitialLatentVariance
        )
    );
    assert_eq!(
        refuse_standardised_initial_time_dependent_effect_as_standardised_initial_latent_variance(
            standardised_effect,
            recovered
        ),
        Err(
            PsychometricError::StandardisedInitialTimeDependentEffectIsNotStandardisedInitialLatentVariance
        )
    );
    assert_eq!(
        refuse_initial_time_independent_variance_as_standardised_initial_latent_variance(
            extra, recovered
        ),
        Err(
            PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedInitialLatentVariance
        )
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(1.0, initial_variance),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
fn initial_time_dependent_predictor_variance_recovers_analog_of_added_t0_tipred_var() {
    let coefficient = 0.3_f64;
    let predictor_variance = 4.0_f64;
    let recovered = recover_initial_time_dependent_predictor_variance(
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("addedT0TDPREDVAR analog");
    assert!((recovered - coefficient * coefficient * predictor_variance).abs() < 1e-15);
    let ti_extra = recover_initial_time_independent_predictor_variance(
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("addedT0TIPREDVAR");
    assert_eq!(ti_extra.to_bits(), recovered.to_bits());
    let standardised = recover_standardised_initial_time_dependent_predictor_effect(
        coefficient,
        predictor_variance,
        1.6,
        LagClock::EventTime,
    )
    .expect("T0TDPREDEFFECTstd");
    assert!((standardised - recovered).abs() > 1e-3);
    let covariance = coefficient * predictor_variance;
    assert!((covariance - recovered).abs() > 1e-3);
    assert_eq!(
        recover_initial_time_dependent_predictor_variance(0.0, 4.0, LagClock::EventTime)
            .expect("zero")
            .to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        refuse_initial_time_dependent_variance_as_initial_time_independent_variance(
            recovered, ti_extra
        ),
        Err(PsychometricError::InitialTimeDependentVarianceIsNotInitialTimeIndependentVariance)
    );
    assert_eq!(
        refuse_initial_time_dependent_variance_as_standardised_initial_time_dependent_effect(
            recovered, standardised
        ),
        Err(
            PsychometricError::InitialTimeDependentVarianceIsNotStandardisedInitialTimeDependentEffect
        )
    );
    assert_eq!(
        refuse_initial_time_dependent_variance_as_initial_time_dependent_covariance(
            recovered, covariance
        ),
        Err(PsychometricError::InitialTimeDependentVarianceIsNotInitialTimeDependentCovariance)
    );
    assert_eq!(
        refuse_initial_time_dependent_variance_as_initial_latent_variance(recovered, 1.6),
        Err(PsychometricError::InitialTimeDependentVarianceIsNotInitialLatentVariance)
    );
    assert_eq!(
        refuse_initial_time_dependent_variance_as_trait_variance(recovered, 1.0),
        Err(PsychometricError::InitialTimeDependentVarianceIsNotTraitVariance)
    );
}

#[test]
fn standardised_initial_latent_variance_refuses_non_event_clocks_and_does_not_keep_zero_variance() {
    assert_eq!(
        recover_standardised_initial_latent_variance(1.6, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_initial_latent_variance(0.0, LagClock::EventTime),
        Err(
            PsychometricError::StandardisedInitialLatentVarianceRequiresPositiveInitialLatentVariance
        )
fn initial_time_dependent_predictor_variance_refuses_non_event_clocks_and_negative_variance() {
    assert_eq!(
        recover_initial_time_dependent_predictor_variance(0.3, 4.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_initial_time_dependent_predictor_variance(0.3, -0.1, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn standardised_trait_variance_recovers_driver_table_two_correlation() {
    let trait_variance = 1.6_f64;
    let recovered = recover_standardised_trait_variance(trait_variance, LagClock::EventTime)
        .expect("TRAITVARstd");
    assert!((recovered - 1.0).abs() < 1e-15);
    let larger_trait = recover_standardised_trait_variance(6.4, LagClock::EventTime)
        .expect("TRAITVARstd trait=6.4");
    assert_eq!(larger_trait.to_bits(), recovered.to_bits());
    let t0var_std =
        recover_standardised_initial_latent_variance(trait_variance, LagClock::EventTime)
            .expect("T0VARstd");
    assert_eq!(t0var_std.to_bits(), recovered.to_bits());
    let extra = recover_initial_time_independent_predictor_variance(0.3, 4.0, LagClock::EventTime)
        .expect("addedT0TIPREDVAR");
    let extra_error = (extra - 1.0).abs();
    let recovered_error = (recovered - 1.0).abs();
    assert!(
        recovered_error < extra_error,
        "Driver et al. (2017, 2017-era addedT0TIPREDVAR): extra RMSE {extra_error} must exceed TRAITVARstd RMSE {recovered_error}"
    );
    let unstandardised_error = (trait_variance - 1.0).abs();
    assert!(
        recovered_error < unstandardised_error,
        "Driver et al. (2017, Table 2): unstandardised TRAITVAR RMSE {unstandardised_error} must exceed TRAITVARstd RMSE {recovered_error}"
    );
    assert_eq!(
        refuse_unstandardised_trait_variance_as_standardised_trait_variance(
            trait_variance,
            recovered
        ),
        Err(PsychometricError::UnstandardisedTraitVarianceIsNotStandardisedTraitVariance)
    );
    assert_eq!(
        refuse_standardised_initial_latent_variance_as_standardised_trait_variance(
            t0var_std, recovered
        ),
        Err(PsychometricError::StandardisedInitialLatentVarianceIsNotStandardisedTraitVariance)
    );
    assert_eq!(
        refuse_initial_time_independent_variance_as_standardised_trait_variance(extra, recovered),
        Err(PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedTraitVariance)
    );
}

#[test]
fn standardised_trait_variance_refuses_non_event_clocks_and_does_not_keep_zero_variance() {
    assert_eq!(
        recover_standardised_trait_variance(1.6, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_trait_variance(0.0, LagClock::EventTime),
        Err(PsychometricError::StandardisedTraitVarianceRequiresPositiveTraitVariance)
    );
}

#[test]
fn standardised_manifest_trait_variance_recovers_driver_table_two_correlation() {
    let manifest_trait = 1.6_f64;
    let recovered =
        recover_standardised_manifest_trait_variance(manifest_trait, LagClock::EventTime)
            .expect("MANIFESTTRAITVARstd");
    assert!((recovered - 1.0).abs() < 1e-15);
    let larger_psi = recover_standardised_manifest_trait_variance(6.4, LagClock::EventTime)
        .expect("MANIFESTTRAITVARstd ψ=6.4");
    assert_eq!(larger_psi.to_bits(), recovered.to_bits());
    let trait_std = recover_standardised_trait_variance(manifest_trait, LagClock::EventTime)
        .expect("TRAITVARstd");
    assert_eq!(trait_std.to_bits(), recovered.to_bits());
    let recovered_error = (recovered - 1.0).abs();
    let unstandardised_error = (manifest_trait - 1.0).abs();
    assert!(
        recovered_error < unstandardised_error,
        "Driver et al. (2017, Table 2): unstandardised MANIFESTTRAITVAR RMSE {unstandardised_error} must exceed MANIFESTTRAITVARstd RMSE {recovered_error}"
    );
    let measurement_error = 0.4_f64;
    let measurement_error_rmse = (measurement_error - 1.0).abs();
    assert!(
        recovered_error < measurement_error_rmse,
        "Driver et al. (2017, Table 2): MANIFESTVAR RMSE {measurement_error_rmse} must exceed MANIFESTTRAITVARstd RMSE {recovered_error}"
    );
    assert_eq!(
        refuse_unstandardised_manifest_trait_variance_as_standardised_manifest_trait_variance(
            manifest_trait,
            recovered
        ),
        Err(
            PsychometricError::UnstandardisedManifestTraitVarianceIsNotStandardisedManifestTraitVariance
        )
    );
    assert_eq!(
        refuse_standardised_trait_variance_as_standardised_manifest_trait_variance(
            trait_std, recovered
        ),
        Err(PsychometricError::StandardisedTraitVarianceIsNotStandardisedManifestTraitVariance)
    );
    assert_eq!(
        refuse_measurement_error_as_standardised_manifest_trait_variance(
            measurement_error,
            recovered
        ),
        Err(PsychometricError::MeasurementErrorIsNotStandardisedManifestTraitVariance)
fn initial_time_dependent_observed_variance_recovers_eq5_of_analog_added_t0_tipred_var() {
    let loading = 2.0_f64;
    let coefficient = 0.3_f64;
    let predictor_variance = 4.0_f64;
    let extra = recover_initial_time_dependent_predictor_variance(
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("addedT0TDPREDVAR analog");
    let recovered = recover_initial_time_dependent_observed_variance(
        loading,
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("eq5 analog");
    assert!((recovered - loading * loading * extra).abs() < 1e-15);
    let ti_observed = recover_initial_time_independent_observed_variance(
        loading,
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("eq5 addedT0TIPREDVAR");
    assert_eq!(ti_observed.to_bits(), recovered.to_bits());
    let initial_observed =
        recover_manifest_observed_variance(loading, 1.6, 0.1).expect("λ² p_0 + θ");
    assert!((initial_observed - recovered).abs() > 1e-3);
    assert_eq!(
        recover_initial_time_dependent_observed_variance(0.0, 0.3, 4.0, LagClock::EventTime)
            .expect("zero loading")
            .to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        refuse_initial_time_dependent_observed_variance_as_initial_time_dependent_variance(
            recovered, extra
        ),
        Err(
            PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialTimeDependentVariance
        )
    );
    assert_eq!(
        refuse_initial_time_dependent_observed_variance_as_initial_observed_variance(
            recovered,
            initial_observed
        ),
        Err(PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialObservedVariance)
    );
    assert_eq!(
        refuse_initial_time_dependent_observed_variance_as_initial_time_independent_observed_variance(
            recovered,
            ti_observed
        ),
        Err(
            PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialTimeIndependentObservedVariance
        )
    );
    assert_eq!(
        refuse_initial_time_dependent_observed_variance_as_measurement_error(recovered, 0.1),
        Err(PsychometricError::InitialTimeDependentObservedVarianceIsNotMeasurementError)
    );
}

#[test]
fn standardised_manifest_trait_variance_refuses_non_event_clocks_and_does_not_keep_zero_variance() {
    assert_eq!(
        recover_standardised_manifest_trait_variance(1.6, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_manifest_trait_variance(0.0, LagClock::EventTime),
        Err(
            PsychometricError::StandardisedManifestTraitVarianceRequiresPositiveManifestTraitVariance
        )
fn initial_time_dependent_observed_variance_refuses_non_event_clocks_and_negative_variance() {
    assert_eq!(
        recover_initial_time_dependent_observed_variance(2.0, 0.3, 4.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_initial_time_dependent_observed_variance(2.0, 0.3, -0.1, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn standardised_manifest_variance_recovers_driver_table_two_correlation() {
    let measurement_error = 0.4_f64;
    let recovered = recover_standardised_manifest_variance(measurement_error, LagClock::EventTime)
        .expect("MANIFESTVARstd");
    assert!((recovered - 1.0).abs() < 1e-15);
    let larger_theta = recover_standardised_manifest_variance(1.6, LagClock::EventTime)
        .expect("MANIFESTVARstd θ=1.6");
    assert_eq!(larger_theta.to_bits(), recovered.to_bits());
    let manifest_trait_std =
        recover_standardised_manifest_trait_variance(measurement_error, LagClock::EventTime)
            .expect("MANIFESTTRAITVARstd");
    assert_eq!(manifest_trait_std.to_bits(), recovered.to_bits());
    let recovered_error = (recovered - 1.0).abs();
    let unstandardised_error = (measurement_error - 1.0).abs();
    assert!(
        recovered_error < unstandardised_error,
        "Driver et al. (2017, Table 2): unstandardised MANIFESTVAR RMSE {unstandardised_error} must exceed MANIFESTVARstd RMSE {recovered_error}"
    );
    let observed = recover_manifest_observed_variance(2.0, 0.4, measurement_error).expect("Var(y)");
    let observed_rmse = (observed - 1.0).abs();
    assert!(
        recovered_error < observed_rmse,
        "Driver et al. (2017, Eq. 5): Var(y) RMSE {observed_rmse} must exceed MANIFESTVARstd RMSE {recovered_error}"
    );
    assert_eq!(
        refuse_unstandardised_manifest_variance_as_standardised_manifest_variance(
            measurement_error,
            recovered
        ),
        Err(PsychometricError::UnstandardisedManifestVarianceIsNotStandardisedManifestVariance)
    );
    assert_eq!(
        refuse_standardised_manifest_trait_variance_as_standardised_manifest_variance(
            manifest_trait_std,
            recovered
        ),
        Err(PsychometricError::StandardisedManifestTraitVarianceIsNotStandardisedManifestVariance)
    );
    assert_eq!(
        refuse_observed_variance_as_standardised_manifest_variance(observed, recovered),
        Err(PsychometricError::ObservedVarianceIsNotStandardisedManifestVariance)
    );
}

#[test]
fn standardised_manifest_variance_refuses_non_event_clocks_and_does_not_keep_zero_variance() {
    assert_eq!(
        recover_standardised_manifest_variance(0.4, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_manifest_variance(0.0, LagClock::EventTime),
        Err(PsychometricError::StandardisedManifestVarianceRequiresPositiveManifestVariance)
    );
}

#[test]
fn standardised_initial_time_independent_effect_recovers_driver_table_three_footnote_four() {
    let initial_variance = 1.6_f64;
    let coefficient = 0.3_f64;
    let predictor_variance = 1.0_f64;
    let recovered = recover_standardised_initial_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        initial_variance,
        LagClock::EventTime,
    )
    .expect("T0TIPREDEFFECTstd");
    assert!(
        (recovered - coefficient * predictor_variance.sqrt() / initial_variance.sqrt()).abs()
            < 1e-15
    );
    let larger_p0 = recover_standardised_initial_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        6.4,
        LagClock::EventTime,
    )
    .expect("T0TIPREDEFFECTstd p_0=6.4");
    assert!((larger_p0 - recovered).abs() > 1e-3);
    let continuous = recover_standardised_continuous_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        0.4,
        -0.5,
        LagClock::EventTime,
    )
    .expect("TIPREDEFFECTstd");
    assert!((continuous - recovered).abs() > 1e-3);
    let asymptotic = recover_standardised_asymptotic_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        0.4,
        -0.5,
        LagClock::EventTime,
    )
    .expect("asymTIPREDEFFECTstd");
    assert!((asymptotic - recovered).abs() > 1e-3);
    let trait_variance = 1.0_f64;
    let total = recover_trait_plus_state_latent_variance(trait_variance, initial_variance)
        .expect("trait+state var");
    let contaminated = coefficient * predictor_variance.sqrt() / total.sqrt();
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_initial_time_independent_effect_as_standardised_initial_time_independent_effect(
            coefficient, recovered
        ),
        Err(PsychometricError::UnstandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect)
    );
    assert_eq!(
        refuse_standardised_continuous_time_independent_effect_as_standardised_initial_time_independent_effect(
            continuous, recovered
        ),
        Err(PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect)
    );
    assert_eq!(
        refuse_standardised_asymptotic_time_independent_effect_as_standardised_initial_time_independent_effect(
            asymptotic, recovered
        ),
        Err(PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect)
    );
    assert_eq!(
        refuse_trait_contaminated_initial_time_independent_effect_as_standardised_initial_time_independent_effect(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, initial_variance),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_continuous_intercept_refuses_non_event_clocks_and_does_not_keep_growing_or_zero_diffusion()
 {
    assert_eq!(
        recover_standardised_continuous_intercept(0.3, 0.4, -0.5, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_continuous_intercept(0.3, 0.4, 0.5, LagClock::EventTime),
        Err(PsychometricError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_standardised_continuous_intercept(0.3, 0.0, -0.5, LagClock::EventTime),
        Err(
            PsychometricError::StandardisedContinuousInterceptRequiresPositiveWithinSubjectVariance
fn standardised_initial_time_independent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances(
) {
fn standardised_initial_time_independent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances()
 {
    assert_eq!(
        recover_standardised_initial_time_independent_predictor_effect(
            0.3,
            1.0,
            1.6,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_initial_time_independent_predictor_effect(
            0.3,
            1.0,
            0.0,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositiveInitialLatentVariance
        )
    );
    assert_eq!(
        recover_standardised_initial_time_independent_predictor_effect(
            0.3,
            0.0,
            1.6,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositivePredictorVariance
        )
    );
}

#[test]
fn standardised_initial_time_dependent_effect_recovers_driver_table_three_footnote_four() {
    let initial_variance = 1.6_f64;
    let coefficient = 0.3_f64;
    let predictor_variance = 1.0_f64;
    let recovered = recover_standardised_initial_time_dependent_predictor_effect(
        coefficient,
        predictor_variance,
        initial_variance,
        LagClock::EventTime,
    )
    .expect("T0TDPREDEFFECTstd");
    assert!(
        (recovered - coefficient * predictor_variance.sqrt() / initial_variance.sqrt()).abs()
            < 1e-15
    );
    let larger_p0 = recover_standardised_initial_time_dependent_predictor_effect(
        coefficient,
        predictor_variance,
        6.4,
        LagClock::EventTime,
    )
    .expect("T0TDPREDEFFECTstd p_0=6.4");
    assert!((larger_p0 - recovered).abs() > 1e-3);
    let same_numbers_ti = recover_standardised_initial_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        initial_variance,
        LagClock::EventTime,
    )
    .expect("T0TIPREDEFFECTstd same numbers");
    assert_eq!(same_numbers_ti.to_bits(), recovered.to_bits());
    let continuous = recover_standardised_continuous_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        0.4,
        -0.5,
        LagClock::EventTime,
    )
    .expect("TIPREDEFFECTstd");
    assert!((continuous - recovered).abs() > 1e-3);
    let trait_variance = 1.0_f64;
    let total = recover_trait_plus_state_latent_variance(trait_variance, initial_variance)
        .expect("trait+state var");
    let contaminated = coefficient * predictor_variance.sqrt() / total.sqrt();
    assert!((contaminated - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect(
            coefficient, recovered
        ),
        Err(PsychometricError::UnstandardisedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect)
    );
    assert_eq!(
        refuse_standardised_initial_time_independent_effect_as_standardised_initial_time_dependent_effect(
            same_numbers_ti, recovered
        ),
        Err(PsychometricError::StandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeDependentEffect)
    );
    assert_eq!(
        refuse_standardised_continuous_time_independent_effect_as_standardised_initial_time_dependent_effect(
            continuous, recovered
        ),
        Err(PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedInitialTimeDependentEffect)
    );
    assert_eq!(
        refuse_trait_contaminated_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect(
            contaminated,
            recovered
        ),
        Err(PsychometricError::TraitContaminatedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(trait_variance, initial_variance),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_initial_time_dependent_effect_refuses_non_event_clocks_and_does_not_keep_zero_variances()
 {
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            1.0,
            1.6,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            1.0,
            0.0,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositiveInitialLatentVariance
        )
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            0.0,
            1.6,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositivePredictorVariance
        )
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            -0.1,
            1.6,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            1.0,
            -0.1,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            f64::NAN,
            1.6,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            0.3,
            1.0,
            f64::NAN,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            f64::NAN,
            1.0,
            1.6,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_initial_time_dependent_predictor_effect(
            4.0,
            1e308,
            1e-308,
            LagClock::EventTime
        ),
fn initial_time_independent_predictor_variance_recovers_driver_added_t0_tipred_var() {
    let coefficient = 0.3_f64;
    let predictor_variance = 4.0_f64;
    let recovered = recover_initial_time_independent_predictor_variance(
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("addedT0TIPREDVAR");
    assert!((recovered - coefficient * coefficient * predictor_variance).abs() < 1e-15);
    let asymptotic = recover_asymptotic_time_independent_predictor_variance(
        coefficient,
        predictor_variance,
        -0.5,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    assert!((asymptotic - recovered).abs() > 1e-3);
    assert_eq!(
        recover_asymptotic_time_independent_predictor_variance(
            coefficient,
            predictor_variance,
            0.5,
            LagClock::EventTime,
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
    let standardised = recover_standardised_initial_time_independent_predictor_effect(
        coefficient,
        predictor_variance,
        1.6,
        LagClock::EventTime,
    )
    .expect("T0TIPREDEFFECTstd");
    assert!((standardised - recovered).abs() > 1e-3);
    assert_eq!(
        recover_initial_time_independent_predictor_variance(0.0, 4.0, LagClock::EventTime)
            .expect("zero")
            .to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        refuse_initial_time_independent_variance_as_asymptotic_time_independent_variance(
            recovered, asymptotic
        ),
        Err(
            PsychometricError::InitialTimeIndependentVarianceIsNotAsymptoticTimeIndependentVariance
        )
    );
    assert_eq!(
        refuse_initial_time_independent_variance_as_standardised_initial_time_independent_effect(
            recovered, standardised
        ),
        Err(
            PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedInitialTimeIndependentEffect
        )
    );
    assert_eq!(
        refuse_initial_time_independent_variance_as_initial_latent_variance(recovered, 1.6),
        Err(PsychometricError::InitialTimeIndependentVarianceIsNotInitialLatentVariance)
    );
    assert_eq!(
        refuse_initial_time_independent_variance_as_trait_variance(recovered, 1.0),
        Err(PsychometricError::InitialTimeIndependentVarianceIsNotTraitVariance)
    );
}

#[test]
fn initial_time_independent_predictor_variance_refuses_non_event_clocks_and_negative_variance() {
    assert_eq!(
        recover_initial_time_independent_predictor_variance(0.3, 4.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_initial_time_independent_predictor_variance(0.3, -0.1, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn initial_time_independent_observed_variance_recovers_driver_eq5_of_added_t0_tipred_var() {
    let loading = 2.0_f64;
    let coefficient = 0.3_f64;
    let predictor_variance = 4.0_f64;
    let extra = recover_initial_time_independent_predictor_variance(
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("addedT0TIPREDVAR");
    let recovered = recover_initial_time_independent_observed_variance(
        loading,
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("eq5 addedT0TIPREDVAR");
    assert!((recovered - loading * loading * extra).abs() < 1e-15);
    let asymptotic_observed = recover_asymptotic_time_independent_observed_variance(
        loading,
        coefficient,
        predictor_variance,
        -0.5,
        LagClock::EventTime,
    )
    .expect("λ² (B/a)² v");
    assert!((asymptotic_observed - recovered).abs() > 1e-3);
    let initial_observed =
        recover_manifest_observed_variance(loading, 1.6, 0.1).expect("λ² p_0 + θ");
    assert!((initial_observed - recovered).abs() > 1e-3);
    assert_eq!(
        recover_initial_time_independent_observed_variance(0.0, 0.3, 4.0, LagClock::EventTime)
            .expect("zero loading")
            .to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        refuse_initial_time_independent_observed_variance_as_initial_time_independent_variance(
            recovered, extra
        ),
        Err(
            PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialTimeIndependentVariance
        )
    );
    assert_eq!(
        refuse_initial_time_independent_observed_variance_as_initial_observed_variance(
            recovered,
            initial_observed
        ),
        Err(PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialObservedVariance)
    );
    assert_eq!(
        refuse_initial_time_independent_observed_variance_as_asymptotic_time_independent_observed_variance(
            recovered,
            asymptotic_observed
        ),
        Err(
            PsychometricError::InitialTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentObservedVariance
        )
    );
    assert_eq!(
        refuse_initial_time_independent_observed_variance_as_measurement_error(recovered, 0.1),
        Err(PsychometricError::InitialTimeIndependentObservedVarianceIsNotMeasurementError)
    );
}

#[test]
fn initial_time_independent_observed_variance_refuses_non_event_clocks_and_negative_variance() {
    assert_eq!(
        recover_initial_time_independent_observed_variance(2.0, 0.3, 4.0, LagClock::SystemTime),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_initial_time_independent_observed_variance(2.0, 0.3, -0.1, LagClock::EventTime),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn asymptotic_time_independent_observed_variance_recovers_driver_eq5_of_added_tipred_var() {
    let loading = 2.0_f64;
    let coefficient = 0.3_f64;
    let predictor_variance = 4.0_f64;
    let log_rate = -0.5_f64;
    let extra = recover_asymptotic_time_independent_predictor_variance(
        coefficient,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let recovered = recover_asymptotic_time_independent_observed_variance(
        loading,
        coefficient,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("eq5 addedTIPREDVAR");
    assert!((recovered - loading * loading * extra).abs() < 1e-15);
    let initial_observed = recover_initial_time_independent_observed_variance(
        loading,
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("eq5 addedT0TIPREDVAR");
    assert!((initial_observed - recovered).abs() > 1e-3);
    let stationary_observed =
        recover_manifest_observed_variance(loading, 1.6, 0.1).expect("λ² p + θ");
    assert!((stationary_observed - recovered).abs() > 1e-3);
    assert_eq!(
        recover_asymptotic_time_independent_observed_variance(
            0.0,
            0.3,
            4.0,
            -0.5,
            LagClock::EventTime
        )
        .expect("zero loading")
        .to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        refuse_asymptotic_time_independent_observed_variance_as_asymptotic_time_independent_variance(
            recovered, extra
        ),
        Err(
            PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentVariance
        )
    );
    assert_eq!(
        refuse_asymptotic_time_independent_observed_variance_as_initial_time_independent_observed_variance(
            recovered,
            initial_observed
        ),
        Err(
            PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotInitialTimeIndependentObservedVariance
        )
    );
    assert_eq!(
        refuse_asymptotic_time_independent_observed_variance_as_stationary_observed_variance(
            recovered,
            stationary_observed
        ),
        Err(PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotStationaryObservedVariance)
    );
    assert_eq!(
        refuse_asymptotic_time_independent_observed_variance_as_measurement_error(recovered, 0.1),
        Err(PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotMeasurementError)
    );
}

#[test]
fn asymptotic_time_independent_observed_variance_refuses_non_event_clocks_and_unstable_drift() {
    assert_eq!(
        recover_asymptotic_time_independent_observed_variance(
            2.0,
            0.3,
            4.0,
            -0.5,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_asymptotic_time_independent_observed_variance(
            2.0,
            0.3,
            -0.1,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_asymptotic_time_independent_observed_variance(
            2.0,
            0.3,
            4.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
}

#[test]
fn standardised_asymptotic_time_independent_variance_recovers_driver_added_tipred_var_std() {
    let coefficient = 0.3_f64;
    let predictor_variance = 4.0_f64;
    let log_rate = -0.5_f64;
    let extra = recover_asymptotic_time_independent_predictor_variance(
        coefficient,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVAR");
    let recovered = recover_standardised_asymptotic_time_independent_predictor_variance(
        coefficient,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("addedTIPREDVARstd");
    assert_eq!(recovered.to_bits(), 1.0_f64.to_bits());
    let doubled = recover_standardised_asymptotic_time_independent_predictor_variance(
        coefficient,
        8.0,
        log_rate,
        LagClock::EventTime,
    )
    .expect("doubled v");
    assert_eq!(doubled.to_bits(), recovered.to_bits());
    let observed = recover_asymptotic_time_independent_observed_variance(
        2.0,
        coefficient,
        predictor_variance,
        log_rate,
        LagClock::EventTime,
    )
    .expect("eq5 addedTIPREDVAR");
    let initial_extra = recover_initial_time_independent_predictor_variance(
        coefficient,
        predictor_variance,
        LagClock::EventTime,
    )
    .expect("addedT0TIPREDVAR");
    assert!((extra - recovered).abs() > 1e-3);
    assert!((observed - recovered).abs() > 1e-3);
    assert!((initial_extra - recovered).abs() > 1e-3);
    assert_eq!(
        refuse_unstandardised_asymptotic_time_independent_variance_as_standardised_asymptotic_time_independent_variance(
            extra, recovered
        ),
        Err(
            PsychometricError::UnstandardisedAsymptoticTimeIndependentVarianceIsNotStandardisedAsymptoticTimeIndependentVariance
        )
    );
    assert_eq!(
        refuse_asymptotic_time_independent_observed_variance_as_standardised_asymptotic_time_independent_variance(
            observed, recovered
        ),
        Err(
            PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotStandardisedAsymptoticTimeIndependentVariance
        )
    );
    assert_eq!(
        refuse_initial_time_independent_variance_as_standardised_asymptotic_time_independent_variance(
            initial_extra, recovered
        ),
        Err(
            PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedAsymptoticTimeIndependentVariance
        )
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(1.0, extra),
        Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
    );
}

#[test]
fn standardised_asymptotic_time_independent_variance_refuses_zero_extra_and_non_event_clocks() {
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_variance(
            0.3,
            4.0,
            -0.5,
            LagClock::SystemTime
        ),
        Err(PsychometricError::EventTimeRequired)
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_variance(
            0.3,
            -0.1,
            -0.5,
            LagClock::EventTime
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_variance(
            0.0,
            4.0,
            -0.5,
            LagClock::EventTime
        ),
        Err(
            PsychometricError::StandardisedAsymptoticTimeIndependentVarianceRequiresPositiveExtraVariance
        )
    );
    assert_eq!(
        recover_standardised_asymptotic_time_independent_predictor_variance(
            0.3,
            4.0,
            0.0,
            LagClock::EventTime
        ),
        Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
    );
}
