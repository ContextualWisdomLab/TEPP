//! Deterministic binary64 accumulation shared by validation metrics.

/// Sum finite values in a canonical order with Neumaier compensation.
///
/// Callers own domain validation and any scale normalization needed to keep the
/// final sum representable. Canonical ordering keeps equivalent metric inputs
/// from changing only because transport order changed.
pub(crate) fn deterministic_compensated_sum(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let mut sum = 0.0_f64;
    let mut correction = 0.0_f64;
    for value in values {
        let next = sum + value;
        if sum.abs() >= value.abs() {
            correction += (sum - next) + value;
        } else {
            correction += (value - next) + sum;
        }
        sum = next;
    }
    sum + correction
}

#[cfg(test)]
mod tests {
    use super::deterministic_compensated_sum;

    #[test]
    fn canonical_compensated_sum_is_order_stable() {
        let left = deterministic_compensated_sum(vec![1.0, 1e-100, -1.0]);
        let right = deterministic_compensated_sum(vec![-1.0, 1.0, 1e-100]);
        assert_eq!(left.to_bits(), right.to_bits());
    }
}
