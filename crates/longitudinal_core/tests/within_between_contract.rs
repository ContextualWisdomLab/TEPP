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
fn decomposed_components_have_lower_computed_rmse_than_a_pooled_collapse() {
    let observations = [
        OccasionObservation::new(0, 0, 2.0),
        OccasionObservation::new(0, 1, 2.2),
        OccasionObservation::new(1, 0, 0.0),
        OccasionObservation::new(1, 1, 0.4),
    ];
    let recovered = decompose_within_between(&observations).expect("decompose");
    let truth = recovered.clone();
    let grand_mean = {
        let mut total = 0.0_f64;
        for observation in &observations {
            total += observation.score();
        }
        total / f64::from(u32::try_from(observations.len()).expect("len"))
    };
    let collapse: Vec<ComponentValue> = recovered
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
    let expected = {
        let mut sum_squares = 0.0_f64;
        for (truth_row, decided_row) in truth.iter().zip(recovered.iter()) {
            let residual = decided_row.value() - truth_row.value();
            sum_squares += residual * residual;
        }
        (sum_squares / f64::from(u32::try_from(truth.len()).expect("len"))).sqrt()
    };
    assert!((recovered_rmse - expected).abs() < f64::EPSILON);
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
