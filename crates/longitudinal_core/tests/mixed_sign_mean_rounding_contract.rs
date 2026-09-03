use longitudinal_core::{
    EventTimedObservation, center_occasion_mean_event_lags, center_within_unit_event_lags,
};

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
    assert_eq!(
        pairs[1].earlier_residual().to_bits(),
        (-f64::from_bits(10)).to_bits()
    );
    assert_eq!(
        pairs[1].later_residual().to_bits(),
        f64::from_bits(19).to_bits()
    );
}

#[test]
fn mixed_sign_subnormal_occasion_mean_uses_the_same_single_rounding_authority() {
    let ulp = f64::from_bits(1);
    let rows = [
        timed(1, 0.0, -20.0 * ulp),
        timed(2, 0.0, -20.0 * ulp),
        timed(3, 0.0, 9.0 * ulp),
        timed(1, 1.0, 0.0),
        timed(2, 1.0, 0.0),
        timed(3, 1.0, 0.0),
    ];

    let pairs = center_occasion_mean_event_lags(&rows).expect("admissible occasion rows");

    // Occasion t=0 has the same exact mean -31/3 ULPs. The public occasion
    // path must share the CWC numerical authority rather than reintroducing a
    // retained-only mean followed by a second weighting round.
    assert_eq!(
        pairs[0].earlier_residual().to_bits(),
        (-f64::from_bits(10)).to_bits()
    );
    assert_eq!(
        pairs[1].earlier_residual().to_bits(),
        (-f64::from_bits(10)).to_bits()
    );
    assert_eq!(
        pairs[2].earlier_residual().to_bits(),
        f64::from_bits(19).to_bits()
    );
}
