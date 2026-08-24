//! Between-unit differences cannot be scored as within-unit change.

use longitudinal_core::{
    ComponentLevel, ComponentValue, LongitudinalError, OccasionObservation,
    component_root_mean_square_error, decompose_within_between, refuse_between_as_within_change,
};

#[test]
fn between_component_cannot_claim_within_unit_change() {
    assert_eq!(
        refuse_between_as_within_change(ComponentLevel::Between),
        Err(LongitudinalError::BetweenIsNotWithinChange)
    );
    assert_eq!(
        refuse_between_as_within_change(ComponentLevel::Within),
        Ok(())
    );
}

#[test]
fn decomposed_components_recover_known_truth_below_pooled_collapse() {
    let observations = [
        OccasionObservation::new(0, 0, 2.0),
        OccasionObservation::new(0, 1, 2.2),
        OccasionObservation::new(1, 0, 0.0),
        OccasionObservation::new(1, 1, 0.4),
    ];
    let recovered = decompose_within_between(&observations).expect("decompose");

    // Independent known truth: unit 0 has mean 2.1 with within deviations
    // -0.1/+0.1; unit 1 has mean 0.2 with within deviations -0.2/+0.2. The
    // truth vector is written from the statistical model, not from the
    // decomposition output.
    let truth = vec![
        ComponentValue::new(0, 0, ComponentLevel::Between, 2.1),
        ComponentValue::new(0, 0, ComponentLevel::Within, -0.1),
        ComponentValue::new(0, 1, ComponentLevel::Within, 0.1),
        ComponentValue::new(1, 0, ComponentLevel::Between, 0.2),
        ComponentValue::new(1, 0, ComponentLevel::Within, -0.2),
        ComponentValue::new(1, 1, ComponentLevel::Within, 0.2),
    ];
    assert_eq!(recovered.len(), truth.len());
    for (truth_row, recovered_row) in truth.iter().zip(recovered.iter()) {
        assert_eq!(truth_row.unit_index(), recovered_row.unit_index());
        assert_eq!(truth_row.occasion_index(), recovered_row.occasion_index());
        assert_eq!(truth_row.level(), recovered_row.level());
        assert!((truth_row.value() - recovered_row.value()).abs() < 1e-12);
    }

    let grand_mean = {
        let mut total = 0.0_f64;
        for observation in &observations {
            total += observation.score();
        }
        total / f64::from(u32::try_from(observations.len()).expect("len"))
    };
    let collapse: Vec<ComponentValue> = truth
        .iter()
        .map(|row| match row.level() {
            ComponentLevel::Between => ComponentValue::new(
                row.unit_index(),
                row.occasion_index(),
                row.level(),
                grand_mean,
            ),
            ComponentLevel::Within => {
                let score = observations
                    .iter()
                    .find(|observation| {
                        observation.unit_index() == row.unit_index()
                            && observation.occasion_index() == row.occasion_index()
                    })
                    .expect("score")
                    .score();
                ComponentValue::new(
                    row.unit_index(),
                    row.occasion_index(),
                    row.level(),
                    score - grand_mean,
                )
            }
        })
        .collect();

    let recovered_rmse = component_root_mean_square_error(&truth, &recovered).expect("recovered");
    let collapse_rmse = component_root_mean_square_error(&truth, &collapse).expect("collapse");
    assert!(recovered_rmse < 1e-12);
    // Every collapsed component misses its known-truth value by exactly 0.95,
    // so the pooled collapse RMSE is 0.95.
    assert!((collapse_rmse - 0.95).abs() < 1e-12);
    assert!(recovered_rmse < collapse_rmse);
}

#[test]
fn empty_or_non_finite_components_fail_closed() {
    assert_eq!(
        component_root_mean_square_error(&[], &[]),
        Err(LongitudinalError::InvalidComponentPayload)
    );
    assert_eq!(
        decompose_within_between(&[]),
        Err(LongitudinalError::InvalidObservationPayload)
    );
}
