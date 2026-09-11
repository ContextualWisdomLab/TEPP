//! Characterizes represented-input geometries that need wider exact proof products.
//!
//! These fixtures keep residual construction and every distinct pairwise subtraction
//! exact in binary64 while canonical anchor-relative coefficients make narrow `u128`
//! products overflow before cancellation. The exact pair numerator still fits `u128`.
//! A second boundary shows that the same width pressure reaches the exact
//! candidate/midpoint comparison, so widening only the O(n) numerator identity would
//! not yet establish end-to-end production admission equivalence.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

    fn as_u128(self) -> Option<u128> {
        (self.high == 0).then_some(self.low)
    }
}

fn subtraction_roundoff(recovered: f64, truth: f64, residual: f64) -> f64 {
    let negated_truth = -truth;
    let truth_virtual = residual - recovered;
    let recovered_virtual = residual - truth_virtual;
    let recovered_roundoff = recovered - recovered_virtual;
    let truth_roundoff = negated_truth - truth_virtual;
    recovered_roundoff + truth_roundoff
}

fn is_ieee_zero(value: f64) -> bool {
    value.to_bits() << 1 == 0
}

fn u128_to_binary64(value: u128) -> f64 {
    value
        .to_string()
        .parse::<f64>()
        .expect("u128 decimal representation is finite binary64")
}

fn positive_dyadic(value: f64) -> Option<(u128, i32)> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let bits = value.to_bits();
    let exponent_bits = i32::try_from((bits >> 52) & 0x7ff).ok()?;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    let (mut significand, mut exponent) = if exponent_bits == 0 {
        (u128::from(fraction), -1074)
    } else {
        (
            u128::from((1_u64 << 52) | fraction),
            exponent_bits - 1023 - 52,
        )
    };
    if significand == 0 {
        return None;
    }
    let trailing = significand.trailing_zeros();
    significand >>= trailing;
    exponent += i32::try_from(trailing).ok()?;
    Some((significand, exponent))
}

fn midpoint_dyadic(left: f64, right: f64) -> Option<(u128, i32)> {
    let (left_significand, left_exponent) = positive_dyadic(left)?;
    let (right_significand, right_exponent) = positive_dyadic(right)?;
    let common_exponent = left_exponent.min(right_exponent);
    let left_shift = left_exponent.checked_sub(common_exponent)?.unsigned_abs();
    let right_shift = right_exponent.checked_sub(common_exponent)?.unsigned_abs();
    let left_units = left_significand.checked_mul(1_u128.checked_shl(left_shift)?)?;
    let right_units = right_significand.checked_mul(1_u128.checked_shl(right_shift)?)?;
    let mut midpoint_significand = left_units.checked_add(right_units)?;
    let mut midpoint_exponent = common_exponent.checked_sub(1)?;
    let trailing = midpoint_significand.trailing_zeros();
    midpoint_significand >>= trailing;
    midpoint_exponent += i32::try_from(trailing).ok()?;
    Some((midpoint_significand, midpoint_exponent))
}

fn represented_values(sample_count: usize) -> Vec<f64> {
    assert!(sample_count >= 3);
    let diameter = 9_007_199_254_740_992.0_f64;
    let mut values = Vec::with_capacity(sample_count);
    values.extend([0.0, 1.0]);
    values.extend((2..sample_count).map(|_| diameter));
    values
}

fn canonical_coefficients(sample_count: usize) -> Vec<u128> {
    assert!(sample_count >= 3);
    let diameter = 1_u128 << 53;
    let mut coefficients = Vec::with_capacity(sample_count);
    coefficients.extend([0, 1]);
    coefficients.extend((2..sample_count).map(|_| diameter));
    coefficients
}

fn exact_pair_numerator_for_three_level_fixture(sample_count: usize) -> u128 {
    let repeated = u128::try_from(sample_count - 2).expect("sample count fits u128");
    let diameter = 1_u128 << 53;
    let near_diameter = diameter - 1;
    1_u128
        .checked_add(
            repeated
                .checked_mul(
                    diameter
                        .checked_mul(diameter)
                        .expect("diameter square fits u128")
                        .checked_add(
                            near_diameter
                                .checked_mul(near_diameter)
                                .expect("near-diameter square fits u128"),
                        )
                        .expect("two distinct pair squares fit u128"),
                )
                .expect("pair-count-weighted squares fit u128"),
        )
        .expect("exact pair numerator fits u128")
}

fn assert_represented_subtractions_are_exact(sample_count: usize) {
    let represented = represented_values(sample_count);
    let diameter = 9_007_199_254_740_992.0_f64;

    for value in &represented {
        let residual = *value - 0.0;
        assert!(
            is_ieee_zero(subtraction_roundoff(*value, 0.0, residual)),
            "truth-zero residual construction must be exact"
        );
    }

    for (left, right) in [(0.0, 1.0), (0.0, diameter), (1.0, diameter)] {
        let difference = left - right;
        assert!(
            is_ieee_zero(subtraction_roundoff(left, right, difference)),
            "every distinct represented pair subtraction used by the fixture must be exact"
        );
    }
}

fn canonical_sums(sample_count: usize) -> (u128, u128) {
    let coefficients = canonical_coefficients(sample_count);
    assert_eq!(
        coefficients
            .iter()
            .copied()
            .filter(|coefficient| *coefficient != 0)
            .map(u128::trailing_zeros)
            .min(),
        Some(0),
        "coefficient 1 prevents a removable dyadic scale from hiding width pressure"
    );

    let coefficient_sum = coefficients
        .iter()
        .copied()
        .try_fold(0_u128, u128::checked_add)
        .expect("represented coefficient sum fits u128");
    let square_sum = coefficients
        .iter()
        .copied()
        .try_fold(0_u128, |sum, value| {
            sum.checked_add(value.checked_mul(value)?)
        })
        .expect("represented square sum fits u128");
    (coefficient_sum, square_sum)
}

#[test]
fn represented_pair_admission_can_require_wide_linear_products() {
    const SAMPLE_COUNT: usize = 4_096;
    assert_represented_subtractions_are_exact(SAMPLE_COUNT);
    let (coefficient_sum, square_sum) = canonical_sums(SAMPLE_COUNT);
    let sample_count = u128::try_from(SAMPLE_COUNT).expect("sample count fits u128");

    assert!(
        sample_count.checked_mul(square_sum).is_none(),
        "n*S2 must overflow narrow u128 before cancellation"
    );
    assert!(
        coefficient_sum.checked_mul(coefficient_sum).is_none(),
        "S1^2 must overflow narrow u128 before cancellation"
    );

    let exact_pair_numerator = exact_pair_numerator_for_three_level_fixture(SAMPLE_COUNT);
    assert_eq!(
        exact_pair_numerator,
        664_289_479_338_799_435_974_172_876_300_357_631_u128
    );

    let wide_difference = Wide256::multiply_u128(sample_count, square_sum)
        .checked_sub(Wide256::multiply_u128(coefficient_sum, coefficient_sum))
        .expect("n*S2 must dominate S1^2 exactly")
        .as_u128()
        .expect("the cancellation result remains pair-admissible u128");
    assert_eq!(wide_difference, exact_pair_numerator);

    let denominator = sample_count
        .checked_mul(sample_count)
        .and_then(|value| value.checked_mul(sample_count - 1))
        .expect("unreduced scientific denominator fits u128");
    assert_eq!(denominator, 68_702_699_520);
    assert!(
        denominator <= (1_u128 << 53),
        "this represented width case is not blocked by the exact-denominator gate"
    );
}

#[test]
fn represented_wide_recovery_also_needs_wider_exact_midpoint_products() {
    const SAMPLE_COUNT: usize = 2_050;
    assert_represented_subtractions_are_exact(SAMPLE_COUNT);
    let (coefficient_sum, square_sum) = canonical_sums(SAMPLE_COUNT);
    let sample_count = u128::try_from(SAMPLE_COUNT).expect("sample count fits u128");
    let numerator = exact_pair_numerator_for_three_level_fixture(SAMPLE_COUNT);
    let denominator = sample_count
        .checked_mul(sample_count)
        .and_then(|value| value.checked_mul(sample_count - 1))
        .expect("unreduced scientific denominator fits u128");

    assert_eq!(
        Wide256::multiply_u128(sample_count, square_sum)
            .checked_sub(Wide256::multiply_u128(coefficient_sum, coefficient_sum))
            .expect("wide cancellation remains ordered")
            .as_u128(),
        Some(numerator)
    );
    assert!(sample_count.checked_mul(square_sum).is_none());
    assert!(coefficient_sum.checked_mul(coefficient_sum).is_none());
    assert_eq!(denominator, 8_610_922_500);
    assert!(denominator <= (1_u128 << 53));

    let candidate = (u128_to_binary64(numerator) / u128_to_binary64(denominator)).sqrt();
    assert_eq!(candidate.to_bits(), 0x4296_998e_1aff_78de);
    let (candidate_significand, candidate_exponent) =
        positive_dyadic(candidate).expect("positive candidate is dyadic");
    let candidate_square = candidate_significand
        .checked_mul(candidate_significand)
        .expect("binary64 significand square fits u128");
    let candidate_shift = candidate_exponent
        .checked_mul(-2)
        .and_then(|value| u32::try_from(value).ok())
        .expect("candidate comparison shift is positive and bounded");
    let candidate_factor = 1_u128
        .checked_shl(candidate_shift)
        .expect("candidate comparison factor fits u128");

    assert!(
        denominator.checked_mul(candidate_square).is_none(),
        "the current u128 exact-square comparator cannot form the right operand"
    );
    assert!(
        numerator.checked_mul(candidate_factor).is_none(),
        "the current u128 exact-square comparator cannot form the scaled numerator"
    );
    let wide_candidate_left = Wide256::multiply_u128(numerator, candidate_factor);
    let wide_candidate_right = Wide256::multiply_u128(denominator, candidate_square);
    assert!(
        wide_candidate_left > wide_candidate_right,
        "the exact target lies above the floating candidate square"
    );

    let neighbor = f64::from_bits(candidate.to_bits() + 1);
    let (midpoint_significand, midpoint_exponent) =
        midpoint_dyadic(candidate, neighbor).expect("adjacent midpoint is exact dyadic");
    let midpoint_square = midpoint_significand
        .checked_mul(midpoint_significand)
        .expect("midpoint significand square fits u128");
    let midpoint_shift = midpoint_exponent
        .checked_mul(-2)
        .and_then(|value| u32::try_from(value).ok())
        .expect("midpoint comparison shift is positive and bounded");
    let midpoint_factor = 1_u128
        .checked_shl(midpoint_shift)
        .expect("midpoint comparison factor fits u128");
    let wide_midpoint_left = Wide256::multiply_u128(numerator, midpoint_factor);
    let wide_midpoint_right = Wide256::multiply_u128(denominator, midpoint_square);
    assert!(
        wide_midpoint_left < wide_midpoint_right,
        "the exact target lies below the upward midpoint square, proving the candidate is nearest"
    );
}
