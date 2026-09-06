//! Characterizes a represented-input geometry that needs wider O(n) cancellation products.
//!
//! The fixture keeps every residual and every distinct pairwise subtraction exact in
//! binary64, while canonical anchor-relative coefficients make the narrow `u128`
//! linear identity overflow before cancellation. The exact pair numerator still fits
//! `u128`, so this is a reachable represented-input reason to retain the `Wide256`
//! characterization rather than relying only on synthetic integer coefficients.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

fn represented_values(sample_count: usize) -> Vec<f64> {
    assert!(sample_count >= 3);
    let diameter = (1_u64 << 53) as f64;
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

#[test]
fn represented_pair_admission_can_require_wide_linear_products() {
    const SAMPLE_COUNT: usize = 4_096;
    let represented = represented_values(SAMPLE_COUNT);
    let diameter = (1_u64 << 53) as f64;

    for value in &represented {
        let residual = *value - 0.0;
        assert_eq!(
            subtraction_roundoff(*value, 0.0, residual),
            0.0,
            "truth-zero residual construction must be exact"
        );
    }

    for (left, right) in [(0.0, 1.0), (0.0, diameter), (1.0, diameter)] {
        let difference = left - right;
        assert_eq!(
            subtraction_roundoff(left, right, difference),
            0.0,
            "every distinct represented pair subtraction used by the fixture must be exact"
        );
    }

    let coefficients = canonical_coefficients(SAMPLE_COUNT);
    assert_eq!(
        coefficients
            .iter()
            .copied()
            .filter(|coefficient| *coefficient != 0)
            .map(u128::trailing_zeros)
            .min(),
        Some(0),
        "the coefficient 1 prevents a removable common dyadic scale from hiding width pressure"
    );

    let coefficient_sum = coefficients.iter().copied().try_fold(0_u128, |sum, value| {
        sum.checked_add(value)
    }).expect("represented coefficient sum fits u128");
    let square_sum = coefficients.iter().copied().try_fold(0_u128, |sum, value| {
        sum.checked_add(value.checked_mul(value)?)
    }).expect("represented square sum fits u128");
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
        "this represented width case is not blocked by the current exact-denominator gate"
    );
}
