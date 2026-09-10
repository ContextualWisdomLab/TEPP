//! Process-isolated resource characterization for the bias-SE exact-proof budget.
//!
//! This example measures one route per process on the same deterministic odd-dyadic
//! geometry. It exists to separate the candidate narrow→wide path from the current
//! narrow→pair fallback for allocator/RSS evidence. It does not widen production
//! admission and is not an independent scientific acceptance oracle.

use std::hint::black_box;
use std::mem::size_of;
use std::time::{Duration, Instant};

const SAMPLE_COUNT: usize = 2_047;
const DIAMETER: u128 = (1_u128 << 58) + 1;

#[derive(Clone, Copy)]
struct KernelObservation {
    aligned_pair_square_sum: u128,
    scratch_records: usize,
    scratch_payload_bytes: usize,
    used_wide_product: bool,
    used_pairwise_fallback: bool,
}

#[derive(Clone, Copy)]
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

type Kernel = fn(&[u128]) -> Option<KernelObservation>;

fn boundary_fixture() -> Vec<u128> {
    let mut values = Vec::with_capacity(SAMPLE_COUNT);
    values.push(0);
    values.extend((1..SAMPLE_COUNT).map(|_| DIAMETER));
    values
}

fn normalized_linear_terms(values: &[u128]) -> Option<(u128, u128)> {
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
    Some((coefficient_sum, square_sum))
}

fn pair_square_sum_linear(values: &[u128]) -> Option<KernelObservation> {
    let sample_count = u128::try_from(values.len()).ok()?;
    let (coefficient_sum, square_sum) = normalized_linear_terms(values)?;
    let pair_square_sum = sample_count
        .checked_mul(square_sum)?
        .checked_sub(coefficient_sum.checked_mul(coefficient_sum)?)?;
    Some(KernelObservation {
        aligned_pair_square_sum: pair_square_sum,
        scratch_records: 0,
        scratch_payload_bytes: 0,
        used_wide_product: false,
        used_pairwise_fallback: false,
    })
}

fn pair_square_sum_linear_wide_product(values: &[u128]) -> Option<KernelObservation> {
    let sample_count = u128::try_from(values.len()).ok()?;
    let (coefficient_sum, square_sum) = normalized_linear_terms(values)?;
    let pair_square_sum = Wide256::multiply_u128(sample_count, square_sum)
        .checked_sub(Wide256::multiply_u128(coefficient_sum, coefficient_sum))?
        .as_u128()?;
    Some(KernelObservation {
        aligned_pair_square_sum: pair_square_sum,
        scratch_records: 0,
        scratch_payload_bytes: 0,
        used_wide_product: true,
        used_pairwise_fallback: false,
    })
}

fn pair_square_sum_quadratic_buffered(values: &[u128]) -> Option<KernelObservation> {
    let pair_count = values
        .len()
        .checked_mul(values.len().checked_sub(1)?)?
        .checked_div(2)?;
    let mut records: Vec<Option<(u128, i32)>> = Vec::with_capacity(pair_count);
    for left in 0..values.len() {
        for right in left + 1..values.len() {
            let difference = values[left].abs_diff(values[right]);
            records.push((difference != 0).then_some((difference, 0)));
        }
    }
    let scratch_records = records.capacity();
    let scratch_payload_bytes =
        scratch_records.checked_mul(size_of::<Option<(u128, i32)>>())?;
    let mut pair_square_sum = 0_u128;
    for (difference, _) in records.into_iter().flatten() {
        pair_square_sum = pair_square_sum.checked_add(difference.checked_mul(difference)?)?;
    }
    Some(KernelObservation {
        aligned_pair_square_sum: pair_square_sum,
        scratch_records,
        scratch_payload_bytes,
        used_wide_product: false,
        used_pairwise_fallback: false,
    })
}

fn narrow_pair_fallback(values: &[u128]) -> Option<KernelObservation> {
    if let Some(observation) = pair_square_sum_linear(values) {
        return Some(observation);
    }
    let mut observation = pair_square_sum_quadratic_buffered(values)?;
    observation.used_pairwise_fallback = true;
    Some(observation)
}

fn narrow_wide_candidate(values: &[u128]) -> Option<KernelObservation> {
    if let Some(observation) = pair_square_sum_linear(values) {
        return Some(observation);
    }
    pair_square_sum_linear_wide_product(values)
}

fn percentile_95(mut durations: Vec<Duration>) -> Duration {
    durations.sort_unstable();
    let rank = durations.len().saturating_mul(95).div_ceil(100);
    durations[rank.saturating_sub(1)]
}

fn measure(values: &[u128], samples: usize, kernel: Kernel) -> (Duration, KernelObservation) {
    for _ in 0..3 {
        black_box(kernel(black_box(values)).expect("fixture must remain exactly representable"));
    }
    let mut durations = Vec::with_capacity(samples);
    for _ in 0..samples {
        let started = Instant::now();
        black_box(kernel(black_box(values)).expect("fixture must remain exactly representable"));
        durations.push(started.elapsed());
    }
    let observation = kernel(values).expect("fixture must remain exactly representable");
    (percentile_95(durations), observation)
}

fn expected_pair_square_sum() -> u128 {
    u128::try_from(SAMPLE_COUNT - 1)
        .expect("sample count fits u128")
        .checked_mul(
            DIAMETER
                .checked_mul(DIAMETER)
                .expect("diameter square fits u128"),
        )
        .expect("odd-boundary pair-square sum fits u128")
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let mode = arguments.next().unwrap_or_else(|| "candidate".to_owned());
    let samples = arguments
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(25)
        .max(1);
    assert!(
        arguments.next().is_none(),
        "expected mode and optional timing sample count only"
    );

    let values = boundary_fixture();
    let (kernel_name, kernel): (&str, Kernel) = match mode.as_str() {
        "candidate" => ("narrow_wide_candidate", narrow_wide_candidate),
        "fallback" => ("narrow_pair_fallback", narrow_pair_fallback),
        _ => panic!("mode must be 'candidate' or 'fallback'"),
    };
    let (p95, observation) = measure(&values, samples, kernel);
    assert_eq!(
        observation.aligned_pair_square_sum,
        expected_pair_square_sum()
    );
    match mode.as_str() {
        "candidate" => {
            assert!(observation.used_wide_product);
            assert!(!observation.used_pairwise_fallback);
            assert_eq!(observation.scratch_records, 0);
            assert_eq!(observation.scratch_payload_bytes, 0);
        }
        "fallback" => {
            assert!(!observation.used_wide_product);
            assert!(observation.used_pairwise_fallback);
            assert_eq!(observation.scratch_records, 2_094_081);
        }
        _ => unreachable!("mode validated above"),
    }

    println!(
        "mode,sample_count,kernel,p95_ns,timing_samples,scratch_records,scratch_payload_bytes,pair_record_size_bytes,used_wide_product,used_pairwise_fallback"
    );
    println!(
        "{mode},{SAMPLE_COUNT},{kernel_name},{},{samples},{},{},{},{},{}",
        p95.as_nanos(),
        observation.scratch_records,
        observation.scratch_payload_bytes,
        size_of::<Option<(u128, i32)>>(),
        observation.used_wide_product,
        observation.used_pairwise_fallback
    );
}
