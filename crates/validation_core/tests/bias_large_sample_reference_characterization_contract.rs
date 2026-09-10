//! Large-sample public reference contracts for future bias-SE exact-proof admission.
//!
//! These fixtures characterize the established fallback authority above the current
//! `n=16` exact-proof admission ceiling. They intentionally do not widen production
//! admission and must not be treated as an independent scientific oracle by
//! themselves. Any future wider exact route must preserve these represented results
//! and refusal semantics before it can replace the fallback for the same domain.

use validation_core::bias_standard_error;

const SAMPLE_COUNT: usize = 2_047;

fn two_level_fixture(anchor_index: usize, gap: f64) -> (Vec<f64>, Vec<f64>) {
    let truth = vec![0.0; SAMPLE_COUNT];
    let mut recovered = vec![gap; SAMPLE_COUNT];
    recovered[anchor_index] = 0.0;
    (truth, recovered)
}

#[test]
fn large_sample_two_level_reference_is_bitwise_permutation_invariant() {
    // 2^52 + 1 is exactly representable. With one zero and 2,046 identical
    // nonzero residuals, the exact two-level identity is SE(mean) = |gap| / n.
    // 0x4280_0200_4008_0101 is the nearest binary64 to (2^52 + 1) / 2047.
    let gap = f64::from_bits(0x4330_0000_0000_0001);
    let expected_bits = 0x4280_0200_4008_0101_u64;

    for anchor_index in [0, SAMPLE_COUNT / 2, SAMPLE_COUNT - 1] {
        let (truth, recovered) = two_level_fixture(anchor_index, gap);
        let first = bias_standard_error(&truth, &recovered);
        let second = bias_standard_error(&truth, &recovered);
        assert_eq!(first.map(f64::to_bits), Ok(expected_bits));
        assert_eq!(second.map(f64::to_bits), Ok(expected_bits));
    }
}

#[test]
fn large_sample_non_singleton_two_level_reference_exposes_fallback_rounding_gap() {
    // For m zero residuals and n-m copies of gap,
    // SE(mean)^2 = gap^2 * m(n-m) / [n^2(n-1)]. At n=17, m=5 and
    // gap=2^52+1, exact rational-square-root evaluation rounds to ...a492.
    // The established n>16 fallback currently returns ...a490 instead. Keep
    // that discrepancy explicit as characterization evidence rather than making
    // this test silently widen the exact-proof route to repair itself.
    let sample_count = 17_usize;
    let zero_count = 5_usize;
    let gap = f64::from_bits(0x4330_0000_0000_0001);
    let current_fallback_bits = 0x42fd_294a_104a_a490_u64;
    let exact_target_bits = 0x42fd_294a_104a_a492_u64;
    let truth = vec![0.0; sample_count];
    let mut prefix = vec![gap; sample_count];
    prefix[..zero_count].fill(0.0);
    let mut suffix = prefix.clone();
    suffix.reverse();
    let mut rotated = prefix.clone();
    rotated.rotate_left(sample_count / 3);

    for recovered in [&prefix, &suffix, &rotated] {
        let first = bias_standard_error(&truth, recovered).map(f64::to_bits);
        let second = bias_standard_error(&truth, recovered).map(f64::to_bits);
        assert_eq!(first, Ok(current_fallback_bits));
        assert_eq!(second, first);
        assert_ne!(first, Ok(exact_target_bits));
    }
}

#[test]
fn large_sample_two_level_reference_preserves_false_zero_refusal() {
    // With gap = 2^-1074 and one zero anchor, exact SE(mean) = gap / 2047 is
    // positive but lies below the binary64 half-subnormal boundary. The public
    // fallback must reject that non-representable positive result rather than
    // report a perfect zero, independent of the anchor's position.
    let gap = f64::from_bits(1);

    for anchor_index in [0, SAMPLE_COUNT / 2, SAMPLE_COUNT - 1] {
        let (truth, recovered) = two_level_fixture(anchor_index, gap);
        let result = bias_standard_error(&truth, &recovered);
        assert_eq!(result, Err(validation_core::ValidationError::InvalidInput));
    }
}
