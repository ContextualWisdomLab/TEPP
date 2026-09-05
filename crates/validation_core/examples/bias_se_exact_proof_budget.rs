//! Reproducible integer-kernel timing and layout harness for bias-SE exact-proof budgeting.
//!
//! This example compares three checked-integer kernels on deterministic compact
//! dyadic coefficients: a production-layout-shaped buffered O(n²) pair proof, an
//! allocation-free two-pass O(n²) variant, and an algebraically equivalent O(n)
//! accumulator. It is characterization tooling, not production admission and not
//! buyer-path latency evidence by itself.

use std::hint::black_box;
use std::mem::size_of;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct KernelObservation {
    aligned_pair_square_sum: u128,
    unit_exponent: i32,
    scratch_records: usize,
    scratch_payload_bytes: usize,
}

fn fixture(sample_count: usize) -> Vec<u128> {
    (0..sample_count)
        .map(|index| {
            let value = u128::try_from(index).expect("fixture index fits u128");
            (value * 1_000_003 + value * value * 97 + 17) % 4_000_000_001
        })
        .collect()
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
        });
    }
    let aligned_pair_square_sum = accumulate_aligned_pair_square_sum(records, unit_exponent)?;
    Some(KernelObservation {
        aligned_pair_square_sum,
        unit_exponent,
        scratch_records,
        scratch_payload_bytes,
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
    })
}

fn pair_square_sum_linear(values: &[u128]) -> Option<KernelObservation> {
    let minimum = *values.iter().min()?;
    let sample_count = u128::try_from(values.len()).ok()?;
    let mut coefficient_sum = 0_u128;
    let mut square_sum = 0_u128;
    for value in values {
        let coefficient = value.checked_sub(minimum)?;
        coefficient_sum = coefficient_sum.checked_add(coefficient)?;
        square_sum = square_sum.checked_add(coefficient.checked_mul(coefficient)?)?;
    }
    let pair_square_sum = sample_count
        .checked_mul(square_sum)?
        .checked_sub(coefficient_sum.checked_mul(coefficient_sum)?)?;
    Some(KernelObservation {
        aligned_pair_square_sum: pair_square_sum,
        unit_exponent: 0,
        scratch_records: 0,
        scratch_payload_bytes: 0,
    })
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
    sample_count: usize,
    kernel_name: &str,
    p95: Duration,
    samples: usize,
    observation: KernelObservation,
) {
    println!(
        "{sample_count},{kernel_name},{},{samples},{},{},{},{}",
        p95.as_nanos(),
        observation.unit_exponent,
        observation.scratch_records,
        observation.scratch_payload_bytes,
        size_of::<Option<(u128, i32)>>()
    );
}

fn main() {
    let samples = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(25)
        .max(1);

    println!(
        "sample_count,kernel,p95_ns,timing_samples,unit_exponent,scratch_records,scratch_payload_bytes,pair_record_size_bytes"
    );
    for sample_count in [16_usize, 64, 256, 1_024, 2_047] {
        let values = fixture(sample_count);
        let buffered = pair_square_sum_quadratic_buffered(&values)
            .expect("buffered quadratic result stays within u128");
        let two_pass = pair_square_sum_quadratic_two_pass(&values)
            .expect("two-pass quadratic result stays within u128");
        let linear = pair_square_sum_linear(&values).expect("linear result stays within u128");
        let exact_pair_square_sum = restored_pair_square_sum(buffered)
            .expect("buffered result restores to exact pair-square sum");
        assert_eq!(
            restored_pair_square_sum(two_pass),
            Some(exact_pair_square_sum),
            "quadratic kernels must agree"
        );
        assert_eq!(
            restored_pair_square_sum(linear),
            Some(exact_pair_square_sum),
            "linear identity must agree with pair reference"
        );

        let (buffered_p95, buffered_observation) =
            measure(&values, samples, pair_square_sum_quadratic_buffered);
        let (two_pass_p95, two_pass_observation) =
            measure(&values, samples, pair_square_sum_quadratic_two_pass);
        let (linear_p95, linear_observation) = measure(&values, samples, pair_square_sum_linear);
        emit(
            sample_count,
            "quadratic_buffered",
            buffered_p95,
            samples,
            buffered_observation,
        );
        emit(
            sample_count,
            "quadratic_two_pass",
            two_pass_p95,
            samples,
            two_pass_observation,
        );
        emit(
            sample_count,
            "linear",
            linear_p95,
            samples,
            linear_observation,
        );
    }
}
