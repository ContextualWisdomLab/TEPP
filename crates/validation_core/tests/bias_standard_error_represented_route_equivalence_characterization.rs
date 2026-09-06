//! Characterizes represented-input equivalence between pairwise and production neutral-zero bias-SE proofs.
//!
//! This is test-only evidence for issue #491. Production admission remains
//! `n=4..=16`: the neutral-zero O(n) proof is primary, while pairwise O(n²)
//! remains a fail-closed comparison/reference path until represented-input
//! equivalence, exact rounding, resource evidence, and protected-head quality
//! gates are all satisfied.

use validation_core::bias_standard_error;

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

fn multiply_by_power_of_two(value: u128, shift: u32) -> Option<u128> {
    value.checked_mul(1_u128.checked_shl(shift)?)
}

fn represented_values(sample_count: usize) -> Vec<f64> {
    assert!(sample_count >= 3);
    let diameter = (1_u64 << 53) as f64;
    let mut values = Vec::with_capacity(sample_count);
    values.extend([0.0, 1.0]);
    values.extend((2..sample_count).map(|_| diameter));
    values
}

fn pairwise_exact_numerator(values: &[f64]) -> Option<(u128, i32)> {
    let mut unit_exponent = i32::MAX;
    for left in 0..values.len() {
        for right in left + 1..values.len() {
            let difference = values[left] - values[right];
            if !difference.is_finite()
                || subtraction_roundoff(values[left], values[right], difference) != 0.0
            {
                return None;
            }
            if difference != 0.0 {
                unit_exponent = unit_exponent.min(positive_dyadic(difference.abs())?.1);
            }
        }
    }
    if unit_exponent == i32::MAX {
        return Some((0, 0));
    }

    let mut pair_square_sum = 0_u128;
    for left in 0..values.len() {
        for right in left + 1..values.len() {
            let difference = values[left] - values[right];
            if difference == 0.0 {
                continue;
            }
            let (significand, exponent) = positive_dyadic(difference.abs())?;
            let shift = exponent.checked_sub(unit_exponent)?.unsigned_abs();
            let coefficient = multiply_by_power_of_two(significand, shift)?;
            pair_square_sum = pair_square_sum.checked_add(coefficient.checked_mul(coefficient)?)?;
        }
    }
    Some((pair_square_sum, unit_exponent))
}

fn neutral_zero_linear_exact_numerator(values: &[f64]) -> Option<(u128, i32, u128, u128)> {
    let mut unit_exponent = i32::MAX;
    for &coordinate in values {
        if coordinate == 0.0 {
            continue;
        }
        let (_, exponent) = positive_dyadic(coordinate.abs())?;
        unit_exponent = unit_exponent.min(exponent);
    }
    if unit_exponent == i32::MAX {
        return Some((0, 0, 0, 0));
    }

    let mut positive_sum = 0_u128;
    let mut negative_sum = 0_u128;
    let mut square_sum = 0_u128;
    for &coordinate in values {
        if coordinate == 0.0 {
            continue;
        }
        let (significand, exponent) = positive_dyadic(coordinate.abs())?;
        let shift = exponent.checked_sub(unit_exponent)?.unsigned_abs();
        let coefficient = multiply_by_power_of_two(significand, shift)?;
        if coordinate.is_sign_negative() {
            negative_sum = negative_sum.checked_add(coefficient)?;
        } else {
            positive_sum = positive_sum.checked_add(coefficient)?;
        }
        square_sum = square_sum.checked_add(coefficient.checked_mul(coefficient)?)?;
    }

    let signed_sum_magnitude = positive_sum.abs_diff(negative_sum);
    let sample_count = u128::try_from(values.len()).ok()?;
    let numerator = Wide256::multiply_u128(sample_count, square_sum)
        .checked_sub(Wide256::multiply_u128(
            signed_sum_magnitude,
            signed_sum_magnitude,
        ))?
        .as_u128()?;
    Some((
        numerator,
        unit_exponent,
        signed_sum_magnitude,
        square_sum,
    ))
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

#[test]
fn represented_pairwise_and_neutral_zero_routes_share_the_same_exact_ratio() {
    for sample_count in [4_usize, 16, 17, 65, 257, 2_050] {
        let values = represented_values(sample_count);
        for value in values.iter().copied() {
            let residual = value - 0.0;
            assert_eq!(
                subtraction_roundoff(value, 0.0, residual),
                0.0,
                "represented residual construction must remain exact for n={sample_count}"
            );
        }

        let (pair_numerator, pair_exponent) =
            pairwise_exact_numerator(&values).expect("pairwise represented-input authority");
        let (linear_numerator, linear_exponent, signed_sum_magnitude, square_sum) =
            neutral_zero_linear_exact_numerator(&values)
                .expect("neutral-zero represented-input candidate");
        assert_eq!(
            linear_exponent, pair_exponent,
            "common exact unit must agree"
        );
        assert_eq!(
            linear_numerator, pair_numerator,
            "neutral-zero O(n) identity must preserve the O(n²) exact pair numerator for n={sample_count}"
        );

        let sample_count_u128 = u128::try_from(sample_count).expect("sample count fits u128");
        let denominator = sample_count_u128
            .checked_mul(sample_count_u128)
            .and_then(|value| value.checked_mul(sample_count_u128 - 1))
            .expect("scientific denominator fits u128");
        let divisor = gcd(pair_numerator, denominator);
        assert_eq!(
            (
                linear_numerator / divisor,
                denominator / divisor,
                linear_exponent,
            ),
            (
                pair_numerator / divisor,
                denominator / divisor,
                pair_exponent,
            ),
            "both routes must present the exact rounder with the same reduced ratio"
        );

        if sample_count == 2_050 {
            assert!(
                sample_count_u128.checked_mul(square_sum).is_none(),
                "n=2050 must exercise the wider cancellation product"
            );
            assert!(
                signed_sum_magnitude
                    .checked_mul(signed_sum_magnitude)
                    .is_none(),
                "n=2050 must exercise the wider squared-sum product"
            );
            assert_eq!(
                pair_numerator,
                332_306_998_946_228_931_332_463_617_650_984_961_u128
            );
            assert_eq!(denominator, 8_610_922_500);
            assert_eq!(divisor, 1);
            assert_eq!(pair_exponent, 0);
            assert_eq!(
                ((pair_numerator as f64) / (denominator as f64)).sqrt().to_bits(),
                0x4296_998e_1aff_78de,
                "the shared exact ratio must reach the already-authoritative n=2050 exact-rounding fixture"
            );
        }
    }
}

#[test]
fn neutral_zero_linear_route_is_order_invariant() {
    let mut values = represented_values(65);
    let forward = neutral_zero_linear_exact_numerator(&values).expect("forward route");
    values.reverse();
    let reversed = neutral_zero_linear_exact_numerator(&values).expect("reversed route");
    assert_eq!(forward.0, reversed.0);
    assert_eq!(forward.1, reversed.1);
    assert_eq!(
        forward.0,
        pairwise_exact_numerator(&values)
            .expect("reversed pair authority")
            .0
    );
}

#[test]
fn neutral_zero_route_admits_mixed_sign_geometry_when_pairwise_refuses() {
    let diameter = (1_u64 << 53) as f64;
    let values = [-diameter, 0.0, 1.0, diameter];
    assert!(
        pairwise_exact_numerator(&values).is_none(),
        "pairwise subtraction must refuse the represented -2^53 versus +1 difference"
    );

    let (numerator, unit_exponent, signed_sum_magnitude, square_sum) =
        neutral_zero_linear_exact_numerator(&values).expect("neutral-zero exact proof");
    assert_eq!(unit_exponent, 0);
    assert_eq!(signed_sum_magnitude, 1);
    assert_eq!(square_sum, (1_u128 << 107) + 1);
    assert_eq!(numerator, (1_u128 << 109) + 3);

    let truth = [0.0; 4];
    let standard_error = bias_standard_error(&truth, &values).expect("finite standard error");
    assert_eq!(standard_error.to_bits(), 0x432a_20bd_700c_2c3e);
}
