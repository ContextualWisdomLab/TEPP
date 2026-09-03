//! Exact-zero lagged covariance has one public no-association encoding.

use longitudinal_core::{EventTimeInterval, recover_event_time_lagged_correlation};

#[test]
fn exact_zero_covariance_has_one_canonical_no_association_encoding() {
    let interval = EventTimeInterval::new(1.0).expect("unit event-time interval");

    let positive_zero = recover_event_time_lagged_correlation(0.0, 1.0, 4.0, interval)
        .expect("exact zero covariance is a valid no-association result");
    let negative_zero = recover_event_time_lagged_correlation(-0.0, 1.0, 4.0, interval)
        .expect("signed-zero covariance is the same exact no-association result");

    assert_eq!(positive_zero.to_bits(), 0.0_f64.to_bits());
    assert_eq!(negative_zero.to_bits(), 0.0_f64.to_bits());
}
