//! Characterize exact bias-standard-error proof resource and rounding envelopes.
//!
//! The `n > 16` fixtures quantify pair-record growth, checked-`u128` intermediate
//! ceilings, a two-limb `Wide256` recovery reference, and exact binary64 midpoint
//! bounds. They remain characterization evidence rather than an independent
//! scientific oracle. Any wider Draft admission must still refuse when its checked
//! arithmetic proof cannot establish the represented result and must not enable
//! quadratic scratch above the bounded pairwise-reference ceiling.
//!
use validation_core::bias_standard_error;

const SEVENTEEN_OBSERVATION_FIXTURE: [u128; 17] = [
    38_557_579,
    48_779_805,
    63_558_649,
    106_352_599,
    139_863_777,
    142_786_819,
    267_163_239,
    275_103_292,
    375_678_558,
    454_709_869,
    484_300_224,
    623_646_610,
    989_643_121,
    1_027_595_814,
    1_520_220_488,
    1_569_903_156,
    1_805_452_085,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Wide256 {
    high: u128,
    low: u128,
}

impl Wide256 {
    fn multiply_u128(left: u128, right: u128) -> Self {
        let mask = u128::from(u64::MAX);
        let left_limbs = [
            u64::try_from(left & mask).expect("masked low limb fits u64"),
            u64::try_from(left >> 64).expect("high limb fits u64"),
        ];
        let right_limbs = [
            u64::try_from(right & mask).expect("masked low limb fits u64"),
            u64::try_from(right >> 64).expect("high limb fits u64"),
        ];
        let mut limbs = [0_u64; 4];

        for (left_index, left_limb) in left_limbs.iter().copied().enumerate() {
            let mut carry = 0_u128;
            for (right_index, right_limb) in right_limbs.iter().copied().enumerate() {
                let limb_index = left_index + right_index;
                let accumulator = u128::from(left_limb)
                    .checked_mul(u128::from(right_limb))
                    .expect("64-bit limb product fits u128")
                    .checked_add(u128::from(limbs[limb_index]))
                    .expect("schoolbook partial sum fits u128")
                    .checked_add(carry)
                    .expect("schoolbook carry sum fits u128");
                limbs[limb_index] =
                    u64::try_from(accumulator & mask).expect("masked schoolbook limb fits u64");
                carry = accumulator >> 64;
            }
            limbs[left_index + 2] =
                u64::try_from(carry).expect("schoolbook multiplication carry fits u64");
        }

        Self {
            high: u128::from(limbs[2]) | (u128::from(limbs[3]) << 64),
            low: u128::from(limbs[0]) | (u128::from(limbs[1]) << 64),
        }
    }

    fn checked_sub(self, right: Self) -> Option<Self> {
        let (low, borrow) = self.low.overflowing_sub(right.low);
        let high = self
            .high
            .checked_sub(right.high)?
            .checked_sub(u128::from(u8::from(borrow)))?;
        Some(Self { high, low })
    }

    fn checked_shl(self, shift: u32) -> Option<Self> {
        if shift == 0 {
            return Some(self);
        }
        if shift >= 256 {
            return None;
        }
        if shift >= 128 {
            if self.high != 0 {
                return None;
            }
            let high_shift = shift - 128;
            if high_shift == 0 {
                return Some(Self {
                    high: self.low,
                    low: 0,
                });
            }
            if self.low >> (128 - high_shift) != 0 {
                return None;
            }
            return Some(Self {
                high: self.low << high_shift,
                low: 0,
            });
        }
        if self.high >> (128 - shift) != 0 {
            return None;
        }
        Some(Self {
            high: (self.high << shift) | (self.low >> (128 - shift)),
            low: self.low << shift,
        })
    }

    fn as_u128(self) -> Option<u128> {
        (self.high == 0).then_some(self.low)
    }
}

fn deterministic_compact_fixture(sample_count: usize) -> Vec<u128> {
    (0..sample_count)
        .map(|index| {
            let value = u128::try_from(index).expect("fixture index fits u128");
            (value * 1_000_003 + value * value * 97 + 17) % 4_000_000_001
        })
        .collect()
}

fn pair_square_sum_quadratic(values: &[u128]) -> Option<u128> {
    let mut sum = 0_u128;
    for left in 0..values.len() {
        for right in left + 1..values.len() {
            let difference = values[left].abs_diff(values[right]);
            sum = sum.checked_add(difference.checked_mul(difference)?)?;
        }
    }
    Some(sum)
}

fn pair_square_sum_linear(values: &[u128]) -> Option<u128> {
    let minimum = *values.iter().min()?;
    let sample_count = u128::try_from(values.len()).ok()?;
    let common_shift = values
        .iter()
        .filter_map(|value| {
            let coefficient = value.checked_sub(minimum)?;
            (coefficient != 0).then_some(coefficient.trailing_zeros())
        })
        .min()
        .unwrap_or(0);

    let mut coefficient_sum = 0_u128;
    let mut square_sum = 0_u128;
    for value in values {
        let coefficient = value.checked_sub(minimum)? >> common_shift;
        coefficient_sum = coefficient_sum.checked_add(coefficient)?;
        square_sum = square_sum.checked_add(coefficient.checked_mul(coefficient)?)?;
    }
    let normalized_sum = sample_count
        .checked_mul(square_sum)?
        .checked_sub(coefficient_sum.checked_mul(coefficient_sum)?)?;
    let squared_unit = 1_u128.checked_shl(common_shift.checked_mul(2)?)?;
    normalized_sum.checked_mul(squared_unit)
}

fn pair_square_sum_linear_wide_product(values: &[u128]) -> Option<Wide256> {
    let minimum = *values.iter().min()?;
    let sample_count = u128::try_from(values.len()).ok()?;
    let common_shift = values
        .iter()
        .filter_map(|value| {
            let coefficient = value.checked_sub(minimum)?;
            (coefficient != 0).then_some(coefficient.trailing_zeros())
        })
        .min()
        .unwrap_or(0);

    let mut coefficient_sum = 0_u128;
    let mut square_sum = 0_u128;
    for value in values {
        let coefficient = value.checked_sub(minimum)? >> common_shift;
        coefficient_sum = coefficient_sum.checked_add(coefficient)?;
        square_sum = square_sum.checked_add(coefficient.checked_mul(coefficient)?)?;
    }

    let normalized_sum = Wide256::multiply_u128(sample_count, square_sum)
        .checked_sub(Wide256::multiply_u128(coefficient_sum, coefficient_sum))?;
    normalized_sum.checked_shl(common_shift.checked_mul(2)?)
}

fn greatest_common_divisor(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn pair_record_count(sample_count: u128) -> Option<u128> {
    sample_count
        .checked_mul(sample_count.checked_sub(1)?)?
        .checked_div(2)
}

fn worst_case_linear_intermediate(sample_count: u128, diameter: u128) -> Option<u128> {
    sample_count
        .checked_mul(sample_count)?
        .checked_mul(diameter.checked_mul(diameter)?)
}

fn worst_case_pair_square_numerator(sample_count: u128, diameter: u128) -> Option<u128> {
    let split_product =
        (sample_count / 2).checked_mul(sample_count.checked_sub(sample_count / 2)?)?;
    split_product.checked_mul(diameter.checked_mul(diameter)?)
}

fn scientific_denominator(sample_count: u128) -> Option<u128> {
    sample_count
        .checked_mul(sample_count)?
        .checked_mul(sample_count.checked_sub(1)?)
}

#[test]
fn seventeen_observation_fixture_proves_exact_binary64_target() {
    const EXACT_PAIR_SQUARE_SUM: u128 = 92_549_865_125_191_410_206;
    const SCIENTIFIC_DENOMINATOR: u128 = 4_624;
    const REDUCED_NUMERATOR: u128 = 46_274_932_562_595_705_103;
    const REDUCED_DENOMINATOR: u128 = 2_312;
    const MIDPOINT_DENOMINATOR: u128 = 1_u128 << 26;
    const LOWER_MIDPOINT_SIGNIFICAND: u128 = 9_494_210_789_449_009;
    const UPPER_MIDPOINT_SIGNIFICAND: u128 = 9_494_210_789_449_011;

    let pairwise = pair_square_sum_quadratic(&SEVENTEEN_OBSERVATION_FIXTURE)
        .expect("quadratic exact pair sum stays within u128");
    let linear = pair_square_sum_linear(&SEVENTEEN_OBSERVATION_FIXTURE)
        .expect("linear exact identity stays within u128");
    assert_eq!(pairwise, EXACT_PAIR_SQUARE_SUM);
    assert_eq!(linear, pairwise);

    let divisor = greatest_common_divisor(pairwise, SCIENTIFIC_DENOMINATOR);
    assert_eq!(divisor, 2);
    assert_eq!(pairwise / divisor, REDUCED_NUMERATOR);
    assert_eq!(SCIENTIFIC_DENOMINATOR / divisor, REDUCED_DENOMINATOR);

    // Prove the unique nearest binary64 without using the production midpoint
    // helper. The lower and upper exact midpoints around 0x...8e99 are the two
    // odd significands above divided by 2^26. Squaring is monotone here, and the
    // cross-products fit u128, so this independently brackets sqrt(N/D).
    let target_scaled = REDUCED_NUMERATOR
        .checked_mul(MIDPOINT_DENOMINATOR * MIDPOINT_DENOMINATOR)
        .expect("target midpoint comparison fits u128");
    let lower_midpoint_scaled = REDUCED_DENOMINATOR
        .checked_mul(LOWER_MIDPOINT_SIGNIFICAND * LOWER_MIDPOINT_SIGNIFICAND)
        .expect("lower midpoint comparison fits u128");
    let upper_midpoint_scaled = REDUCED_DENOMINATOR
        .checked_mul(UPPER_MIDPOINT_SIGNIFICAND * UPPER_MIDPOINT_SIGNIFICAND)
        .expect("upper midpoint comparison fits u128");
    assert!(lower_midpoint_scaled < target_scaled);
    assert!(target_scaled < upper_midpoint_scaled);

    let truth = [0.0; 17];
    let recovered = SEVENTEEN_OBSERVATION_FIXTURE
        .map(|value| f64::from(u32::try_from(value).expect("fixture value fits u32 exactly")));
    assert_eq!(
        bias_standard_error(&truth, &recovered)
            .expect("proof-driven result remains representable")
            .to_bits(),
        0x41a0_dd77_9ac3_8e99
    );
}

#[test]
fn linear_checked_integer_kernel_matches_pair_reference_when_it_admits() {
    for sample_count in [4_usize, 16, 17, 32, 64, 128, 256] {
        let values = deterministic_compact_fixture(sample_count);
        let pairwise = pair_square_sum_quadratic(&values)
            .expect("compact-grid pair reference stays within u128");
        let linear =
            pair_square_sum_linear(&values).expect("compact-grid linear kernel stays within u128");
        assert_eq!(
            linear, pairwise,
            "linear sufficient proof must preserve the exact pair numerator at n={sample_count}"
        );
    }
}

#[test]
fn linear_checked_integer_kernel_normalizes_a_common_power_of_two_unit() {
    let diameter = 1_u128 << 58;
    let mut values = Vec::with_capacity(65);
    values.push(0);
    values.extend((0..64).map(|_| diameter));

    let pairwise =
        pair_square_sum_quadratic(&values).expect("common-power pair numerator stays within u128");
    assert_eq!(pairwise, 1_u128 << 122);
    assert_eq!(
        pair_square_sum_linear(&values),
        Some(pairwise),
        "a shared 2^58 dyadic unit must be removed before checked O(n) intermediates are judged"
    );
}

#[test]
fn linear_checked_integer_kernel_is_not_admission_equivalent_to_pair_reference() {
    let diameter = (1_u128 << 58) + 1;

    let mut fits_both = Vec::with_capacity(64);
    fits_both.push(0);
    fits_both.extend((0..63).map(|_| diameter));
    let pairwise_64 =
        pair_square_sum_quadratic(&fits_both).expect("64-sample pair numerator stays within u128");
    assert_eq!(
        pair_square_sum_linear(&fits_both),
        Some(pairwise_64),
        "n=64 remains inside the normalized linear intermediate budget"
    );

    let mut pair_only = Vec::with_capacity(65);
    pair_only.push(0);
    pair_only.extend((0..64).map(|_| diameter));
    let pairwise_65 = pair_square_sum_quadratic(&pair_only)
        .expect("65-sample pair numerator still stays within u128");
    let expected_pairwise_65 = 64_u128
        .checked_mul(
            diameter
                .checked_mul(diameter)
                .expect("diameter square fits"),
        )
        .expect("pair numerator fits");
    assert_eq!(pairwise_65, expected_pairwise_65);
    assert_eq!(
        pair_square_sum_linear(&pair_only),
        None,
        "odd diameter prevents dyadic rescaling, so n*sum(c_i^2) overflows before cancellation while the exact pair numerator still fits"
    );
}

#[test]
fn wide_product_reference_recovers_the_pair_only_odd_boundary() {
    let diameter = (1_u128 << 58) + 1;
    let mut pair_only = Vec::with_capacity(65);
    pair_only.push(0);
    pair_only.extend((0..64).map(|_| diameter));

    let pairwise =
        pair_square_sum_quadratic(&pair_only).expect("65-sample pair numerator stays within u128");
    assert_eq!(pair_square_sum_linear(&pair_only), None);
    assert_eq!(
        pair_square_sum_linear_wide_product(&pair_only).and_then(Wide256::as_u128),
        Some(pairwise),
        "two-limb intermediate products must distinguish a narrow-u128 refusal from an exact pair refusal"
    );
}

#[test]
fn wide_product_reference_preserves_full_width_u128_multiplication() {
    let maximum = u128::MAX;
    assert_eq!(
        Wide256::multiply_u128(maximum, maximum),
        Wide256 {
            high: maximum - 1,
            low: 1,
        }
    );
}

#[test]
fn pair_record_counts_are_exact_resource_evidence() {
    assert_eq!(pair_record_count(16), Some(120));
    assert_eq!(pair_record_count(17), Some(136));
    assert_eq!(pair_record_count(2_048), Some(2_096_128));
    assert_eq!(pair_record_count(3_162), Some(4_997_541));
}

#[test]
fn compact_dyadic_linear_intermediate_ceiling_is_2047_samples() {
    let exact_integer_diameter = 1_u128 << 53;
    assert!(worst_case_linear_intermediate(2_047, exact_integer_diameter).is_some());
    assert!(worst_case_linear_intermediate(2_048, exact_integer_diameter).is_none());

    let maximum_safe_denominator = scientific_denominator(2_047).expect("bounded denominator");
    assert!(maximum_safe_denominator < (1_u128 << 53));
}

#[test]
fn exact_pair_square_numerator_has_a_wider_u128_envelope_than_linear_intermediates() {
    let exact_integer_diameter = 1_u128 << 53;
    assert!(worst_case_pair_square_numerator(4_095, exact_integer_diameter).is_some());
    assert!(worst_case_pair_square_numerator(4_096, exact_integer_diameter).is_none());
}

#[test]
fn unreduced_scientific_denominator_crosses_binary64_integer_bound_after_208064() {
    let maximum_exact_binary64_integer = 1_u128 << 53;
    assert!(
        scientific_denominator(208_064).expect("denominator fits u128")
            <= maximum_exact_binary64_integer
    );
    assert!(
        scientific_denominator(208_065).expect("denominator fits u128")
            > maximum_exact_binary64_integer
    );
}
