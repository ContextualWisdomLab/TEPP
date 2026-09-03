//! Binary64 rounding contracts shared by longitudinal centering paths.

use longitudinal_core::{
    EventTimedObservation, center_occasion_mean_event_lags, center_within_unit_event_lags,
};

fn timed(unit: u32, event_time: f64, score_bits: u64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, f64::from_bits(score_bits))
}

#[test]
fn cwc_mean_rounds_halfway_subnormal_to_even_after_normalization() {
    let pairs = center_within_unit_event_lags(&[
        timed(1, 0.0, 1),
        timed(1, 1.0, 14),
        timed(2, 0.0, 4),
        timed(2, 1.0, 12),
    ])
    .expect("both units contribute finite event-time lags");

    assert_eq!(pairs.len(), 2);
    assert_eq!(
        pairs[0].earlier_residual().to_bits(),
        (-f64::from_bits(7)).to_bits(),
        "the exact 7.5-ULP unit mean must round to the even 8-ULP value"
    );
    assert_eq!(pairs[0].later_residual().to_bits(), f64::from_bits(6).to_bits());
}

#[test]
fn occasion_mean_rounds_halfway_subnormal_to_even_after_normalization() {
    let pairs = center_occasion_mean_event_lags(&[
        timed(1, 0.0, 1),
        timed(2, 0.0, 14),
        timed(1, 1.0, 4),
        timed(2, 1.0, 12),
    ])
    .expect("both occasions and units satisfy the longitudinal evidence floor");

    assert_eq!(pairs.len(), 2);
    assert_eq!(
        pairs[0].earlier_residual().to_bits(),
        (-f64::from_bits(7)).to_bits(),
        "the exact 7.5-ULP occasion mean must round to the even 8-ULP value"
    );
    assert_eq!(
        pairs[0].later_residual().to_bits(),
        (-f64::from_bits(4)).to_bits()
    );
}
