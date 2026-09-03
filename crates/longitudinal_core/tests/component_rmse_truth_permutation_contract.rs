use longitudinal_core::{
    ComponentLevel, ComponentValue, component_root_mean_square_error,
};

#[test]
fn component_rmse_is_bit_identical_under_truth_row_permutation() {
    let just_below_one = f64::from_bits(1.0_f64.to_bits() - 1);
    let truth_a = [
        ComponentValue::new(0, 0, ComponentLevel::Between, 0.0),
        ComponentValue::new(1, 0, ComponentLevel::Between, 0.0),
        ComponentValue::new(2, 0, ComponentLevel::Between, 0.0),
        ComponentValue::new(3, 0, ComponentLevel::Between, 0.0),
    ];
    let truth_b = [
        ComponentValue::new(0, 0, ComponentLevel::Between, 0.0),
        ComponentValue::new(1, 0, ComponentLevel::Between, 0.0),
        ComponentValue::new(3, 0, ComponentLevel::Between, 0.0),
        ComponentValue::new(2, 0, ComponentLevel::Between, 0.0),
    ];
    let decided = [
        ComponentValue::new(0, 0, ComponentLevel::Between, 1.0),
        ComponentValue::new(1, 0, ComponentLevel::Between, 1e-100),
        ComponentValue::new(2, 0, ComponentLevel::Between, 3.0),
        ComponentValue::new(3, 0, ComponentLevel::Between, just_below_one),
    ];

    let first = component_root_mean_square_error(&truth_a, &decided).expect("first RMSE");
    let permuted = component_root_mean_square_error(&truth_b, &decided).expect("permuted RMSE");

    assert_eq!(first.to_bits(), permuted.to_bits());
}
