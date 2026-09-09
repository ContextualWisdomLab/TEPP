//! Public contract for raw three-level square-sum roundoff fallback.

use validation_core::bias_standard_error;

#[test]
fn collapsed_subtraction_roundoffs_refuse_inexact_raw_square_sum() {
    let dominant = 2.0_f64.powi(-54);
    let minor = 2.0_f64.powi(-81);
    let recovered = [1.0; 3];

    // Every represented residual high part rounds to `1.0`, while error-free
    // subtraction retains low terms `[0, dominant, minor]`. The bounded
    // represented-input proof therefore refuses and the production bias fallback
    // evaluates this translated three-level geometry.
    //
    // `dominant²`, `minor²`, and `dominant * minor` are individually exact, but
    // `dominant² + minor² = 2^-108 + 2^-162` is not representable because the
    // second term is one quarter of an ulp at `2^-108`. Exact three-level
    // admission must refuse that rounded square sum and preserve the represented
    // sample through the generic translated path.
    let truth = [0.0, -dominant, -minor];
    let first = bias_standard_error(&truth, &recovered)
        .expect("inexact raw square sum must fall back without losing dispersion");
    assert_eq!(first.to_bits(), 0x3c75_5555_5400_0000);

    let permuted_truth = [-minor, 0.0, -dominant];
    let second = bias_standard_error(&permuted_truth, &recovered)
        .expect("permutation preserves the represented low-term geometry");
    assert_eq!(second.to_bits(), first.to_bits());
}
