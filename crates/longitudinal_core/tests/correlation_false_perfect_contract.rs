use longitudinal_core::{EventTimeInterval, LongitudinalError, recover_event_time_lagged_correlation};

#[test]
fn strict_binary_covariance_bound_cannot_round_to_perfect_correlation() {
    let earlier_variance = f64::from_bits(4_607_182_418_800_016_408);
    let later_variance = f64::from_bits(4_607_182_418_800_016_427);
    let covariance = f64::from_bits(4_607_182_418_800_016_417);
    let interval = EventTimeInterval::new(1.0).expect("unit event-time interval");

    // For these exact binary64 inputs covariance² is strictly below
    // earlier_variance * later_variance, but rounded sqrt/division arithmetic
    // produces 1.0. Reporting that as perfect association would strengthen the
    // scientific claim beyond the supplied covariance evidence.
    assert_eq!(
        recover_event_time_lagged_correlation(
            covariance,
            earlier_variance,
            later_variance,
            interval,
        ),
        Err(LongitudinalError::InvalidTemporalAssociationInput)
    );
}
