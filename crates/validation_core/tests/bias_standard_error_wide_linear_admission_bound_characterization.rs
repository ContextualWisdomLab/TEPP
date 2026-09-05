fn deterministic_compact_fixture(sample_count: usize) -> Vec<u128> {
    (0..sample_count)
        .map(|index| {
            let value = u128::try_from(index).expect("fixture index fits u128");
            (value * 1_000_003 + value * value * 97 + 17) % 4_000_000_001
        })
        .collect()
}

fn boundary_fixture(sample_count: usize, diameter: u128) -> Vec<u128> {
    let mut values = Vec::with_capacity(sample_count);
    values.push(0);
    values.extend((1..sample_count).map(|_| diameter));
    values
}

fn pair_square_sum(values: &[u128]) -> Option<u128> {
    let mut sum = 0_u128;
    for left in 0..values.len() {
        for right in left + 1..values.len() {
            let difference = values[left].abs_diff(values[right]);
            sum = sum.checked_add(difference.checked_mul(difference)?)?;
        }
    }
    Some(sum)
}

fn normalized_linear_terms(values: &[u128]) -> Option<(u128, u128, u32)> {
    let minimum = *values.iter().min()?;
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
    Some((coefficient_sum, square_sum, common_shift))
}

fn assert_pair_admission_bounds_linear_accumulators(values: &[u128]) {
    let pair_sum = pair_square_sum(values).expect("fixture pair numerator fits u128");
    let (coefficient_sum, square_sum, common_shift) =
        normalized_linear_terms(values).expect("pair-admitted normalized terms fit u128");
    let squared_shift = common_shift.checked_mul(2).expect("dyadic square shift fits u32");
    let normalized_pair_sum = pair_sum >> squared_shift;
    let restored_pair_sum = normalized_pair_sum
        .checked_shl(squared_shift)
        .expect("fixture normalized pair numerator restores exactly");

    assert_eq!(restored_pair_sum, pair_sum);
    assert!(
        coefficient_sum <= square_sum,
        "integer anchor coefficients satisfy c <= c^2 termwise"
    );
    assert!(
        square_sum <= normalized_pair_sum,
        "because at least one anchor coefficient is zero, every c_i^2 occurs in the exact pair numerator"
    );
}

#[test]
fn pair_admitted_bound_survives_known_compact_and_boundary_geometries() {
    for sample_count in [4_usize, 16, 17, 32, 64, 128, 256] {
        assert_pair_admission_bounds_linear_accumulators(&deterministic_compact_fixture(
            sample_count,
        ));
    }

    assert_pair_admission_bounds_linear_accumulators(&boundary_fixture(65, 1_u128 << 58));
    assert_pair_admission_bounds_linear_accumulators(&boundary_fixture(
        65,
        (1_u128 << 58) + 1,
    ));
}

#[test]
fn pair_admitted_bound_holds_across_small_integer_composition_space() {
    for sample_count in 2_usize..=7 {
        let state_count = 4_usize.pow(u32::try_from(sample_count).expect("small exponent"));
        for mut state in 0..state_count {
            let mut values = Vec::with_capacity(sample_count);
            for _ in 0..sample_count {
                values.push(u128::try_from(state % 4).expect("base-four digit fits u128"));
                state /= 4;
            }
            assert_pair_admission_bounds_linear_accumulators(&values);
        }
    }
}
