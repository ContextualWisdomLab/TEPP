use longitudinal_core::{
    EventTimedObservation, LongitudinalError, center_occasion_mean_event_lags,
    recover_occasion_mean_centered_irregular_residual_log_rate,
    recover_within_unit_irregular_residual_log_rate,
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
fn occasion_mean_residual_rate_is_not_cwc_rate_on_the_same_panel() {
    let drift = -0.4_f64;
    let phi = drift.exp();
    let rows = [
        observation(1, 0.0, 1.2),
        observation(1, 1.0, 5.0 + 1.2 * phi),
        observation(1, 2.0, 11.0 + 1.2 * (drift * 2.0).exp()),
        observation(2, 0.0, -0.8),
        observation(2, 1.0, 5.0 - 0.8 * phi),
        observation(2, 2.0, 11.0 - 0.8 * (drift * 2.0).exp()),
    ];

    let occasion = recover_occasion_mean_centered_irregular_residual_log_rate(&rows)
        .expect("occasion-mean residual rate");
    assert!((occasion - drift).abs() < 1.0e-12);

    let cwc = recover_within_unit_irregular_residual_log_rate(&rows).expect("CWC residual rate");
    assert!(
        (cwc - drift).abs() > 1.0e-6,
        "Hamaker Eq. 1a occasion deviations and person-mean CWC residuals are different estimands: occasion={occasion}, CWC={cwc}"
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
fn representable_subnormal_occasion_mean_preserves_round_to_even() {
    let minimum_subnormal = f64::from_bits(1);
    let two_subnormals = f64::from_bits(2);
    let rows = [
        observation(1, 0.0, minimum_subnormal),
        observation(1, 1.0, 0.0),
        observation(2, 0.0, two_subnormals),
        observation(2, 1.0, 0.0),
    ];

    let pairs = center_occasion_mean_event_lags(&rows)
        .expect("subnormal occasion mean must remain representable");
    assert_eq!(pairs.len(), 2);
    assert_eq!(
        pairs[0].earlier_residual().to_bits(),
        (-minimum_subnormal).to_bits(),
        "mean([1 ulp, 2 ulp]) is 1.5 ulp and rounds ties-to-even to 2 ulp"
    );
    assert_eq!(pairs[1].earlier_residual().to_bits(), 0.0_f64.to_bits());
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

#[test]
fn sparse_unaligned_and_nonfinite_occasion_inputs_fail_closed() {
    assert_eq!(
        center_occasion_mean_event_lags(&[]),
        Err(LongitudinalError::InvalidObservationPayload)
    );
    assert_eq!(
        center_occasion_mean_event_lags(&[observation(1, 0.0, 1.0)]),
        Err(LongitudinalError::InvalidObservationPayload)
    );

    let unaligned = [
        observation(1, 0.0, 1.0),
        observation(1, 1.0, 0.5),
        observation(2, 0.1, -1.0),
        observation(2, 1.1, -0.5),
    ];
    assert_eq!(
        center_occasion_mean_event_lags(&unaligned),
        Err(LongitudinalError::InvalidObservationPayload)
    );

    let nonfinite = [
        observation(1, f64::NAN, 1.0),
        observation(1, 1.0, 0.5),
        observation(2, 0.0, -1.0),
        observation(2, 1.0, -0.5),
    ];
    assert_eq!(
        center_occasion_mean_event_lags(&nonfinite),
        Err(LongitudinalError::InvalidObservationPayload)
    );
}

#[test]
fn singleton_wave_unit_does_not_manufacture_or_block_lag_evidence() {
    let drift = -0.5_f64;
    let phi = drift.exp();
    let rows = [
        observation(1, 0.0, 1.0),
        observation(1, 1.0, 4.0 + phi),
        observation(2, 0.0, -1.0),
        observation(2, 1.0, 4.0 - phi),
        observation(3, 0.0, 0.0),
    ];

    let pairs = center_occasion_mean_event_lags(&rows).expect("two lag-contributing units remain");
    assert_eq!(pairs.len(), 2);
    let recovered = recover_occasion_mean_centered_irregular_residual_log_rate(&rows)
        .expect("singleton-wave unit is not a lag contributor");
    assert!((recovered - drift).abs() < 1.0e-12);
}
