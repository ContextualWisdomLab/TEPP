use longitudinal_core::{
    LongitudinalError, recover_event_time_standardised_discrete_drift,
    refuse_trait_plus_state_association_as_standardised_discrete_drift,
    refuse_trait_variance_as_standardisation_variance,
    refuse_unstandardised_discrete_drift_as_standardised_discrete_drift,
};

#[test]
fn driver_page_sixteen_scalar_standardised_drift_recovers_on_event_time() {
    let recovered = recover_event_time_standardised_discrete_drift(0.4, -0.5, 1.0)
        .expect("positive stationary within-person variance");
    assert!((recovered - (-0.5_f64).exp()).abs() < 1e-15);

    let longer = recover_event_time_standardised_discrete_drift(0.4, -0.5, 2.5)
        .expect("irregular positive event interval");
    assert!((longer - (-1.25_f64).exp()).abs() < 1e-15);
    assert!((longer - recovered).abs() > 1e-9);
}

#[test]
fn standardised_drift_fails_closed_without_positive_stationary_within_variance() {
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.0, -0.5, 1.0),
        Err(LongitudinalError::StandardisedDriftRequiresPositiveWithinVariance)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.4, 0.0, 1.0),
        Err(LongitudinalError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.4, 0.5, 1.0),
        Err(LongitudinalError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.4, -0.5, 0.0),
        Err(LongitudinalError::NonPositiveEventInterval)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.4, -0.5, f64::NAN),
        Err(LongitudinalError::NonPositiveEventInterval)
    );
}

#[test]
fn standardised_drift_rejects_nonfinite_or_unrepresentable_inputs() {
    for (diffusion, rate, delta) in [
        (f64::NAN, -0.5, 1.0),
        (f64::INFINITY, -0.5, 1.0),
        (-0.1, -0.5, 1.0),
        (0.4, f64::NAN, 1.0),
        (0.4, f64::NEG_INFINITY, 1.0),
    ] {
        assert_eq!(
            recover_event_time_standardised_discrete_drift(diffusion, rate, delta),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }

    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.4, -800.0, 1.0),
        Err(LongitudinalError::InvalidTemporalTransformInput)
    );
}

#[test]
fn equal_scalar_value_does_not_conflate_named_estimands() {
    assert_eq!(
        refuse_unstandardised_discrete_drift_as_standardised_discrete_drift(0.5, 0.5),
        Err(LongitudinalError::UnstandardisedDriftIsNotStandardisedDrift)
    );
    assert_eq!(
        refuse_trait_plus_state_association_as_standardised_discrete_drift(0.5, 0.5),
        Err(LongitudinalError::TraitStateAssociationIsNotStandardisedDrift)
    );
    assert_eq!(
        refuse_trait_variance_as_standardisation_variance(1.0, 0.4),
        Err(LongitudinalError::TraitVarianceIsNotDriftStandardisationVariance)
    );
}
