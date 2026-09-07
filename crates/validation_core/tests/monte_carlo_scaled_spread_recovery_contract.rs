//! Preserve a finite Monte Carlo spread when raw deviations overflow binary64.
//!
//! Three `f64::MAX` samples and one `-f64::MAX` sample have the exactly
//! representable mean `f64::MAX / 2`, sample standard deviation `f64::MAX`, and
//! standard error `f64::MAX / 2`. The negative raw deviation overflows, so the
//! scaled fallback must recover those representable quantities rather than
//! reject the summary merely because direct subtraction is not representable.

use validation_core::summarize_replications;

#[test]
fn scaled_spread_fallback_preserves_representable_full_range_summary() {
    let summary = summarize_replications(
        &[f64::MAX, f64::MAX, f64::MAX, -f64::MAX],
        0.0,
        1.0,
    )
    .expect("the scaled spread remains representable");

    assert_eq!(summary.replication_count, 4);
    assert_eq!(summary.mean.to_bits(), (f64::MAX / 2.0).to_bits());
    assert_eq!(summary.standard_deviation.to_bits(), f64::MAX.to_bits());
    assert_eq!(
        summary.standard_error.to_bits(),
        (f64::MAX / 2.0).to_bits()
    );
    assert_eq!(summary.percentile_lower.to_bits(), (-f64::MAX).to_bits());
    assert_eq!(summary.percentile_upper.to_bits(), f64::MAX.to_bits());
}
