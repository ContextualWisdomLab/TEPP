//! Preserve all-covered Wilson rounding when exact denominator compensation points upward.
//!
//! The existing partial-denominator contracts cover a downward correction and a
//! no-move case. This represented-input specimen exercises the opposite residual
//! direction: forming `1 + z²` rounds the denominator upward enough that the
//! direct lower endpoint is one ULP too small, so exact residual comparison must
//! select the adjacent larger binary64 value.

use validation_core::wilson_coverage_interval;

#[test]
fn exact_count_positive_residual_moves_the_lower_endpoint_up_one_ulp() {
    let z = f64::from_bits(0x3e46_c000_0000_0000);
    let z2 = z * z;
    assert_eq!(z2.to_bits(), 0x3ca0_2c80_0000_0000);

    let rounded_denominator = 1.0 + z2;
    assert_eq!(rounded_denominator.to_bits(), 0x3ff0_0000_0000_0001);
    assert_eq!(
        (1.0 / rounded_denominator).to_bits(),
        0x3fef_ffff_ffff_fffe,
        "the direct quotient must expose the one-ULP-low predecessor result"
    );

    let truth = [0.0];
    let lower_bounds = [-1.0];
    let upper_bounds = [1.0];
    let (lower, upper) = wilson_coverage_interval(&truth, &lower_bounds, &upper_bounds, z)
        .expect("one covered interval with finite positive z must produce Wilson evidence");

    assert_eq!(lower.to_bits(), 0x3fef_ffff_ffff_ffff);
    assert_eq!(upper.to_bits(), 1.0_f64.to_bits());
}
