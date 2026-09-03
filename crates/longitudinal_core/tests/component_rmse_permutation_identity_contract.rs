use longitudinal_core::{
    ComponentLevel, ComponentValue, component_root_mean_square_error,
};

#[test]
fn component_rmse_aligns_by_scientific_identity_not_row_position() {
    let truth = [
        ComponentValue::new(7, 0, ComponentLevel::Between, 1.25),
        ComponentValue::new(7, 3, ComponentLevel::Within, -0.5),
        ComponentValue::new(11, 0, ComponentLevel::Between, 4.0),
    ];
    let decided = [
        ComponentValue::new(11, 0, ComponentLevel::Between, 4.0),
        ComponentValue::new(7, 0, ComponentLevel::Between, 1.25),
        ComponentValue::new(7, 3, ComponentLevel::Within, -0.5),
    ];

    assert_eq!(component_root_mean_square_error(&truth, &decided), Ok(0.0));
}
