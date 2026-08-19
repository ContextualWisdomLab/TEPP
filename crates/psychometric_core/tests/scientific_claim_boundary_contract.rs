//! Scientific claim boundaries for compositional coordinates and posterior draws.

use psychometric_core::{
    ClusteredEventScore, ClusteredScore, IndicatorKind, LagClock, LaggedWithinResidual,
    ordinary_least_squares_slope, posterior_draw_point_estimate_mean,
    recover_cluster_mean_within_between_slopes, recover_discrete_constant_predictor_effect,
    recover_discrete_continuous_intercept_effect, recover_discrete_lagged_latent_covariance,
    recover_discrete_latent_mean, recover_discrete_latent_variance, recover_discrete_observed_mean,
    recover_discrete_process_noise, recover_discrete_time_varying_predictor_effect,
    recover_irregular_centered_residual_log_rate, recover_loading_point_estimate_mean,
    recover_manifest_lagged_observed_covariance, recover_manifest_observed_mean,
    recover_manifest_observed_variance, recover_manifest_trait_plus_state_observed_variance,
    recover_stationary_latent_variance, recover_trait_plus_state_latent_variance,
    recover_within_residual_event_time_log_rate,
    refuse_continuous_intercept_as_discrete_mean_increment,
    refuse_continuous_intercept_as_initial_latent_mean,
    refuse_continuous_intercept_as_manifest_means,
    refuse_finite_interval_process_noise_as_stationary_variance,
    refuse_initial_latent_mean_as_evolved_mean,
    refuse_initial_observed_mean_as_evolved_observed_mean,
    refuse_latent_lagged_covariance_as_observed_covariance, refuse_latent_mean_as_observed_mean,
    refuse_latent_variance_as_observed_variance, refuse_manifest_means_as_observed_mean,
    refuse_manifest_trait_variance_as_measurement_error,
    refuse_measurement_error_as_lagged_observed_covariance,
    refuse_measurement_error_as_observed_variance, refuse_process_noise_as_unconditional_variance,
    refuse_trait_variance_as_process_noise, refuse_trait_variance_as_stationary_within_subject,
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
    assert_eq!(
        recover_discrete_process_noise(1e308, 0.1, 4000.0, LagClock::EventTime),
        Err(psychometric_core::PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_discrete_process_noise(1.0, 1e308, 2.0, LagClock::EventTime),
        Err(psychometric_core::PsychometricError::InvalidNumericInput)
    );
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
    let min_subnormal = f64::from_bits(1);
    let subnormal_ratio =
        recover_stationary_latent_variance(min_subnormal, -min_subnormal, LagClock::EventTime)
            .expect("subnormal ratio");
    assert!((subnormal_ratio - 0.5).abs() < 1e-15);
    assert!(!(f64::MAX / -0.75_f64).is_finite());
    let quotient_overflow =
        recover_stationary_latent_variance(f64::MAX, -0.75, LagClock::EventTime)
            .expect("q/a overflow");
    assert_eq!(quotient_overflow.to_bits(), (f64::MAX / 1.5).to_bits());
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
