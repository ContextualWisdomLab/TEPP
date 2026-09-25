//! Exact-zero CWC residuals have one public encoding.

use longitudinal_core::{EventTimedObservation, center_within_unit_event_lags};

fn timed(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, score)
}

#[test]
fn exact_zero_cwc_residuals_use_one_canonical_public_encoding() {
    let pairs = center_within_unit_event_lags(&[
        timed(0, 0.0, -1.0),
        timed(0, 1.0, -0.0),
        timed(0, 2.0, 1.0),
        timed(1, 0.0, 4.0),
        timed(1, 1.0, 4.0),
        timed(1, 2.0, 4.0),
    ])
    .expect("finite repeated observations admit CWC lags");

    assert_eq!(pairs.len(), 4);
    assert_eq!(pairs[0].earlier_residual().to_bits(), (-1.0_f64).to_bits());
    assert_eq!(pairs[0].later_residual().to_bits(), 0.0_f64.to_bits());
    assert_eq!(pairs[1].earlier_residual().to_bits(), 0.0_f64.to_bits());
    assert_eq!(pairs[1].later_residual().to_bits(), 1.0_f64.to_bits());
    for pair in &pairs[2..] {
        assert_eq!(pair.earlier_residual().to_bits(), 0.0_f64.to_bits());
        assert_eq!(pair.later_residual().to_bits(), 0.0_f64.to_bits());
    }
}
