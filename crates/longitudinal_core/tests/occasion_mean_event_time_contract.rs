use longitudinal_core::{
    EventTimedObservation, LongitudinalError, center_occasion_mean_event_lags,
    recover_occasion_mean_centered_irregular_residual_log_rate,
    refuse_occasion_mean_centered_log_rate_as_within_person_lag,
};

fn observation(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, score)
}

#[test]
fn signed_zero_is_one_numeric_occasion() {
    let drift = -0.5_f64;
    let phi = drift.exp();
    let rows = [
        observation(1, -0.0, 1.0),
        observation(1, 1.0, phi),
        observation(2, 0.0, -1.0),
        observation(2, 1.0, -phi),
    ];

    let pairs = center_occasion_mean_event_lags(&rows).expect("numeric zero is one occasion");
    assert_eq!(pairs.len(), 2);

    let recovered = recover_occasion_mean_centered_irregular_residual_log_rate(&rows)
        .expect("recover occasion-mean residual rate");
    assert!((recovered - drift).abs() < 1.0e-12);
    assert_eq!(
        refuse_occasion_mean_centered_log_rate_as_within_person_lag(recovered),
        Err(LongitudinalError::BetweenIsNotWithinChange)
    );
}

#[test]
fn signed_zero_duplicate_unit_is_rejected_as_one_occasion() {
    let rows = [
        observation(1, -0.0, 1.0),
        observation(1, 0.0, 1.5),
        observation(2, 0.0, -1.0),
        observation(2, 1.0, -0.5),
    ];

    assert_eq!(
        center_occasion_mean_event_lags(&rows),
        Err(LongitudinalError::InvalidObservationPayload)
    );
}

#[test]
fn representable_occasion_mean_is_not_rejected_for_intermediate_sum_overflow() {
    let max = f64::MAX;
    let rows = [
        observation(1, 0.0, 0.75 * max),
        observation(1, 1.0, 1.0),
        observation(2, 0.0, 0.75 * max),
        observation(2, 1.0, 1.0),
        observation(3, 0.0, -0.5 * max),
        observation(3, 1.0, -2.0),
    ];

    let pairs = center_occasion_mean_event_lags(&rows)
        .expect("finite occasion mean must survive same-sign intermediate overflow");
    assert_eq!(pairs.len(), 3);
    assert!(pairs.iter().all(|pair| {
        pair.earlier_residual().is_finite()
            && pair.later_residual().is_finite()
            && pair.event_interval().as_f64().is_finite()
    }));
}

#[test]
fn occasion_mean_is_bit_stable_under_row_permutation() {
    let large = f64::MAX * 0.5;
    let next_one = f64::from_bits(1.0_f64.to_bits() + 1);
    let rows_a = [
        observation(1, 0.0, 1.0),
        observation(1, 1.0, 1.0),
        observation(2, 0.0, next_one),
        observation(2, 1.0, 1.0),
        observation(3, 0.0, large),
        observation(3, 1.0, 1.0),
    ];
    let rows_b = [
        observation(1, 0.0, 1.0),
        observation(1, 1.0, 1.0),
        observation(3, 0.0, large),
        observation(3, 1.0, 1.0),
        observation(2, 0.0, next_one),
        observation(2, 1.0, 1.0),
    ];

    let pairs_a = center_occasion_mean_event_lags(&rows_a).expect("first permutation");
    let pairs_b = center_occasion_mean_event_lags(&rows_b).expect("second permutation");
    assert_eq!(pairs_a, pairs_b);
}
