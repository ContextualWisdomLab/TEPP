//! Preserve the exact two-level count geometry at the production `n=16` admission boundary.
//!
//! Six residuals at zero and ten at `next_down(1.0)` give
//! `m(n-m)/(n-1) = 6*10/15 = 4`, so the represented-input standard error of the mean is exactly
//! `|gap|/8`, with public bits `0x3fbf_ffff_ffff_ffff`. Reconstructing the identity through rounded
//! translated moments and `sqrt` would instead round this case to `0.125`. The contract also
//! requires permutation and sign invariance and fails closed when a minimum-subnormal gap would
//! make the exact positive result collapse to false zero. This fixture is inside, and does not
//! widen, the production exact-proof admission budget `n=4..=16`.

use validation_core::{ValidationError, bias_standard_error};

#[test]
fn bias_standard_error_preserves_exact_dyadic_two_level_count_geometry() {
    let repeated = f64::from_bits(0x3fef_ffff_ffff_ffff);
    let recovered = [
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, repeated, repeated, repeated, repeated, repeated, repeated,
        repeated, repeated, repeated, repeated,
    ];

    let standard_error =
        bias_standard_error(&[0.0; 16], &recovered).expect("represented-input standard error");
    // With six observations at one exact residual level and ten at the other,
    // m(n-m)/(n-1) = 6*10/15 = 4. Therefore SE(mean) is exactly |gap|/8.
    // Reconstructing that dyadic identity through translated sums, squares and
    // sqrt rounds next_down(1.0) up to 0.125 instead of preserving gap/8.
    assert_eq!(standard_error.to_bits(), 0x3fbf_ffff_ffff_ffff);

    let permuted = [
        repeated, 0.0, repeated, 0.0, repeated, 0.0, repeated, 0.0, repeated, 0.0, repeated, 0.0,
        repeated, repeated, repeated, repeated,
    ];
    let permuted_standard_error = bias_standard_error(&[0.0; 16], &permuted)
        .expect("permuted represented-input standard error");
    assert_eq!(permuted_standard_error.to_bits(), 0x3fbf_ffff_ffff_ffff);

    let mirrored = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&[0.0; 16], &mirrored)
        .expect("mirrored represented-input standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3fbf_ffff_ffff_ffff);

    let minimum_subnormal = f64::from_bits(1);
    assert_eq!(
        bias_standard_error(
            &[0.0; 16],
            &[
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
            ],
        ),
        Err(ValidationError::InvalidInput)
    );
}
