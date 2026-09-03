use longitudinal_core::{
    ComponentLevel, ComponentValue, LongitudinalError, component_root_mean_square_error,
};

#[test]
fn stable_between_component_cannot_gain_weight_from_occasion_aliases() {
    let truth = [
        ComponentValue::new(7, 0, ComponentLevel::Between, 1.25),
        ComponentValue::new(7, 2, ComponentLevel::Between, 1.25),
        ComponentValue::new(11, 1, ComponentLevel::Within, 0.0),
    ];
    let decided = [
        ComponentValue::new(7, 0, ComponentLevel::Between, 1.25),
        ComponentValue::new(7, 2, ComponentLevel::Between, 1.25),
        ComponentValue::new(11, 1, ComponentLevel::Within, 3.0),
    ];

    assert_eq!(
        component_root_mean_square_error(&truth, &decided),
        Err(LongitudinalError::InvalidComponentPayload)
    );
}
