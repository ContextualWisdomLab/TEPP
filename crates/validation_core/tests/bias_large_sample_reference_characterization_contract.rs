//! Large-sample public reference contracts for future bias-SE exact-proof admission.
//!
//! These fixtures characterize the established fallback authority above the current
//! `n=16` exact-proof admission ceiling. They intentionally do not widen production
//! admission and must not be treated as an independent scientific oracle by
//! themselves. Any future wider exact route must preserve these represented results
//! and refusal semantics before it can replace the fallback for the same domain.

use validation_core::{ValidationError, bias_standard_error};

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
    let gap = (1_u64 << 52) as f64 + 1.0;
    let expected_bits = 0x4280_0200_4008_0101_u64;

    for anchor_index in [0, SAMPLE_COUNT / 2, SAMPLE_COUNT - 1] {
        let (truth, recovered) = two_level_fixture(anchor_index, gap);
        let first = bias_standard_error(&truth, &recovered).expect("large-sample reference result");
        let second = bias_standard_error(&truth, &recovered)
            .expect("deterministic repeated result");
        assert_eq!(first.to_bits(), expected_bits);
        assert_eq!(second.to_bits(), expected_bits);
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
        assert_eq!(
            bias_standard_error(&truth, &recovered),
            Err(ValidationError::InvalidInput)
        );
    }
}
