//! Scalar Driver p.16 `discreteDRIFTstd` contract on event time.

use longitudinal_core::{
    EventTimeInterval, LongitudinalError, recover_event_time_standardised_discrete_drift,
    refuse_trait_plus_state_association_as_standardised_discrete_drift,
    refuse_trait_variance_as_standardisation_variance,
    refuse_unstandardised_discrete_drift_as_standardised_discrete_drift,
};

fn event_time(value: f64) -> EventTimeInterval {
    EventTimeInterval::new(value).expect("test interval must be valid event time")
}

#[test]
fn driver_page_sixteen_scalar_standardised_drift_recovers_on_event_time() {
    let recovered = recover_event_time_standardised_discrete_drift(0.4, -0.5, event_time(1.0))
        .expect("positive stationary within-person variance");
    assert!((recovered - (-0.5_f64).exp()).abs() < 1e-15);

    let longer = recover_event_time_standardised_discrete_drift(0.4, -0.5, event_time(2.5))
        .expect("irregular positive event interval");
    assert!((longer - (-1.25_f64).exp()).abs() < 1e-15);
    assert!(longer < recovered);
}

#[test]
fn known_truth_grid_has_machine_precision_rmse() {
    let cases = [
        (0.1_f64, -0.15_f64, 0.25_f64),
        (0.4, -0.5, 1.0),
        (1.2, -1.25, 0.8),
        (3.0, -0.05, 7.0),
    ];
    let mut squared_error_sum = 0.0;
    for (diffusion, rate, delta) in cases {
        let recovered =
            recover_event_time_standardised_discrete_drift(diffusion, rate, event_time(delta))
                .expect("known-truth case");
        let truth = (rate * delta).exp();
        squared_error_sum += (recovered - truth).powi(2);
    }
    let rmse = (squared_error_sum / cases.len() as f64).sqrt();
    assert!(rmse <= f64::EPSILON);
}

#[test]
fn extreme_stable_rate_preserves_representable_stationary_result() {
    let delta = 1.0 / f64::MAX;
    let recovered =
        recover_event_time_standardised_discrete_drift(f64::MAX, -f64::MAX, event_time(delta))
            .expect("doubling the drift must not create an avoidable overflow");
    let truth = (-f64::MAX * delta).exp();
    assert!((recovered - truth).abs() <= f64::EPSILON);
}

#[test]
fn event_time_exponent_underflow_fails_closed_instead_of_becoming_one() {
    assert_eq!(
        recover_event_time_standardised_discrete_drift(
            f64::MIN_POSITIVE,
            -f64::MIN_POSITIVE,
            event_time(f64::MIN_POSITIVE),
        ),
        Err(LongitudinalError::InvalidTemporalTransformInput)
    );
}

#[test]
fn standardised_drift_fails_closed_without_positive_stationary_within_variance() {
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.0, -0.5, event_time(1.0)),
        Err(LongitudinalError::StandardisedDriftRequiresPositiveWithinVariance)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_drift(
            f64::from_bits(1),
            -1.0e307,
            event_time(1.0e-307),
        ),
        Err(LongitudinalError::StandardisedDriftRequiresPositiveWithinVariance)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.4, 0.0, event_time(1.0)),
        Err(LongitudinalError::StationaryVarianceRequiresStableDrift)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.4, 0.5, event_time(1.0)),
        Err(LongitudinalError::StationaryVarianceRequiresStableDrift)
    );
}

#[test]
fn event_time_value_object_rejects_wrong_clock_payload_shape() {
    assert_eq!(
        EventTimeInterval::new(0.0),
        Err(LongitudinalError::NonPositiveEventInterval)
    );
    assert_eq!(
        EventTimeInterval::new(-1.0),
        Err(LongitudinalError::NonPositiveEventInterval)
    );
    assert_eq!(
        EventTimeInterval::new(f64::NAN),
        Err(LongitudinalError::NonPositiveEventInterval)
    );
    assert_eq!(
        EventTimeInterval::new(f64::INFINITY),
        Err(LongitudinalError::NonPositiveEventInterval)
    );
}

#[test]
fn standardised_drift_rejects_nonfinite_negative_or_unrepresentable_inputs() {
    for diffusion in [f64::NAN, f64::INFINITY, -0.1] {
        assert_eq!(
            recover_event_time_standardised_discrete_drift(diffusion, -0.5, event_time(1.0)),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }
    for rate in [f64::NAN, f64::NEG_INFINITY] {
        assert_eq!(
            recover_event_time_standardised_discrete_drift(0.4, rate, event_time(1.0)),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }
    assert_eq!(
        recover_event_time_standardised_discrete_drift(
            f64::MAX,
            -f64::MIN_POSITIVE,
            event_time(1.0),
        ),
        Err(LongitudinalError::InvalidTemporalTransformInput)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.4, -2.0, event_time(f64::MAX)),
        Err(LongitudinalError::InvalidTemporalTransformInput)
    );
    assert_eq!(
        recover_event_time_standardised_discrete_drift(0.4, -800.0, event_time(1.0)),
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
