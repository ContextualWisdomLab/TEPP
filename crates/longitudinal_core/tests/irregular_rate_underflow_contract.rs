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

#[test]
fn equal_residuals_preserve_exact_zero_rate() {
    let event_interval = EventTimeInterval::new(f64::MAX).expect("finite positive event interval");
    let pair = LaggedWithinResidual::new(1.0, 1.0, event_interval);

    let recovered = recover_centered_irregular_residual_log_rate(&[pair])
        .expect("equal residual magnitudes are exact no-change");
    assert_eq!(recovered.to_bits(), 0.0_f64.to_bits());
}
