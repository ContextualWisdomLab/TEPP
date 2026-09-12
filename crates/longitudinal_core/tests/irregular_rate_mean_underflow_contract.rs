//! A nonzero mean irregular-residual log-rate must not collapse to exact zero.

use longitudinal_core::{
    EventTimeInterval, LaggedWithinResidual, LongitudinalError,
    recover_centered_irregular_residual_log_rate,
};

#[test]
fn nonzero_mean_log_rate_that_is_not_binary64_representable_fails_closed() {
    let tiny_interval = EventTimeInterval::new(1.0e307).expect("finite positive event interval");
    let unit_interval = EventTimeInterval::new(1.0).expect("unit interval");
    let earlier = 1.0_f64;
    let later = f64::from_bits(earlier.to_bits() + 1);
    let nonzero_pair = LaggedWithinResidual::new(earlier, later, tiny_interval);

    let one_pair_rate = recover_centered_irregular_residual_log_rate(&[nonzero_pair])
        .expect("the individual positive log-rate is representable");
    assert!(one_pair_rate > 0.0);

    let mut pairs = vec![LaggedWithinResidual::new(1.0, 1.0, unit_interval); 15];
    pairs.push(nonzero_pair);

    assert_eq!(
        recover_centered_irregular_residual_log_rate(&pairs),
        Err(LongitudinalError::InvalidTemporalTransformInput),
        "a mathematically positive mean rate must not be reported as exact no-change"
    );
}
