//! Characterizes represented-input bias-SE behavior without copying production arithmetic.
//!
//! Issue #491 requires an independent oracle for the admitted production domain.
//! These fixtures use exactly represented integer residuals, compute the pair-distance
//! numerator directly in integer arithmetic, and then assert the public API's audited
//! binary64 result and permutation invariance. Production admission remains `n=4..=16`.

use validation_core::bias_standard_error;

const DIAMETER_INTEGER: i128 = 1_i128 << 53;
const DIAMETER: f64 = (1_u64 << 53) as f64;

fn represented_integer_family(sample_count: usize) -> Vec<i128> {
    assert!((4..=16).contains(&sample_count));
    let mut values = Vec::with_capacity(sample_count);
    values.extend([0, 1]);
    values.extend((2..sample_count).map(|_| DIAMETER_INTEGER));
    values
}

fn represented_f64_family(sample_count: usize) -> Vec<f64> {
    represented_integer_family(sample_count)
        .into_iter()
        .map(|value| value as f64)
        .collect()
}

fn exact_integer_pair_square_sum(values: &[i128]) -> u128 {
    let mut pair_square_sum = 0_u128;
    for left in 0..values.len() {
        for right in left + 1..values.len() {
            let distance = (values[left] - values[right]).unsigned_abs();
            pair_square_sum = pair_square_sum
                .checked_add(
                    distance
                        .checked_mul(distance)
                        .expect("bounded fixture square fits u128"),
                )
                .expect("bounded fixture pair numerator fits u128");
        }
    }
    pair_square_sum
}

#[test]
fn admitted_represented_families_match_independent_integer_pair_oracles() {
    let cases = [
        (
            4_usize,
            324_518_553_658_426_690_754_359_001_612_291_u128,
            0x4322_79a7_4590_331c_u64,
        ),
        (
            16_usize,
            2_271_629_875_608_986_835_280_513_011_286_031_u128,
            0x4305_dc33_8d87_81a3_u64,
        ),
    ];

    for (sample_count, expected_pair_numerator, expected_bits) in cases {
        let integer_values = represented_integer_family(sample_count);
        assert_eq!(
            exact_integer_pair_square_sum(&integer_values),
            expected_pair_numerator,
            "independent exact pair-distance oracle changed for n={sample_count}"
        );

        let values = represented_f64_family(sample_count);
        let truth = vec![0.0; sample_count];
        assert_eq!(
            bias_standard_error(&truth, &values)
                .expect("represented family has a finite standard error")
                .to_bits(),
            expected_bits,
            "public exact route must preserve the audited correctly-rounded result for n={sample_count}"
        );
    }
}

#[test]
fn admitted_neutral_zero_route_is_permutation_and_reversal_invariant() {
    let sample_count = 16_usize;
    let expected_bits = 0x4305_dc33_8d87_81a3_u64;
    let truth = vec![0.0; sample_count];
    let values = represented_f64_family(sample_count);

    let mut reversed = values.clone();
    reversed.reverse();
    let mut rotated = values.clone();
    rotated.rotate_left(5);

    for candidate in [&values, &reversed, &rotated] {
        assert_eq!(
            bias_standard_error(&truth, candidate)
                .expect("permuted represented family has a finite standard error")
                .to_bits(),
            expected_bits
        );
    }
}

#[test]
fn neutral_zero_route_recovers_a_represented_geometry_that_direct_pair_subtraction_loses() {
    let represented = [-DIAMETER, 0.0, 1.0, DIAMETER];
    let exact_integers = [-DIAMETER_INTEGER, 0, 1, DIAMETER_INTEGER];

    assert_eq!(
        -DIAMETER - 1.0,
        -DIAMETER,
        "direct binary64 pair subtraction rounds away the unit difference at -2^53"
    );
    assert_eq!(
        exact_integer_pair_square_sum(&exact_integers),
        (1_u128 << 109) + 3,
        "independent integer oracle must retain the unit contribution"
    );

    let truth = [0.0; 4];
    assert_eq!(
        bias_standard_error(&truth, &represented)
            .expect("neutral-zero exact proof recovers the represented geometry")
            .to_bits(),
        0x432a_20bd_700c_2c3e
    );
}
