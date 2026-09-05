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
    let split_product = (sample_count / 2)
        .checked_mul(sample_count.checked_sub(sample_count / 2)?)?;
    split_product.checked_mul(diameter.checked_mul(diameter)?)
}

fn scientific_denominator(sample_count: u128) -> Option<u128> {
    sample_count
        .checked_mul(sample_count)?
        .checked_mul(sample_count.checked_sub(1)?)
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
fn linear_checked_integer_kernel_matches_pair_reference_when_it_admits() {
    for sample_count in [4_usize, 16, 17, 32, 64, 128, 256] {
        let values = deterministic_compact_fixture(sample_count);
        let pairwise = pair_square_sum_quadratic(&values)
            .expect("compact-grid pair reference stays within u128");
        let linear = pair_square_sum_linear(&values)
            .expect("compact-grid linear kernel stays within u128");
        assert_eq!(
            linear, pairwise,
            "linear sufficient proof must preserve the exact pair numerator at n={sample_count}"
        );
    }
}

#[test]
fn linear_checked_integer_kernel_is_not_admission_equivalent_to_pair_reference() {
    let diameter = 1_u128 << 58;

    let mut fits_both = Vec::with_capacity(64);
    fits_both.push(0);
    fits_both.extend((0..63).map(|_| diameter));
    let pairwise_64 = pair_square_sum_quadratic(&fits_both)
        .expect("64-sample pair numerator stays within u128");
    assert_eq!(
        pair_square_sum_linear(&fits_both),
        Some(pairwise_64),
        "n=64 remains inside the minimum-shifted linear intermediate budget"
    );

    let mut pair_only = Vec::with_capacity(65);
    pair_only.push(0);
    pair_only.extend((0..64).map(|_| diameter));
    let pairwise_65 = pair_square_sum_quadratic(&pair_only)
        .expect("65-sample pair numerator still stays within u128");
    let expected_pairwise_65 = 64_u128
        .checked_mul(diameter.checked_mul(diameter).expect("diameter square fits"))
        .expect("pair numerator fits");
    assert_eq!(pairwise_65, expected_pairwise_65);
    assert_eq!(
        pair_square_sum_linear(&pair_only),
        None,
        "n*sum(c_i^2) overflows before cancellation even though the exact pair numerator fits"
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
