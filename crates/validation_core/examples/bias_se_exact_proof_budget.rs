//! Reproducible integer-kernel timing and layout harness for bias-SE exact-proof budgeting.
//!
//! This example compares checked-integer proof kernels on deterministic dyadic
//! coefficients: a production-layout-shaped buffered O(n²) pair proof, an
//! allocation-free two-pass O(n²) variant, an algebraically equivalent O(n)
//! sufficient accumulator, and the viable hybrid shape that uses the O(n) path
//! only when it admits and otherwise falls back to the buffered pair proof.
//! The O(n) path first removes the shared power-of-two unit from anchor-relative
//! coefficients so checked-intermediate refusal is evaluated on the canonical
//! dyadic grid rather than on an arbitrary raw integer scale. It is
//! characterization tooling, not production admission and not buyer-path latency
//! evidence by itself.

use std::hint::black_box;
use std::mem::size_of;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct KernelObservation {
    aligned_pair_square_sum: u128,
    unit_exponent: i32,
    scratch_records: usize,
    scratch_payload_bytes: usize,
    used_pairwise_fallback: bool,
}

fn fixture(sample_count: usize) -> Vec<u128> {
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

fn multiply_by_power_of_two(value: u128, shift: u32) -> Option<u128> {
    let factor = 1_u128.checked_shl(shift)?;
    value.checked_mul(factor)
}

fn compact_dyadic(value: u128) -> Option<(u128, i32)> {
    if value == 0 {
        return None;
    }
    let trailing = value.trailing_zeros();
    Some((value >> trailing, i32::try_from(trailing).ok()?))
}

fn accumulate_aligned_pair_square_sum(
    records: impl IntoIterator<Item = Option<(u128, i32)>>,
    unit_exponent: i32,
) -> Option<u128> {
    let mut sum = 0_u128;
    for (significand, exponent) in records.into_iter().flatten() {
        let shift = exponent.checked_sub(unit_exponent)?.unsigned_abs();
        let coefficient = multiply_by_power_of_two(significand, shift)?;
        sum = sum.checked_add(coefficient.checked_mul(coefficient)?)?;
    }
    Some(sum)
}

fn pair_square_sum_quadratic_buffered(values: &[u128]) -> Option<KernelObservation> {
    let pair_count = values
        .len()
        .checked_mul(values.len().checked_sub(1)?)?
        .checked_div(2)?;
    let mut records: Vec<Option<(u128, i32)>> = Vec::with_capacity(pair_count);
    let mut unit_exponent = i32::MAX;
    for left in 0..values.len() {
        for right in left + 1..values.len() {
            let record = compact_dyadic(values[left].abs_diff(values[right]));
            if let Some((_, exponent)) = record {
                unit_exponent = unit_exponent.min(exponent);
            }
            records.push(record);
        }
    }
    let scratch_records = records.capacity();
    let scratch_payload_bytes = scratch_records.checked_mul(size_of::<Option<(u128, i32)>>())?;
    if unit_exponent == i32::MAX {
        return Some(KernelObservation {
            aligned_pair_square_sum: 0,
            unit_exponent: 0,
            scratch_records,
            scratch_payload_bytes,
            used_pairwise_fallback: false,
        });
    }
    let aligned_pair_square_sum = accumulate_aligned_pair_square_sum(records, unit_exponent)?;
    Some(KernelObservation {
        aligned_pair_square_sum,
        unit_exponent,
        scratch_records,
        scratch_payload_bytes,
        used_pairwise_fallback: false,
    })
}

fn pair_square_sum_quadratic_two_pass(values: &[u128]) -> Option<KernelObservation> {
    let mut unit_exponent = i32::MAX;
    for left in 0..values.len() {
        for right in left + 1..values.len() {
            if let Some((_, exponent)) = compact_dyadic(values[left].abs_diff(values[right])) {
                unit_exponent = unit_exponent.min(exponent);
            }
        }
    }
    if unit_exponent == i32::MAX {
        return Some(KernelObservation {
            aligned_pair_square_sum: 0,
            unit_exponent: 0,
            scratch_records: 0,
            scratch_payload_bytes: 0,
            used_pairwise_fallback: false,
        });
    }

    let records = (0..values.len()).flat_map(|left| {
        (left + 1..values.len())
            .map(move |right| compact_dyadic(values[left].abs_diff(values[right])))
    });
    let aligned_pair_square_sum = accumulate_aligned_pair_square_sum(records, unit_exponent)?;
    Some(KernelObservation {
        aligned_pair_square_sum,
        unit_exponent,
        scratch_records: 0,
        scratch_payload_bytes: 0,
        used_pairwise_fallback: false,
    })
}

fn pair_square_sum_linear(values: &[u128]) -> Option<KernelObservation> {
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
    let pair_square_sum = sample_count
        .checked_mul(square_sum)?
        .checked_sub(coefficient_sum.checked_mul(coefficient_sum)?)?;
    Some(KernelObservation {
        aligned_pair_square_sum: pair_square_sum,
        unit_exponent: i32::try_from(common_shift).ok()?,
        scratch_records: 0,
        scratch_payload_bytes: 0,
        used_pairwise_fallback: false,
    })
}

fn pair_square_sum_hybrid(values: &[u128]) -> Option<KernelObservation> {
    if let Some(observation) = pair_square_sum_linear(values) {
        return Some(observation);
    }
    let mut observation = pair_square_sum_quadratic_buffered(values)?;
    observation.used_pairwise_fallback = true;
    Some(observation)
}

fn restored_pair_square_sum(observation: KernelObservation) -> Option<u128> {
    let shift = observation.unit_exponent.checked_mul(2)?.unsigned_abs();
    multiply_by_power_of_two(observation.aligned_pair_square_sum, shift)
}

fn percentile_95(mut durations: Vec<Duration>) -> Duration {
    durations.sort_unstable();
    let rank = durations.len().saturating_mul(95).div_ceil(100);
    durations[rank.saturating_sub(1)]
}

fn measure(
    values: &[u128],
    samples: usize,
    kernel: fn(&[u128]) -> Option<KernelObservation>,
) -> (Duration, KernelObservation) {
    for _ in 0..3 {
        black_box(kernel(black_box(values)).expect("fixture must remain within u128"));
    }
    let mut durations = Vec::with_capacity(samples);
    for _ in 0..samples {
        let started = Instant::now();
        black_box(kernel(black_box(values)).expect("fixture must remain within u128"));
        durations.push(started.elapsed());
    }
    let observation = kernel(values).expect("fixture must remain within u128");
    (percentile_95(durations), observation)
}

fn emit(
    geometry: &str,
    sample_count: usize,
    kernel_name: &str,
    p95: Duration,
    samples: usize,
    observation: KernelObservation,
) {
    println!(
        "{geometry},{sample_count},{kernel_name},{},{samples},{},{},{},{},{}",
        p95.as_nanos(),
        observation.unit_exponent,
        observation.scratch_records,
        observation.scratch_payload_bytes,
        size_of::<Option<(u128, i32)>>(),
        observation.used_pairwise_fallback
    );
}

fn assert_and_measure_geometry(
    geometry: &str,
    values: &[u128],
    samples: usize,
    expect_linear_admission: bool,
    expect_hybrid_fallback: bool,
) {
    let buffered = pair_square_sum_quadratic_buffered(values)
        .expect("buffered quadratic result stays within u128");
    let two_pass = pair_square_sum_quadratic_two_pass(values)
        .expect("two-pass quadratic result stays within u128");
    let hybrid = pair_square_sum_hybrid(values).expect("hybrid result stays within u128");
    let exact_pair_square_sum = restored_pair_square_sum(buffered)
        .expect("buffered result restores to exact pair-square sum");
    assert_eq!(
        restored_pair_square_sum(two_pass),
        Some(exact_pair_square_sum),
        "quadratic kernels must agree"
    );
    assert_eq!(
        restored_pair_square_sum(hybrid),
        Some(exact_pair_square_sum),
        "hybrid must preserve the exact pair numerator"
    );
    assert_eq!(hybrid.used_pairwise_fallback, expect_hybrid_fallback);

    match pair_square_sum_linear(values) {
        Some(linear) => {
            assert!(expect_linear_admission, "linear admission was not expected");
            assert_eq!(
                restored_pair_square_sum(linear),
                Some(exact_pair_square_sum),
                "linear identity must agree with pair reference"
            );
        }
        None => assert!(!expect_linear_admission, "linear refusal was not expected"),
    }

    let mut kernels: Vec<(
        &str,
        fn(&[u128]) -> Option<KernelObservation>,
    )> = vec![
        ("quadratic_buffered", pair_square_sum_quadratic_buffered),
        ("quadratic_two_pass", pair_square_sum_quadratic_two_pass),
        ("hybrid", pair_square_sum_hybrid),
    ];
    if expect_linear_admission {
        kernels.push(("linear", pair_square_sum_linear));
    }
    for (kernel_name, kernel) in kernels {
        let (p95, observation) = measure(values, samples, kernel);
        emit(
            geometry,
            values.len(),
            kernel_name,
            p95,
            samples,
            observation,
        );
    }
}

fn main() {
    let samples = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(25)
        .max(1);

    println!(
        "geometry,sample_count,kernel,p95_ns,timing_samples,unit_exponent,scratch_records,scratch_payload_bytes,pair_record_size_bytes,used_pairwise_fallback"
    );
    for sample_count in [16_usize, 64, 256, 1_024, 2_047] {
        let values = fixture(sample_count);
        assert_and_measure_geometry("compact_admit", &values, samples, true, false);
    }

    let power_of_two_values = boundary_fixture(65, 1_u128 << 58);
    assert_and_measure_geometry(
        "power_of_two_normalized_admit",
        &power_of_two_values,
        samples,
        true,
        false,
    );

    let odd_diameter = (1_u128 << 58) + 1;
    let odd_64 = boundary_fixture(64, odd_diameter);
    assert_and_measure_geometry("odd_boundary_admit", &odd_64, samples, true, false);

    let odd_65 = boundary_fixture(65, odd_diameter);
    assert_and_measure_geometry(
        "odd_boundary_pair_fallback",
        &odd_65,
        samples,
        false,
        true,
    );
}
