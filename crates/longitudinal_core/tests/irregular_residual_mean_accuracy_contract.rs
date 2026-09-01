//! Accuracy contract for means of extreme but finite event-time log rates.

use longitudinal_core::{
    EventTimeInterval, LaggedWithinResidual, recover_centered_irregular_residual_log_rate,
};

fn pair(rate: f64, interval: f64) -> LaggedWithinResidual {
    let earlier = 1.0_f64;
    let later = (rate * interval).exp();
    LaggedWithinResidual::new(
        earlier,
        later,
        EventTimeInterval::new(interval).expect("positive event interval"),
    )
}

#[test]
fn compensated_mean_preserves_small_signal_between_extreme_rates() {
    let pairs = [
        pair(1.0e100, 1.0e-100),
        pair(1.0, 1.0),
        pair(-1.0e100, 1.0e-100),
    ];
    let recovered = recover_centered_irregular_residual_log_rate(&pairs)
        .expect("finite extreme-rate mean remains identifiable");
    let expected = 1.0 / 3.0;
    assert!((recovered - expected).abs() <= 1.0e-12);
}
