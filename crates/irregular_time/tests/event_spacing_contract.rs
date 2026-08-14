//! Event-time lags cannot be replaced by equal system-time spacing.

use irregular_time::{
    ClockedObservation, IrregularTimeError, event_lag_seconds, lag_root_mean_square_error,
    refuse_equal_system_spacing_as_event_spacing,
};

fn observation(event: i64, system: i64) -> ClockedObservation {
    ClockedObservation::new(event, system)
}

#[test]
fn equal_system_spacing_cannot_replace_irregular_event_lags() {
    let observations = [observation(0, 0), observation(10, 1), observation(13, 2)];
    assert_eq!(
        refuse_equal_system_spacing_as_event_spacing(&observations),
        Err(IrregularTimeError::SystemSpacingIsNotEventSpacing)
    );
    let lags = event_lag_seconds(&observations).expect("event lags");
    assert_eq!(lags, vec![10, 3]);
}

#[test]
fn event_lags_recover_known_truth_better_than_equal_system_spacing() {
    let truth_lags = [10_i64, 3];
    let observations = [observation(0, 0), observation(10, 1), observation(13, 2)];
    let event_lags = event_lag_seconds(&observations).expect("event");
    let assumed_equal = [1_i64, 1];
    let event_rmse = lag_root_mean_square_error(&truth_lags, &event_lags).expect("event rmse");
    let assumed_rmse = lag_root_mean_square_error(&truth_lags, &assumed_equal).expect("assumed");
    let expected = {
        let mut sum_squares = 0.0_f64;
        for (truth, decided) in truth_lags.iter().zip(event_lags.iter()) {
            let residual = f64::from(i32::try_from(*decided).expect("decided"))
                - f64::from(i32::try_from(*truth).expect("truth"));
            sum_squares += residual * residual;
        }
        (sum_squares / f64::from(u32::try_from(truth_lags.len()).expect("len"))).sqrt()
    };
    assert!((event_rmse - expected).abs() < f64::EPSILON);
    assert!(event_rmse < assumed_rmse);
}

#[test]
fn empty_or_non_increasing_event_clocks_fail_closed() {
    assert_eq!(
        event_lag_seconds(&[]),
        Err(IrregularTimeError::InvalidObservationPayload)
    );
    assert_eq!(
        event_lag_seconds(&[observation(5, 0), observation(4, 1)]),
        Err(IrregularTimeError::NonIncreasingEventTime)
    );
}
