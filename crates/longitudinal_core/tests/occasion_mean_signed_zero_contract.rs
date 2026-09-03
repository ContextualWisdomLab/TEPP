//! Exact-zero occasion-mean residuals have one public encoding.

use longitudinal_core::{EventTimedObservation, center_occasion_mean_event_lags};

fn timed(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, score)
}

#[test]
fn exact_zero_occasion_mean_residuals_use_one_canonical_public_encoding() {
    let pairs = center_occasion_mean_event_lags(&[
        timed(0, 0.0, -1.0),
        timed(0, 1.0, -0.0),
        timed(0, 2.0, 1.0),
        timed(1, 0.0, 1.0),
        timed(1, 1.0, 0.0),
        timed(1, 2.0, -1.0),
    ])
    .expect("finite aligned occasions admit Hamaker Eq. 1a lags");

    assert_eq!(pairs.len(), 4);
    assert_eq!(pairs[0].earlier_residual().to_bits(), (-1.0_f64).to_bits());
    assert_eq!(pairs[0].later_residual().to_bits(), 0.0_f64.to_bits());
    assert_eq!(pairs[1].earlier_residual().to_bits(), 0.0_f64.to_bits());
    assert_eq!(pairs[1].later_residual().to_bits(), 1.0_f64.to_bits());
    assert_eq!(pairs[2].earlier_residual().to_bits(), 1.0_f64.to_bits());
    assert_eq!(pairs[2].later_residual().to_bits(), 0.0_f64.to_bits());
    assert_eq!(pairs[3].earlier_residual().to_bits(), 0.0_f64.to_bits());
    assert_eq!(pairs[3].later_residual().to_bits(), (-1.0_f64).to_bits());
}
