//! Parameter-wise bias must not disappear through simplex-wide cancellation.

use validation_core::{mean_absolute_parameter_bias, parameter_mean_biases};

#[test]
fn simplex_wide_zero_sum_does_not_erase_parameter_bias() {
    let truth = [0.6, 0.4];
    let recovered = [vec![0.7, 0.3], vec![0.7, 0.3], vec![0.7, 0.3]];

    let biases = parameter_mean_biases(&truth, &recovered).expect("parameter-wise bias");
    assert!((biases[0] - 0.1).abs() < 1.0e-12);
    assert!((biases[1] + 0.1).abs() < 1.0e-12);
    assert!(biases.iter().sum::<f64>().abs() < 1.0e-12);

    let mean_absolute =
        mean_absolute_parameter_bias(&truth, &recovered).expect("mean absolute parameter bias");
    assert!((mean_absolute - 0.1).abs() < 1.0e-12);
}
