//! Known-truth recovery denominators require unique component identities.

use longitudinal_core::{
    ComponentLevel, ComponentValue, LongitudinalError, component_root_mean_square_error,
};

#[test]
fn duplicate_component_identity_cannot_reweight_known_truth_rmse() {
    let truth = [
        ComponentValue::new(0, 0, ComponentLevel::Between, 1.0),
        ComponentValue::new(0, 0, ComponentLevel::Between, 1.0),
        ComponentValue::new(1, 0, ComponentLevel::Between, 3.0),
    ];
    let decided = [
        ComponentValue::new(0, 0, ComponentLevel::Between, 1.0),
        ComponentValue::new(0, 0, ComponentLevel::Between, 1.0),
        ComponentValue::new(1, 0, ComponentLevel::Between, 5.0),
    ];

    // The duplicate (unit, occasion, level) would count the zero-error unit
    // twice and silently lower the RMSE denominator from the unique-component
    // target. Recovery evidence must fail closed instead of changing weight by
    // duplicate identity multiplicity.
    assert_eq!(
        component_root_mean_square_error(&truth, &decided),
        Err(LongitudinalError::InvalidComponentPayload)
    );
}
