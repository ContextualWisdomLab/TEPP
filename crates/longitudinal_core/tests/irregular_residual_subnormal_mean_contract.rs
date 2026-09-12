//! Binary64 contract for CWC means at the minimum-subnormal boundary.

use longitudinal_core::{EventTimedObservation, center_within_unit_event_lags};

fn timed(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, score)
}

#[test]
fn representable_subnormal_cwc_mean_preserves_round_to_even() {
    let minimum_subnormal = f64::from_bits(1);
    let two_ulps = f64::from_bits(2);
    let pairs = center_within_unit_event_lags(&[
        timed(1, 0.0, minimum_subnormal),
        timed(1, 1.0, two_ulps),
        timed(2, 0.0, 2.0),
        timed(2, 1.0, 4.0),
    ])
    .expect("both units contribute an admitted event-time lag");

    assert_eq!(pairs.len(), 2);
    assert_eq!(
        pairs[0].earlier_residual().to_bits(),
        (-minimum_subnormal).to_bits(),
        "the exact 1.5-ULP mean must round to the even 2-ULP value"
    );
    assert_eq!(pairs[0].later_residual().to_bits(), 0.0_f64.to_bits());
}
