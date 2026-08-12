//! Temporal-order recovery accuracy.

use crate::ValidationError;

/// Accuracy of pairwise temporal order among recovered event times.
///
/// For every pair `i < j`, the recovered pair is correct when
/// `sign(recovered[j] − recovered[i]) == sign(truth[j] − truth[i])`, treating
/// exact ties as a distinct sign class.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty, single-element, unequal,
/// or non-finite vectors.
pub fn temporal_order_accuracy(
    truth_times: &[f64],
    recovered_times: &[f64],
) -> Result<f64, ValidationError> {
    if truth_times.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }
    if truth_times.len() != recovered_times.len() {
        return Err(ValidationError::InvalidInput);
    }
    if truth_times.iter().any(|value| !value.is_finite()) {
        return Err(ValidationError::InvalidInput);
    }
    if recovered_times.iter().any(|value| !value.is_finite()) {
        return Err(ValidationError::InvalidInput);
    }
    let mut correct = 0usize;
    let mut total = 0usize;
    for i in 0..truth_times.len() {
        for j in (i + 1)..truth_times.len() {
            total += 1;
            let truth_sign = (truth_times[j] - truth_times[i]).partial_cmp(&0.0);
            let recovered_sign = (recovered_times[j] - recovered_times[i]).partial_cmp(&0.0);
            if truth_sign == recovered_sign {
                correct += 1;
            }
        }
    }
    Ok(correct as f64 / total as f64)
}

#[cfg(test)]
mod tests {
    use super::temporal_order_accuracy;
    use crate::ValidationError;

    #[test]
    fn order_accuracy_oracle_and_degenerate_cases() {
        let truth = [1.0, 2.0, 3.0];
        let recovered = [0.5, 0.6, 0.4];
        // pairs: (0,1) truth < recovered < ok; (0,2) truth < recovered > fail; (1,2) truth < recovered > fail → 1/3
        assert!(
            (temporal_order_accuracy(&truth, &recovered).expect("acc") - (1.0 / 3.0)).abs() < 1e-12
        );
        assert_eq!(
            temporal_order_accuracy(&[1.0], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            temporal_order_accuracy(&[1.0, 2.0], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            temporal_order_accuracy(&[1.0, f64::NAN], &[1.0, 2.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            temporal_order_accuracy(&[1.0, 2.0], &[1.0, f64::NAN]),
            Err(ValidationError::InvalidInput)
        );
        let ties = [1.0, 1.0, 2.0];
        assert!((temporal_order_accuracy(&ties, &ties).expect("ties") - 1.0).abs() < 1e-12);
    }
}
