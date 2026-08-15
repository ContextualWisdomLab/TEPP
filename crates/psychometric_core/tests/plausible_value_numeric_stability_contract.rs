//! Posterior-draw point-estimate aggregation must remain finite under valid extreme draws.

use psychometric_core::{PsychometricError, posterior_draw_point_estimate_mean};

#[test]
fn scaled_mean_recovers_balanced_extreme_posterior_draws() {
    let mean = posterior_draw_point_estimate_mean(&[
        f64::MAX,
        f64::MAX,
        -f64::MAX,
        -f64::MAX,
    ])
    .expect("balanced finite draws have a finite mean");
    assert_eq!(mean, 0.0);
}

#[test]
fn scaled_mean_preserves_an_extreme_constant_draw() {
    let mean = posterior_draw_point_estimate_mean(&[f64::MAX, f64::MAX])
        .expect("constant finite extreme draws have a finite mean");
    assert_eq!(mean, f64::MAX);
}

#[test]
fn all_zero_draws_have_an_exact_zero_mean() {
    assert_eq!(
        posterior_draw_point_estimate_mean(&[0.0, 0.0, 0.0]).expect("zero draws"),
        0.0
    );
}

#[test]
fn nonfinite_draws_remain_rejected() {
    assert_eq!(
        posterior_draw_point_estimate_mean(&[1.0, f64::INFINITY]),
        Err(PsychometricError::InvalidNumericInput)
    );
}
