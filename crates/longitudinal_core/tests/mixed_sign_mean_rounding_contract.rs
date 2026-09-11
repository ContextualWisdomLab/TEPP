//! Mixed-sign CWC and occasion means divide by the original sample count.

use longitudinal_core::{
    EventTimeInterval, EventTimedObservation, LaggedWithinResidual,
    center_occasion_mean_event_lags, center_within_unit_event_lags,
    recover_centered_irregular_residual_log_rate,
};

fn timed(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, score)
}

fn lagged(earlier: f64, later: f64, interval: f64) -> LaggedWithinResidual {
    LaggedWithinResidual::new(
        earlier,
        later,
        EventTimeInterval::new(interval).expect("positive event interval"),
    )
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

#[test]
fn repeated_small_opposite_rates_change_the_correctly_rounded_mean() {
    let log_two = -(-0.5_f64).ln_1p();
    let pairs = [
        lagged(1.0, 2.0, log_two / 1.0e16_f64),
        lagged(1.0, 0.5, log_two),
        lagged(1.0, 0.5, log_two),
    ];

    let recovered = recover_centered_irregular_residual_log_rate(&pairs)
        .expect("finite mixed-sign mean remains identifiable");
    let expected = 3_333_333_333_333_332.5_f64;

    assert_eq!(
        recovered.to_bits(),
        expected.to_bits(),
        "two -1 rates are jointly significant and must not be rounded away one at a time against 1e16",
    );
}
