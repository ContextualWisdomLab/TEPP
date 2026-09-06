//! Characterizes represented-input bias-SE behavior without copying production arithmetic.
//!
//! Issue #491 requires an independent oracle for the admitted production domain.
//! These fixtures use exactly represented integer residuals, compute the pair-distance
//! numerator directly in integer arithmetic, and then assert the public API's audited
//! binary64 result and permutation invariance. Production admission remains `n=4..=16`.

use validation_core::bias_standard_error;

const DIAMETER_INTEGER: i128 = 1_i128 << 53;
const DIAMETER: f64 = (1_u64 << 53) as f64;

const ADMITTED_CASES: [(usize, u128, u64); 13] = [
    (
        4,
        324_518_553_658_426_690_754_359_001_612_291,
        0x4322_79a7_4590_331c,
    ),
    (
        5,
        486_777_830_487_640_036_131_538_502_418_436,
        0x431f_5a7c_ecdb_684a,
    ),
    (
        6,
        649_037_107_316_853_381_508_718_003_224_581,
        0x431a_fc19_d860_6169,
    ),
    (
        7,
        811_296_384_146_066_726_885_897_504_030_726,
        0x4317_9b54_5654_ce5c,
    ),
    (
        8,
        973_555_660_975_280_072_263_077_004_836_871,
        0x4314_f2ec_413c_b52a,
    ),
    (
        9,
        1_135_814_937_804_493_417_640_256_505_643_016,
        0x4312_d071_7a82_a45e,
    ),
    (
        10,
        1_298_074_214_633_706_763_017_436_006_449_161,
        0x4311_1111_1111_1111,
    ),
    (
        11,
        1_460_333_491_462_920_108_394_615_507_255_306,
        0x430f_3940_7aa2_d4ec,
    ),
    (
        12,
        1_622_592_768_292_133_453_771_795_008_061_451,
        0x430c_c40f_740a_8d6c,
    ),
    (
        13,
        1_784_852_045_121_346_799_148_974_508_867_596,
        0x430a_a9db_d5af_20e5,
    ),
    (
        14,
        1_947_111_321_950_560_144_526_154_009_673_741,
        0x4308_d86b_b06d_a1c7,
    ),
    (
        15,
        2_109_370_598_779_773_489_903_333_510_479_886,
        0x4307_4208_c3da_686f,
    ),
    (
        16,
        2_271_629_875_608_986_835_280_513_011_286_031,
        0x4305_dc33_8d87_81a3,
    ),
];

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
fn every_admitted_sample_count_matches_an_independent_integer_pair_oracle() {
    for (sample_count, expected_pair_numerator, expected_bits) in ADMITTED_CASES {
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
fn every_admitted_sample_count_is_permutation_and_reversal_invariant() {
    for (sample_count, _, expected_bits) in ADMITTED_CASES {
        let truth = vec![0.0; sample_count];
        let values = represented_f64_family(sample_count);

        let mut reversed = values.clone();
        reversed.reverse();
        let mut rotated = values.clone();
        rotated.rotate_left((sample_count / 3).max(1));

        for candidate in [&values, &reversed, &rotated] {
            assert_eq!(
                bias_standard_error(&truth, candidate)
                    .expect("permuted represented family has a finite standard error")
                    .to_bits(),
                expected_bits,
                "permutation changed the public result for n={sample_count}"
            );
        }
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
