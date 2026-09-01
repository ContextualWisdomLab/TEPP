//! RED contract for scalar standardised diffusion maps in Longitudinal Modeling.
//!
//! Driver, Oud, and Voelkle (2017) print the underlying continuous/discrete
//! diffusion transformations and describe relevant-variance standardisation,
//! but the 2017 ctsem summary source does not emit named `DIFFUSIONstd` or
//! `discreteDIFFUSIONstd` matrices. These scalar maps therefore remain explicit
//! research-candidate extensions rather than canonical ctsem output.

use longitudinal_core::{
    EventTimeInterval, LongitudinalError,
    recover_event_time_standardised_continuous_diffusion,
    recover_event_time_standardised_discrete_diffusion,
    refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion,
    refuse_total_variance_scaled_diffusion_as_standardised_diffusion,
    refuse_unstandardised_diffusion_as_standardised_diffusion,
};

#[test]
fn continuous_diffusion_candidate_recovers_relevant_variance_ratio() {
    let recovered = recover_event_time_standardised_continuous_diffusion(0.4, -0.25)
        .expect("positive stationary within variance");
    assert!((recovered - 0.5).abs() < 1e-15);

    let scale_invariant = recover_event_time_standardised_continuous_diffusion(1.6, -0.25)
        .expect("same scalar standardisation at a different q scale");
    assert!((scale_invariant - recovered).abs() < 1e-15);

    let max_path = recover_event_time_standardised_continuous_diffusion(f64::MAX, -0.75)
        .expect("representable q/p must not fail on an avoidable intermediate overflow");
    assert!((max_path - 1.5).abs() < 1e-15);
}

#[test]
fn discrete_diffusion_candidate_preserves_event_interval_semantics() {
    let one = EventTimeInterval::new(1.0).expect("positive event interval");
    let two = EventTimeInterval::new(2.0).expect("positive event interval");

    let recovered = recover_event_time_standardised_discrete_diffusion(0.4, -0.25, one)
        .expect("positive stationary within variance");
    let expected = 1.0 - (-0.5_f64).exp();
    assert!((recovered - expected).abs() < 1e-15);

    let later = recover_event_time_standardised_discrete_diffusion(0.4, -0.25, two)
        .expect("later event interval");
    assert!(later > recovered, "stable-process noise fraction must accumulate with event time");
    assert!(later < 1.0);

    let rescaled = recover_event_time_standardised_discrete_diffusion(1.6, -0.25, one)
        .expect("same scalar standardisation at a different q scale");
    assert!((rescaled - recovered).abs() < 1e-15);
}

#[test]
fn standardised_diffusion_candidates_fail_closed_without_positive_stationarity() {
    let one = EventTimeInterval::new(1.0).expect("positive event interval");

    assert_eq!(
        recover_event_time_standardised_continuous_diffusion(0.0, -0.25),
        Err(LongitudinalError::StandardisedDiffusionRequiresPositiveWithinVariance)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_diffusion(0.0, -0.25, one),
        Err(LongitudinalError::StandardisedDiffusionRequiresPositiveWithinVariance)
    );
    assert_eq!(
        recover_event_time_standardised_continuous_diffusion(0.4, 0.0),
        Err(LongitudinalError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_diffusion(0.4, 0.25, one),
        Err(LongitudinalError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_event_time_standardised_continuous_diffusion(f64::NAN, -0.25),
        Err(LongitudinalError::InvalidTemporalTransformInput)
    );
}

#[test]
fn equal_numbers_do_not_collapse_named_diffusion_estimands() {
    let one = EventTimeInterval::new(1.0).expect("positive event interval");
    let continuous = recover_event_time_standardised_continuous_diffusion(0.4, -0.5)
        .expect("continuous candidate");
    let discrete = recover_event_time_standardised_discrete_diffusion(0.4, -0.5, one)
        .expect("discrete candidate");

    assert_eq!(
        refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion(
            continuous, discrete
        ),
        Err(LongitudinalError::ContinuousDiffusionIsNotDiscreteDiffusion)
    );
    assert_eq!(
        refuse_unstandardised_diffusion_as_standardised_diffusion(continuous, continuous),
        Err(LongitudinalError::UnstandardisedDiffusionIsNotStandardisedDiffusion)
    );
    assert_eq!(
        refuse_total_variance_scaled_diffusion_as_standardised_diffusion(discrete, discrete),
        Err(LongitudinalError::TotalVarianceScaledDiffusionIsNotStandardisedDiffusion)
    );
}
