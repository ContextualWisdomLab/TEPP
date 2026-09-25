//! Finite log-rate pairs must survive overflowing raw ratio intermediates.

use longitudinal_core::{
    EventTimeInterval, LaggedWithinResidual, recover_centered_irregular_residual_log_rate,
};

#[test]
fn representable_mixed_sign_mean_survives_retained_mass_intermediate_overflow() {
    let target_positive_rate = 1.45e308_f64;
    let extreme_interval =
        EventTimeInterval::new(1.0 / target_positive_rate).expect("positive extreme interval");
    let unit_interval = EventTimeInterval::new(1.0).expect("unit interval");

    let pairs = [
        LaggedWithinResidual::new(1.0, std::f64::consts::E, extreme_interval),
        LaggedWithinResidual::new(1.0, std::f64::consts::E, extreme_interval),
        LaggedWithinResidual::new(std::f64::consts::E, 1.0, unit_interval),
        LaggedWithinResidual::new(std::f64::consts::E, 1.0, unit_interval),
    ];

    let positive_rate = std::f64::consts::E.ln() / extreme_interval.as_f64();
    assert!(positive_rate.is_finite());
    assert!(!(positive_rate * 2.0).is_finite());
    let expected = positive_rate / 2.0;

    let recovered = recover_centered_irregular_residual_log_rate(&pairs)
        .expect("finite mathematical mean must not fail on intermediate overflow");
    assert!(recovered.is_finite());
    assert!((recovered - expected).abs() <= expected.abs() * 4.0 * f64::EPSILON);
}
