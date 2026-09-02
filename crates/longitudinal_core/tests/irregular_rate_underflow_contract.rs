use longitudinal_core::{
    EventTimeInterval, LaggedWithinResidual, LongitudinalError,
    recover_centered_irregular_residual_log_rate,
};

#[test]
fn distinct_residuals_with_nonrepresentable_tiny_rate_fail_closed() {
    let event_interval = EventTimeInterval::new(f64::MAX).expect("finite positive event interval");
    let earlier = 1.0_f64;
    let later = f64::from_bits(earlier.to_bits() + 1);
    assert_ne!(earlier.to_bits(), later.to_bits());

    let pair = LaggedWithinResidual::new(earlier, later, event_interval);
    assert_eq!(
        recover_centered_irregular_residual_log_rate(&[pair]),
        Err(LongitudinalError::InvalidTemporalTransformInput)
    );
}
