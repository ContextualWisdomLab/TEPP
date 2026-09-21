//! Public contract for raw three-level cross-product roundoff refusal.

use validation_core::bias_standard_error;

#[test]
fn raw_three_level_refuses_inexact_cross_product_after_square_checks() {
    // `minor` is just below 2^-537. Its square rounds to the minimum positive
    // subnormal, and the FMA residual itself rounds to signed zero. `dominant`
    // has an exactly representable square. Their cross-product, however, is a
    // normal binary64 whose exact product error remains representable as
    // -8 minimum-subnormal units. The raw three-level admission must therefore
    // refuse at the cross-product FMA proof rather than treating the rounded
    // product as authoritative.
    let minor = f64::from_bits(0x1e5f_ffff_fffe_7961);
    let dominant = f64::from_bits(0x21f8_0000_0000_0000);
    let truth = [0.0; 3];

    let residuals = [0.0, minor, dominant];
    let standard_error = bias_standard_error(&truth, &residuals)
        .expect("inexact raw cross-product falls back to represented geometry");

    // For [0, x, y], SE(mean)^2 = (x^2 + y^2 - xy) / 9. The exact dyadic
    // sample above rounds to 2^-481 in binary64.
    assert_eq!(standard_error.to_bits(), 0x21e0_0000_0000_0000);

    let permuted = [0.0, dominant, minor];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permuted represented geometry preserves the same fallback");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
