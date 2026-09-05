//! Reproducible integer-kernel timing harness for bias-SE exact-proof budgeting.
//!
//! This example compares the current pair-distance O(n²) integer identity with
//! an algebraically equivalent O(n) accumulator on deterministic compact dyadic
//! coefficients. It is characterization tooling, not production admission and
//! not buyer-path latency evidence by itself.

use std::hint::black_box;
use std::time::{Duration, Instant};

fn fixture(sample_count: usize) -> Vec<u128> {
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

fn percentile_95(mut durations: Vec<Duration>) -> Duration {
    durations.sort_unstable();
    let rank = durations.len().saturating_mul(95).div_ceil(100);
    durations[rank.saturating_sub(1)]
}

fn measure(
    values: &[u128],
    samples: usize,
    kernel: fn(&[u128]) -> Option<u128>,
) -> Duration {
    for _ in 0..3 {
        black_box(kernel(black_box(values)).expect("fixture must remain within u128"));
    }
    let mut durations = Vec::with_capacity(samples);
    for _ in 0..samples {
        let started = Instant::now();
        black_box(kernel(black_box(values)).expect("fixture must remain within u128"));
        durations.push(started.elapsed());
    }
    percentile_95(durations)
}

fn main() {
    let samples = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(25)
        .max(1);

    println!("sample_count,kernel,p95_ns,timing_samples");
    for sample_count in [16_usize, 64, 256, 1_024, 2_047] {
        let values = fixture(sample_count);
        let quadratic = pair_square_sum_quadratic(&values).expect("quadratic result");
        let linear = pair_square_sum_linear(&values).expect("linear result");
        assert_eq!(quadratic, linear, "algebraic kernels must agree");

        let quadratic_p95 = measure(&values, samples, pair_square_sum_quadratic);
        let linear_p95 = measure(&values, samples, pair_square_sum_linear);
        println!(
            "{sample_count},quadratic,{},{}",
            quadratic_p95.as_nanos(),
            samples
        );
        println!(
            "{sample_count},linear,{},{}",
            linear_p95.as_nanos(),
            samples
        );
    }
}
