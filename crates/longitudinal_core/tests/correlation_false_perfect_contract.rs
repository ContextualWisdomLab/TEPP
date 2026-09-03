//! Strict-interior covariance cannot become a false exact ±1 correlation.

use longitudinal_core::{
    EventTimeInterval, LongitudinalError, recover_event_time_lagged_correlation,
};

#[test]
fn strict_binary_covariance_bound_cannot_round_to_perfect_correlation() {
    let earlier_variance = f64::from_bits(4_607_182_418_800_016_408);
    let later_variance = f64::from_bits(4_607_182_418_800_016_427);
    let covariance = f64::from_bits(4_607_182_418_800_016_417);
    let interval = EventTimeInterval::new(1.0).expect("unit event-time interval");

    // For these exact binary64 inputs covariance² is strictly below
    // earlier_variance * later_variance, but rounded sqrt/division arithmetic
    // produces ±1.0. Reporting either endpoint as perfect association would
    // strengthen the scientific claim beyond the supplied covariance evidence.
    for signed_covariance in [covariance, -covariance] {
        assert_eq!(
            recover_event_time_lagged_correlation(
                signed_covariance,
                earlier_variance,
                later_variance,
                interval,
            ),
            Err(LongitudinalError::InvalidTemporalAssociationInput)
        );
    }
}

#[test]
fn exact_binary_covariance_boundary_cannot_round_below_perfect_correlation() {
    let earlier_variance = 2.0_f64;
    let later_variance = 8.0_f64;
    let covariance = 4.0_f64;
    let interval = EventTimeInterval::new(1.0).expect("unit event-time interval");

    // These represented inputs satisfy covariance² == earlier_variance *
    // later_variance exactly. Rounded square roots make the naive two-step
    // standardization equal to 0x1.fffffffffffffp-1 instead of 1.0. The exact
    // binary covariance relation therefore has to remain authoritative for
    // the endpoint in both sign directions.
    assert_eq!(
        recover_event_time_lagged_correlation(
            covariance,
            earlier_variance,
            later_variance,
            interval,
        ),
        Ok(1.0)
    );
    assert_eq!(
        recover_event_time_lagged_correlation(
            -covariance,
            earlier_variance,
            later_variance,
            interval,
        ),
        Ok(-1.0)
    );
}
