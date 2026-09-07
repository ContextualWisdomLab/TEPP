//! Characterizes exact scaled comparisons needed by wider bias-SE proofs.
//!
//! The current bounded production proof forms `u128` candidate-square and midpoint
//! products directly. Represented inputs can keep the exact pair numerator and
//! denominator admissible while those comparison products require more than 128
//! bits. This test-only reference compares two-limb mantissas with signed dyadic
//! exponents without materializing large powers of two, preserving exact ordering
//! across normal and extreme exponent gaps.

use core::cmp::Ordering;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Wide256 {
    high: u128,
    low: u128,
}

impl Wide256 {
    const fn from_u128(value: u128) -> Self {
        Self {
            high: 0,
            low: value,
        }
    }

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

    const fn is_zero(self) -> bool {
        self.high == 0 && self.low == 0
    }

    fn bit_len(self) -> u32 {
        if self.high != 0 {
            128 + (u128::BITS - self.high.leading_zeros())
        } else {
            u128::BITS - self.low.leading_zeros()
        }
    }

    fn bit(self, index: u32) -> bool {
        if index < u128::BITS {
            ((self.low >> index) & 1) != 0
        } else {
            let high_index = index - u128::BITS;
            ((self.high >> high_index) & 1) != 0
        }
    }
}

fn compare_scaled_wide(
    left: Wide256,
    left_exponent: i32,
    right: Wide256,
    right_exponent: i32,
) -> Option<Ordering> {
    match (left.is_zero(), right.is_zero()) {
        (true, true) => return Some(Ordering::Equal),
        (true, false) => return Some(Ordering::Less),
        (false, true) => return Some(Ordering::Greater),
        (false, false) => {}
    }

    let left_bits = left.bit_len();
    let right_bits = right.bit_len();
    let left_top = left_exponent.checked_add(i32::try_from(left_bits.checked_sub(1)?).ok()?)?;
    let right_top = right_exponent.checked_add(i32::try_from(right_bits.checked_sub(1)?).ok()?)?;
    match left_top.cmp(&right_top) {
        Ordering::Less => return Some(Ordering::Less),
        Ordering::Greater => return Some(Ordering::Greater),
        Ordering::Equal => {}
    }

    let width = left_bits.max(right_bits);
    for offset in 0..width {
        let left_bit = offset < left_bits && left.bit(left_bits - 1 - offset);
        let right_bit = offset < right_bits && right.bit(right_bits - 1 - offset);
        match left_bit.cmp(&right_bit) {
            Ordering::Less => return Some(Ordering::Less),
            Ordering::Greater => return Some(Ordering::Greater),
            Ordering::Equal => {}
        }
    }
    Some(Ordering::Equal)
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
    let left_units = left_significand.checked_shl(left_shift)?;
    let right_units = right_significand.checked_shl(right_shift)?;
    let mut midpoint_significand = left_units.checked_add(right_units)?;
    let mut midpoint_exponent = common_exponent.checked_sub(1)?;
    let trailing = midpoint_significand.trailing_zeros();
    midpoint_significand >>= trailing;
    midpoint_exponent += i32::try_from(trailing).ok()?;
    Some((midpoint_significand, midpoint_exponent))
}

fn exact_pair_numerator(sample_count: usize) -> u128 {
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
                        .expect("two pair-square classes fit u128"),
                )
                .expect("pair-count weighted squares fit u128"),
        )
        .expect("pair numerator fits u128")
}

#[test]
fn represented_n2050_candidate_and_midpoint_order_without_large_shift_materialization() {
    const SAMPLE_COUNT: usize = 2_050;
    let sample_count = u128::try_from(SAMPLE_COUNT).expect("sample count fits u128");
    let numerator = exact_pair_numerator(SAMPLE_COUNT);
    let denominator = sample_count
        .checked_mul(sample_count)
        .and_then(|value| value.checked_mul(sample_count - 1))
        .expect("scientific denominator fits u128");
    assert_eq!(numerator, 332_306_998_946_228_931_332_463_617_650_984_961);
    assert_eq!(denominator, 8_610_922_500);

    let candidate = ((numerator as f64) / (denominator as f64)).sqrt();
    assert_eq!(candidate.to_bits(), 0x4296_998e_1aff_78de);
    let (candidate_significand, candidate_exponent) =
        positive_dyadic(candidate).expect("candidate is positive dyadic");
    let candidate_square = candidate_significand
        .checked_mul(candidate_significand)
        .expect("binary64 candidate significand square fits u128");
    assert!(denominator.checked_mul(candidate_square).is_none());
    let candidate_right = Wide256::multiply_u128(denominator, candidate_square);
    assert_ne!(
        candidate_right.high, 0,
        "comparison really needs more than u128"
    );
    assert_eq!(
        compare_scaled_wide(
            Wide256::from_u128(numerator),
            0,
            candidate_right,
            candidate_exponent
                .checked_mul(2)
                .expect("candidate exponent doubles"),
        ),
        Some(Ordering::Greater),
        "exact target lies above the floating candidate square"
    );

    let neighbor = f64::from_bits(candidate.to_bits() + 1);
    let (midpoint_significand, midpoint_exponent) =
        midpoint_dyadic(candidate, neighbor).expect("adjacent midpoint is exact dyadic");
    let midpoint_square = midpoint_significand
        .checked_mul(midpoint_significand)
        .expect("midpoint significand square fits u128");
    assert!(denominator.checked_mul(midpoint_square).is_none());
    let midpoint_right = Wide256::multiply_u128(denominator, midpoint_square);
    assert_ne!(midpoint_right.high, 0, "midpoint comparison exceeds u128");
    assert_eq!(
        compare_scaled_wide(
            Wide256::from_u128(numerator),
            0,
            midpoint_right,
            midpoint_exponent
                .checked_mul(2)
                .expect("midpoint exponent doubles"),
        ),
        Some(Ordering::Less),
        "exact target lies below the upward midpoint square"
    );
}

#[test]
fn scaled_comparison_handles_extreme_exponents_without_allocating_a_power_of_two() {
    let one = Wide256::from_u128(1);
    let two = Wide256::from_u128(2);
    let three = Wide256::from_u128(3);

    assert_eq!(
        compare_scaled_wide(one, -2_148, two, -2_149),
        Some(Ordering::Equal)
    );
    assert_eq!(
        compare_scaled_wide(one, -2_148, three, -2_149),
        Some(Ordering::Less)
    );
    assert_eq!(
        compare_scaled_wide(three, 2_046, one, 2_047),
        Some(Ordering::Greater)
    );
    assert_eq!(
        compare_scaled_wide(Wide256::from_u128(0), -2_148, one, 2_047),
        Some(Ordering::Less)
    );
    assert_eq!(
        compare_scaled_wide(one, 2_047, Wide256::from_u128(0), -2_148),
        Some(Ordering::Greater)
    );
    assert_eq!(
        compare_scaled_wide(Wide256::from_u128(0), 0, Wide256::from_u128(0), 0),
        Some(Ordering::Equal)
    );
}

#[test]
fn full_width_product_keeps_all_256_product_bits() {
    let product = Wide256::multiply_u128(u128::MAX, u128::MAX);
    assert_eq!(product.high, u128::MAX - 1);
    assert_eq!(product.low, 1);
    assert_eq!(product.bit_len(), 256);
    assert!(product.bit(255));
    assert!(product.bit(0));
}
