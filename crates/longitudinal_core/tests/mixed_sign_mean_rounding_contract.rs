use longitudinal_core::{EventTimedObservation, center_within_unit_event_lags};

fn timed(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, score)
}

#[test]
fn mixed_sign_subnormal_cwc_mean_rounds_once_at_the_final_denominator() {
    let ulp = f64::from_bits(1);
    let rows = [
        timed(1, 0.0, -20.0 * ulp),
        timed(1, 1.0, -20.0 * ulp),
        timed(1, 2.0, 9.0 * ulp),
        timed(2, 0.0, 1.0),
        timed(2, 1.0, 2.0),
    ];

    let pairs = center_within_unit_event_lags(&rows).expect("admissible CWC rows");

    // The exact unit-1 mean is -31/3 minimum-subnormal ULPs, which rounds once
    // to -10 ULPs. The predecessor rounded the retained mixed-sign residual
    // mean first and then weighted it, producing -11 ULPs instead.
    assert_eq!(
        pairs[0].earlier_residual().to_bits(),
        (-f64::from_bits(10)).to_bits()
    );
    assert_eq!(
        pairs[0].later_residual().to_bits(),
        (-f64::from_bits(10)).to_bits()
    );
    assert_eq!(pairs[1].earlier_residual().to_bits(), (-f64::from_bits(10)).to_bits());
    assert_eq!(pairs[1].later_residual().to_bits(), f64::from_bits(19).to_bits());
}
