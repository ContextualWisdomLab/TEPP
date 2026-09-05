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
    let mut coefficient_sum = 0_u128;
    let mut square_sum = 0_u128;
    for value in values {
        let coefficient = value.checked_sub(minimum)?;
        coefficient_sum = coefficient_sum.checked_add(coefficient)?;
        square_sum = square_sum.checked_add(coefficient.checked_mul(coefficient)?)?;
    }
    sample_count
        .checked_mul(square_sum)?
        .checked_sub(coefficient_sum.checked_mul(coefficient_sum)?)
}

fn greatest_common_divisor(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn worst_case_linear_intermediate(sample_count: u128, diameter: u128) -> Option<u128> {
    sample_count
        .checked_mul(sample_count)?
        .checked_mul(diameter.checked_mul(diameter)?)
}

#[test]
fn seventeen_observation_fixture_proves_linear_identity_matches_pair_reference() {
    const EXACT_PAIR_SQUARE_SUM: u128 = 92_549_865_125_191_410_206;
    const SCIENTIFIC_DENOMINATOR: u128 = 4_624;
    const REDUCED_NUMERATOR: u128 = 46_274_932_562_595_705_103;
    const REDUCED_DENOMINATOR: u128 = 2_312;

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

    let truth = [0.0; 17];
    let recovered = SEVENTEEN_OBSERVATION_FIXTURE.map(|value| {
        f64::from(u32::try_from(value).expect("fixture value fits u32 exactly"))
    });
    assert_eq!(
        bias_standard_error(&truth, &recovered)
            .expect("current bounded fallback remains representable")
            .to_bits(),
        0x41a0_dd77_9ac3_8e98
    );
}

#[test]
fn compact_dyadic_worst_case_u128_ceiling_is_2047_samples() {
    let exact_integer_diameter = 1_u128 << 53;
    assert!(worst_case_linear_intermediate(2_047, exact_integer_diameter).is_some());
    assert!(worst_case_linear_intermediate(2_048, exact_integer_diameter).is_none());

    let maximum_safe_denominator = 2_047_u128 * 2_047 * 2_046;
    assert!(maximum_safe_denominator < (1_u128 << 53));
}
