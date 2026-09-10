//! Exact represented-input admission for mean-bias standard error.
//!
//! The general bias implementation remains the fallback authority. Samples with at
//! least three exactly represented residuals first attempt the checked O(n)
//! neutral-zero pair-distance identity. The O(n²) pairwise-difference reference is
//! retained only through sixteen observations when the linear proof refuses. Every
//! admitted ratio is rounded against exact binary64 midpoints without first making
//! a rounded square-root ratio authoritative.

use crate::ValidationError;
use core::cmp::Ordering;

fn subtraction_roundoff(recovered: f64, truth: f64, residual: f64) -> f64 {
    let negated_truth = -truth;
    let truth_virtual = residual - recovered;
    let recovered_virtual = residual - truth_virtual;
    let recovered_roundoff = recovered - recovered_virtual;
    let truth_roundoff = negated_truth - truth_virtual;
    recovered_roundoff + truth_roundoff
}

fn positive_finite_dyadic(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    let (mut significand, mut exponent) = if exponent_bits == 0 {
        (fraction, -1074)
    } else {
        ((1_u64 << 52) | fraction, exponent_bits - 1023 - 52)
    };
    let trailing = significand.trailing_zeros();
    significand >>= trailing;
    exponent += i32::try_from(trailing).expect("u64 trailing-zero count fits i32");
    (significand, exponent)
}

fn positive_dyadic(value: f64) -> Option<(u128, i32)> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let (significand, exponent) = positive_finite_dyadic(value);
    Some((u128::from(significand), exponent))
}

fn multiply_by_power_of_two(value: u128, shift: u32) -> Option<u128> {
    let factor = 1_u128.checked_shl(shift)?;
    value.checked_mul(factor)
}

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

    fn checked_sub(self, right: Self) -> Option<Self> {
        if self < right {
            return None;
        }
        let borrow = u128::from(self.low < right.low);
        Some(Self {
            high: self.high.checked_sub(right.high)?.checked_sub(borrow)?,
            low: self.low.wrapping_sub(right.low),
        })
    }

    const fn to_u128(self) -> Option<u128> {
        if self.high == 0 { Some(self.low) } else { None }
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
    left_exponent: i64,
    right: Wide256,
    right_exponent: i64,
) -> Ordering {
    match (left.is_zero(), right.is_zero()) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        (false, false) => {}
    }

    let left_bits = left.bit_len();
    let right_bits = right.bit_len();
    // The exponents originate in binary64 dyadics, but use i128 for the top-bit
    // comparison so this helper is total even if a future caller widens that
    // exponent domain. Wide256 contributes at most 255 to either exponent.
    let left_top = i128::from(left_exponent) + i128::from(left_bits - 1);
    let right_top = i128::from(right_exponent) + i128::from(right_bits - 1);
    match left_top.cmp(&right_top) {
        Ordering::Less => return Ordering::Less,
        Ordering::Greater => return Ordering::Greater,
        Ordering::Equal => {}
    }

    let width = left_bits.max(right_bits);
    for offset in 0..width {
        let left_bit = offset < left_bits && left.bit(left_bits - 1 - offset);
        let right_bit = offset < right_bits && right.bit(right_bits - 1 - offset);
        match left_bit.cmp(&right_bit) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            Ordering::Equal => {}
        }
    }
    Ordering::Equal
}

fn compare_scaled_ratio_to_dyadic_square(
    numerator: u128,
    numerator_exponent: i64,
    denominator: u128,
    significand: u64,
    exponent: i32,
) -> Ordering {
    // A positive binary64 value has at most 53 significand bits. The midpoint of
    // two adjacent positive binary64 values has at most 54. Both fit u64, so the
    // square fits u128 exactly; multiplying that square by any u128 denominator
    // is total in Wide256. Exponent doubling is performed in i64.
    let significand = u128::from(significand);
    let square = significand * significand;
    let square_exponent = i64::from(exponent) * 2;
    let right = Wide256::multiply_u128(denominator, square);
    compare_scaled_wide(
        Wide256::from_u128(numerator),
        numerator_exponent,
        right,
        square_exponent,
    )
}

fn adjacent_midpoint_dyadic(left: f64, right: f64) -> (u64, i32) {
    // The sole caller supplies `candidate` and its immediate positive finite
    // binary64 neighbor. Adjacent values differ by at most 53 powers of two after
    // trailing-zero reduction, and their exact midpoint has at most 54
    // significand bits, so every operation below is total in u128/u64.
    let (left_significand, left_exponent) = positive_finite_dyadic(left);
    let (right_significand, right_exponent) = positive_finite_dyadic(right);
    let common_exponent = left_exponent.min(right_exponent);
    let left_shift = (left_exponent - common_exponent) as u32;
    let right_shift = (right_exponent - common_exponent) as u32;
    let left_units = u128::from(left_significand) << left_shift;
    let right_units = u128::from(right_significand) << right_shift;
    let mut midpoint_significand = left_units + right_units;
    let mut midpoint_exponent = common_exponent - 1;
    let trailing = midpoint_significand.trailing_zeros();
    midpoint_significand >>= trailing;
    midpoint_exponent += i32::try_from(trailing).expect("u128 trailing-zero count fits i32");
    (
        u64::try_from(midpoint_significand)
            .expect("adjacent binary64 midpoint significand fits u64"),
        midpoint_exponent,
    )
}

fn exact_power_of_two(exponent: i32) -> Option<f64> {
    if (-1022..=1023).contains(&exponent) {
        let biased_exponent = u64::try_from(exponent + 1023).ok()?;
        return Some(f64::from_bits(biased_exponent << 52));
    }
    if (-1074..=-1023).contains(&exponent) {
        let shift = u32::try_from(exponent + 1074).ok()?;
        return Some(f64::from_bits(1_u64 << shift));
    }
    None
}

fn correctly_rounded_scaled_sqrt_ratio(
    numerator: u128,
    denominator: u128,
    unit_exponent: i32,
) -> Option<f64> {
    const MAX_EXACT_BINARY64_DENOMINATOR: u128 = 1_u128 << 53;
    if numerator == 0 || denominator == 0 || denominator > MAX_EXACT_BINARY64_DENOMINATOR {
        return None;
    }
    let unit = exact_power_of_two(unit_exponent)?;
    let denominator_f64 = denominator as f64;
    // The binary64 numerator conversion is only a seed. The returned value is
    // admitted solely after the exact two-limb dyadic-square and midpoint
    // comparisons below. This lets the bounded proof retain exact reduced
    // numerators above 2^53 without pretending that their seed conversion is exact.
    let mut candidate = ((numerator as f64) / denominator_f64).sqrt() * unit;
    if !candidate.is_finite() || candidate <= 0.0 {
        return None;
    }
    let target_exponent = i64::from(unit_exponent) * 2;

    for _ in 0..4 {
        let (candidate_significand, candidate_exponent) = positive_finite_dyadic(candidate);
        let candidate_comparison = compare_scaled_ratio_to_dyadic_square(
            numerator,
            target_exponent,
            denominator,
            candidate_significand,
            candidate_exponent,
        );
        if candidate_comparison == Ordering::Equal {
            return Some(candidate);
        }

        let upward = candidate_comparison == Ordering::Greater;
        let bits = candidate.to_bits();
        let neighbor = if upward {
            f64::from_bits(bits.checked_add(1)?)
        } else {
            if bits == 1 {
                return None;
            }
            f64::from_bits(bits - 1)
        };
        if !neighbor.is_finite() {
            return None;
        }
        let (midpoint_significand, midpoint_exponent) =
            adjacent_midpoint_dyadic(candidate, neighbor);
        let midpoint_comparison = compare_scaled_ratio_to_dyadic_square(
            numerator,
            target_exponent,
            denominator,
            midpoint_significand,
            midpoint_exponent,
        );

        let neighbor_is_closer = if upward {
            midpoint_comparison == Ordering::Greater
        } else {
            midpoint_comparison == Ordering::Less
        };
        if neighbor_is_closer {
            candidate = neighbor;
            continue;
        }
        if midpoint_comparison == Ordering::Equal && candidate.to_bits() & 1 == 1 {
            return Some(neighbor);
        }
        return Some(candidate);
    }
    None
}

fn exact_pairwise_pair_square_sum(residuals: &[f64]) -> Option<(u128, i32)> {
    let pair_count = residuals
        .len()
        .checked_mul(residuals.len().checked_sub(1)?)?
        / 2;
    let mut pair_dyadics = Vec::with_capacity(pair_count);
    let mut unit_exponent = i32::MAX;
    for left in 0..residuals.len() {
        for right in left + 1..residuals.len() {
            let difference = residuals[left] - residuals[right];
            if !difference.is_finite()
                || subtraction_roundoff(residuals[left], residuals[right], difference) != 0.0
            {
                return None;
            }
            if difference == 0.0 {
                pair_dyadics.push(None);
                continue;
            }
            let dyadic = positive_dyadic(difference.abs())?;
            unit_exponent = unit_exponent.min(dyadic.1);
            pair_dyadics.push(Some(dyadic));
        }
    }

    let mut pair_square_sum = 0_u128;
    for dyadic in pair_dyadics.into_iter().flatten() {
        let shift = dyadic.1.checked_sub(unit_exponent)?.unsigned_abs();
        let coefficient = multiply_by_power_of_two(dyadic.0, shift)?;
        let square = coefficient.checked_mul(coefficient)?;
        pair_square_sum = pair_square_sum.checked_add(square)?;
    }
    Some((pair_square_sum, unit_exponent))
}

fn exact_neutral_zero_linear_pair_square_sum(residuals: &[f64]) -> Option<(u128, i32)> {
    // Every finite represented residual is already an exact coordinate relative
    // to neutral zero. First choose the common dyadic unit, then rescan to build
    // the signed first moment and square sum. This keeps the proof O(n) with O(1)
    // proof storage and avoids materializing pair distances.
    let mut unit_exponent = i32::MAX;
    for &coordinate in residuals {
        if coordinate == 0.0 {
            continue;
        }
        let (_, exponent) = positive_dyadic(coordinate.abs())?;
        unit_exponent = unit_exponent.min(exponent);
    }
    if unit_exponent == i32::MAX {
        return Some((0, 0));
    }

    let mut positive_sum = 0_u128;
    let mut negative_sum = 0_u128;
    let mut square_sum = 0_u128;
    for &coordinate in residuals {
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

    let sample_count = u128::try_from(residuals.len()).ok()?;
    let scaled_square_sum = Wide256::multiply_u128(sample_count, square_sum);
    let signed_sum_magnitude = positive_sum.abs_diff(negative_sum);
    let signed_first_moment_square =
        Wide256::multiply_u128(signed_sum_magnitude, signed_sum_magnitude);
    let numerator = scaled_square_sum
        .checked_sub(signed_first_moment_square)?
        .to_u128()?;
    Some((numerator, unit_exponent))
}

fn exact_pair_distance_standard_error(
    truth: &[f64],
    recovered: &[f64],
) -> Option<Result<f64, ValidationError>> {
    // n=2 retains its cheaper identity in `bias.rs`. For n>=3, first attempt the
    // checked O(n) neutral-zero identity. This is an arithmetic-proof admission,
    // not a sample-count staircase: integer width, exponent alignment, denominator
    // reduction, or exact midpoint proof can still refuse and delegate safely.
    if truth.len() != recovered.len() || truth.len() < 3 {
        return None;
    }
    let sample_count = truth.len();

    let mut residuals = Vec::with_capacity(sample_count);
    for index in 0..sample_count {
        let truth_value = truth[index];
        let recovered_value = recovered[index];
        if !truth_value.is_finite() || !recovered_value.is_finite() {
            return None;
        }
        let residual = recovered_value - truth_value;
        if !residual.is_finite()
            || subtraction_roundoff(recovered_value, truth_value, residual) != 0.0
        {
            return None;
        }
        residuals.push(residual);
    }

    // The linear proof is O(n) with O(1) proof storage beyond represented
    // residuals and is therefore eligible at every n>=3. Preserve the O(n²)
    // pairwise implementation only as a bounded comparison/fallback reference for
    // n<=16; a wider linear-proof refusal must never allocate quadratic scratch.
    let (pair_square_sum, unit_exponent) =
        match exact_neutral_zero_linear_pair_square_sum(&residuals) {
            Some(exact) => exact,
            None if sample_count <= 16 => exact_pairwise_pair_square_sum(&residuals)?,
            None => return None,
        };
    if pair_square_sum == 0 {
        return Some(Ok(0.0));
    }

    // For n observations, sum((ri-rj)^2, i<j) / [n²(n-1)] is exactly
    // SE(mean)^2. Reduce that rational before the bounded midpoint proof. The
    // reduced numerator remains an exact u128 authority; its binary64 conversion
    // is only the candidate seed.
    let sample_count_u128 = sample_count as u128;
    let denominator = sample_count_u128
        .checked_mul(sample_count_u128)?
        .checked_mul(sample_count_u128.checked_sub(1)?)?;
    let mut divisor_left = pair_square_sum;
    let mut divisor_right = denominator;
    while divisor_right != 0 {
        let remainder = divisor_left % divisor_right;
        divisor_left = divisor_right;
        divisor_right = remainder;
    }
    let reduced_numerator = pair_square_sum / divisor_left;
    let reduced_denominator = denominator / divisor_left;
    let standard_error =
        correctly_rounded_scaled_sqrt_ratio(reduced_numerator, reduced_denominator, unit_exponent)?;
    Some(Ok(standard_error))
}

/// Standard error of mean signed bias.
///
/// Samples with at least three exactly represented residuals first attempt the
/// checked O(n) neutral-zero pair-distance proof. When that proof refuses, only
/// samples through sixteen observations may use the O(n²) pairwise reference;
/// larger samples delegate directly to the established bias implementation. An
/// exact ratio is admitted only when the bounded dyadic-midpoint rounder proves
/// its binary64 result, so proof failure preserves the existing fail-closed path.
///
/// # Errors
///
/// Returns [`ValidationError`] when the fallback bias implementation rejects the
/// input contract, including mismatched arrays, fewer than two observations, or a
/// non-representable intermediate/result after the exact route refuses.
pub fn bias_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    if let Some(result) = exact_pair_distance_standard_error(truth, recovered) {
        return result;
    }
    crate::bias::bias_standard_error(truth, recovered)
}

#[cfg(test)]
mod tests {
    use super::{
        Wide256, adjacent_midpoint_dyadic, compare_scaled_wide,
        correctly_rounded_scaled_sqrt_ratio, exact_neutral_zero_linear_pair_square_sum,
        exact_pair_distance_standard_error, exact_pairwise_pair_square_sum, exact_power_of_two,
        multiply_by_power_of_two, positive_dyadic,
    };
    use core::cmp::Ordering;

    #[test]
    fn exact_ratio_sqrt_corrects_both_adjacent_rounding_directions() {
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(116, 48, 0)
                .expect("bounded exact ratio")
                .to_bits(),
            0x3ff8_df7d_a2e6_6e88
        );
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(136, 48, 0)
                .expect("bounded exact ratio")
                .to_bits(),
            0x3ffa_ee98_6a40_25f8
        );
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(48, 48, 0), Some(1.0));
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(1_739_374_438_758_325_417, 16, 0)
                .expect("large exact numerator remains bounded by exact midpoint proof")
                .to_bits(),
            0x41b3_a706_d408_9e32
        );
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(155_324_619_328_335_851, 25, 0)
                .expect("five-observation reduced ratio remains bounded")
                .to_bits(),
            0x4192_caf1_6406_5ad0
        );
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(621_603_287_214_182_303, 45, 0)
                .expect("six-observation reduced ratio remains bounded")
                .to_bits(),
            0x419c_057d_42fc_5857
        );
    }

    #[test]
    fn exact_ratio_sqrt_admits_wide_scaled_products_for_represented_n2050_boundary() {
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(
                332_306_998_946_228_931_332_463_617_650_984_961,
                8_610_922_500,
                0,
            )
            .expect("exact represented boundary survives comparison-product width")
            .to_bits(),
            0x4296_998e_1aff_78de
        );
    }

    #[test]
    fn wide_scaled_comparison_covers_full_width_exponents_and_zero_ordering() {
        let zero = Wide256::from_u128(0);
        let one = Wide256::from_u128(1);
        let two = Wide256::from_u128(2);
        let three = Wide256::from_u128(3);
        let product = Wide256::multiply_u128(u128::MAX, u128::MAX);

        assert_eq!(product.high, u128::MAX - 1);
        assert_eq!(product.low, 1);
        assert_eq!(product.bit_len(), 256);
        assert!(product.bit(255));
        assert!(product.bit(0));
        assert_eq!(one.bit_len(), 1);
        assert!(!one.bit(128));

        assert_eq!(compare_scaled_wide(zero, 0, zero, 0), Ordering::Equal);
        assert_eq!(
            compare_scaled_wide(zero, -2_148, one, 2_047),
            Ordering::Less
        );
        assert_eq!(
            compare_scaled_wide(one, 2_047, zero, -2_148),
            Ordering::Greater
        );
        assert_eq!(
            compare_scaled_wide(one, -2_148, two, -2_149),
            Ordering::Equal
        );
        assert_eq!(
            compare_scaled_wide(one, -2_148, three, -2_149),
            Ordering::Less
        );
        assert_eq!(
            compare_scaled_wide(three, 2_046, one, 2_047),
            Ordering::Greater
        );
        assert_eq!(compare_scaled_wide(one, 1, one, 0), Ordering::Greater);
        assert_eq!(compare_scaled_wide(one, 0, one, 1), Ordering::Less);
    }

    #[test]
    fn wide_subtraction_preserves_borrow_and_bounded_downcast() {
        let left = Wide256 { high: 1, low: 0 };
        let right = Wide256::from_u128(1);
        assert_eq!(
            left.checked_sub(right),
            Some(Wide256 {
                high: 0,
                low: u128::MAX,
            })
        );
        assert_eq!(left.checked_sub(left), Some(Wide256::from_u128(0)));
        assert_eq!(right.checked_sub(left), None);
        assert_eq!(Wide256::from_u128(7).to_u128(), Some(7));
        assert_eq!(left.to_u128(), None);
    }

    #[test]
    fn exact_ratio_sqrt_refuses_outside_bounded_proof() {
        let too_large_denominator = (1_u128 << 53) + 1;
        let fraction_denominator = 1_u128 << 52;
        let finite_seed_with_infinite_neighbor = 4 * fraction_denominator - 3;
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(0, 48, 0), None);
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(1, 0, 0), None);
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(1, too_large_denominator, 0),
            None
        );
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(1, 48, 1024), None);
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(1, 48, -1075), None);
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(16, 1, 1023), None);
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(
                finite_seed_with_infinite_neighbor,
                fraction_denominator,
                1023,
            ),
            None
        );
    }

    #[test]
    fn dyadic_helpers_cover_normal_subnormal_and_refusal_boundaries() {
        assert_eq!(positive_dyadic(1.0), Some((1, 0)));
        assert_eq!(positive_dyadic(f64::from_bits(1)), Some((1, -1074)));
        assert_eq!(positive_dyadic(0.0), None);
        assert_eq!(positive_dyadic(-1.0), None);
        assert_eq!(positive_dyadic(f64::INFINITY), None);
        assert_eq!(exact_power_of_two(0), Some(1.0));
        assert_eq!(exact_power_of_two(-1074), Some(f64::from_bits(1)));
        assert_eq!(exact_power_of_two(1024), None);
        assert_eq!(exact_power_of_two(-1075), None);
        assert_eq!(multiply_by_power_of_two(3, 2), Some(12));
        assert_eq!(multiply_by_power_of_two(1, 128), None);
        assert_eq!(multiply_by_power_of_two(u128::MAX, 1), None);
        assert_eq!(
            adjacent_midpoint_dyadic(1.0, f64::from_bits(1.0_f64.to_bits() + 1)),
            (9_007_199_254_740_993, -53)
        );
        assert_eq!(
            adjacent_midpoint_dyadic(f64::from_bits(1), f64::from_bits(2)),
            (3, -1075)
        );
    }

    #[test]
    fn neutral_zero_linear_route_matches_pairwise_on_common_domain() {
        let residuals = [0.0, 1.0, 2.0, 7.0];
        assert_eq!(
            exact_neutral_zero_linear_pair_square_sum(&residuals),
            exact_pairwise_pair_square_sum(&residuals)
        );
    }

    #[test]
    fn pairwise_reference_preserves_duplicate_residual_pairs() {
        let residuals = [0.0, 1.0, 1.0, 2.0];
        assert_eq!(exact_pairwise_pair_square_sum(&residuals), Some((8, 0)));
        assert_eq!(
            exact_pairwise_pair_square_sum(&residuals),
            exact_neutral_zero_linear_pair_square_sum(&residuals)
        );
    }

    #[test]
    fn neutral_zero_linear_route_recovers_pairwise_refusal_geometries() {
        let tiny = 2.0_f64.powi(-54);
        let small = [0.0, 1.0, tiny, 2.0];
        assert_eq!(exact_pairwise_pair_square_sum(&small), None);
        assert!(exact_neutral_zero_linear_pair_square_sum(&small).is_some());

        let diameter = 9_007_199_254_740_992.0_f64;
        let wide = [0.0, 1.0, 2.0, -diameter];
        assert_eq!(exact_pairwise_pair_square_sum(&wide), None);
        let (numerator, unit_exponent) = exact_neutral_zero_linear_pair_square_sum(&wide)
            .expect("zero is an exact non-minimum translation anchor");
        assert_eq!(unit_exponent, 0);
        assert_eq!(numerator, 243_388_915_243_820_099_130_562_543_878_155_u128);

        let no_observed_anchor = [1.0, tiny, 2.0, 3.0];
        assert_eq!(exact_pairwise_pair_square_sum(&no_observed_anchor), None);
        let (numerator, unit_exponent) =
            exact_neutral_zero_linear_pair_square_sum(&no_observed_anchor)
                .expect("neutral zero is exact when no observed residual is a universal anchor");
        assert_eq!(unit_exponent, -54);
        assert_eq!(
            numerator,
            6_490_371_073_168_534_319_490_338_297_741_315_u128
        );
    }

    #[test]
    fn four_observation_identity_is_power_of_two_scale_invariant() {
        let truth = [0.0; 4];
        let recovered = [0.0, 1.0, 2.0, 7.0];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("admitted")
                .expect("representable")
                .to_bits(),
            0x3ff8_df7d_a2e6_6e88
        );

        let unit = 2.0_f64.powi(400);
        let scaled = recovered.map(|value| value * unit);
        let expected = f64::from_bits(0x58f8_df7d_a2e6_6e88);
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &scaled)
                .expect("scaled admitted")
                .expect("scaled representable")
                .to_bits(),
            expected.to_bits()
        );
    }

    #[test]
    fn four_observation_identity_reduces_the_exact_ratio_before_bounded_admission() {
        let truth = [0.0; 4];
        let recovered = [0.0, 14_099_687.0, 16_729_100.0, 94_045_527.0];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("reduced ratio admitted")
                .expect("representable")
                .to_bits(),
            0x4174_46e5_76f8_7445
        );
    }

    #[test]
    fn four_observation_identity_keeps_large_reduced_numerator_in_bounded_proof() {
        let truth = [0.0; 4];
        let recovered = [19_274_968.0, 693_729_138.0, 711_353_557.0, 1_625_519_116.0];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("large reduced ratio admitted")
                .expect("representable")
                .to_bits(),
            0x41b3_a706_d408_9e32
        );
    }

    #[test]
    fn five_observation_identity_keeps_exact_pair_distance_ratio_authoritative() {
        let truth = [0.0; 5];
        let recovered = [
            1_342_748_146.0,
            1_434_848_064.0,
            1_525_257_611.0,
            1_685_877_224.0,
            1_771_341_094.0,
        ];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("five-observation ratio admitted")
                .expect("representable")
                .to_bits(),
            0x4192_caf1_6406_5ad0
        );
    }

    #[test]
    fn six_observation_identity_keeps_exact_pair_distance_ratio_authoritative() {
        let truth = [0.0; 6];
        let recovered = [
            1_120_315_269.0,
            1_513_609_015.0,
            1_569_037_659.0,
            1_789_057_504.0,
            1_807_936_669.0,
            1_914_796_738.0,
        ];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("six-observation ratio admitted")
                .expect("representable")
                .to_bits(),
            0x419c_057d_42fc_5857
        );
    }

    #[test]
    fn proof_driven_linear_identity_and_bounded_pairwise_fallbacks() {
        let truth = [0.0; 4];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[3.0; 4]),
            Some(Ok(0.0))
        );
        assert_eq!(
            exact_pair_distance_standard_error(&truth[..2], &[0.0; 2]),
            None
        );
        assert_eq!(
            exact_pair_distance_standard_error(&truth[..3], &[0.0; 3]),
            Some(Ok(0.0))
        );
        for sample_count in 10..=17 {
            let zeros = vec![0.0; sample_count];
            assert_eq!(
                exact_pair_distance_standard_error(&zeros, &zeros),
                Some(Ok(0.0))
            );
        }

        let narrow_refusal = [0.0, 1.0, 2.0_f64.powi(-200), 2.0];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &narrow_refusal),
            None
        );
        let wide_refusal = vec![2.0_f64.powi(-200); 17];
        let mut wide_refusal_truth = vec![0.0; 17];
        wide_refusal_truth[0] = -1.0;
        assert_eq!(
            exact_pair_distance_standard_error(&wide_refusal_truth, &wide_refusal),
            None
        );

        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[0.0, 1.0, f64::INFINITY, 2.0]),
            None
        );
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[f64::MAX, -f64::MAX, 0.0, 0.0])
                .expect("extreme neutral-zero geometry is exactly admitted")
                .expect("finite standard error remains representable")
                .to_bits(),
            0x7fda_20bd_700c_2c3d
        );

        let tiny = 2.0_f64.powi(-54);
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[0.0, 1.0, tiny, 2.0])
                .expect("neutral-zero linear route admits rounded non-anchor pair")
                .expect("represented result")
                .to_bits(),
            0x3fde_a33e_2c83_c140
        );
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[1.0, tiny, 2.0, 3.0])
                .expect("neutral-zero linear route admits the represented geometry")
                .expect("represented result")
                .to_bits(),
            0x3fe4_a7e9_cb8a_3491
        );
        assert_eq!(
            exact_pair_distance_standard_error(&[1.0, 0.0, 0.0, 0.0], &[tiny, 0.0, 0.0, 0.0]),
            None
        );
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[0.0, 1.0, 2.0, 67_108_864.0])
                .expect("wide neutral-zero geometry remains exactly admitted")
                .expect("finite standard error remains representable")
                .to_bits(),
            0x416f_ffff_f800_0003
        );
    }

    #[test]
    fn seventeen_observation_non_singleton_two_level_identity_is_exact() {
        let truth = [0.0; 17];
        let gap = f64::from_bits(0x4330_0000_0000_0001);
        let mut recovered = [gap; 17];
        recovered[..5].fill(0.0);
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("checked linear proof admits this geometry")
                .expect("exact result is representable")
                .to_bits(),
            0x42fd_294a_104a_a492
        );
    }
}
