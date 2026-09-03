use longitudinal_core::{
    ComponentLevel, ComponentValue, LongitudinalError, component_root_mean_square_error,
};

#[test]
fn nonzero_recovery_error_cannot_collapse_to_perfect_rmse() {
    let truth = [
        ComponentValue::new(0, 0, ComponentLevel::Within, 0.0),
        ComponentValue::new(1, 0, ComponentLevel::Within, 0.0),
        ComponentValue::new(2, 0, ComponentLevel::Within, 0.0),
        ComponentValue::new(3, 0, ComponentLevel::Within, 0.0),
        ComponentValue::new(4, 0, ComponentLevel::Within, 0.0),
    ];
    let recovered = [
        ComponentValue::new(0, 0, ComponentLevel::Within, f64::from_bits(1)),
        ComponentValue::new(1, 0, ComponentLevel::Within, 0.0),
        ComponentValue::new(2, 0, ComponentLevel::Within, 0.0),
        ComponentValue::new(3, 0, ComponentLevel::Within, 0.0),
        ComponentValue::new(4, 0, ComponentLevel::Within, 0.0),
    ];

    assert_eq!(
        component_root_mean_square_error(&truth, &recovered),
        Err(LongitudinalError::InvalidComponentPayload),
        "a nonzero exact recovery error whose RMSE is below binary64 range must fail closed rather than report perfect recovery",
    );
}
