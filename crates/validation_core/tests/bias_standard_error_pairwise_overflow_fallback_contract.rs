//! Regression contract for bounded exact-proof refusal with representable fallback.
//!
//! The four-observation exact route may refuse when its pairwise reference would
//! materialize `f64::MAX - (-f64::MAX)`. That refusal must not erase a finite
//! standard error that the established scaled fallback can still represent.

use validation_core::bias_standard_error;

const EXPECTED_STANDARD_ERROR_BITS: u64 = 0x7fda_20bd_700c_2c3d;

#[test]
fn overflowing_pair_distance_preserves_representable_generic_fallback() {
    let truth = [0.0; 4];
    let recovered = [f64::MAX, -f64::MAX, 1.0, -1.0];

    let standard_error =
        bias_standard_error(&truth, &recovered).expect("representable standard error");

    assert_eq!(standard_error.to_bits(), EXPECTED_STANDARD_ERROR_BITS);

    let permuted = [1.0, f64::MAX, -1.0, -f64::MAX];
    let permuted_standard_error =
        bias_standard_error(&truth, &permuted).expect("permutation-invariant standard error");
    assert_eq!(
        permuted_standard_error.to_bits(),
        EXPECTED_STANDARD_ERROR_BITS
    );
}
