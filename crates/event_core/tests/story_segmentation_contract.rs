//! TDT story segments are not instances; `WindowDiff` and `Pk` come from truth.

use event_core::{
    EventConfidence, EventError, StoryBoundaryLabel, StorySegmentation, decide_story_boundary,
    refuse_story_segmentation_as_instance, refuse_story_segmentation_as_transition,
    story_boundary_precision, story_boundary_recall, story_pk, story_window_diff,
};

fn computed_rmse(truth: &[f64], recovered: &[f64]) -> f64 {
    assert_eq!(truth.len(), recovered.len());
    let n = f64::from(u32::try_from(truth.len()).expect("tiny fixture"));
    let sse: f64 = truth
        .iter()
        .zip(recovered)
        .map(|(truth_value, recovered_value)| {
            let residual = truth_value - recovered_value;
            residual * residual
        })
        .sum();
    (sse / n).sqrt()
}

fn segmentation(unit_count: u32, boundary_after: &[bool]) -> StorySegmentation {
    StorySegmentation::new(unit_count, boundary_after.to_vec()).expect("segmentation")
}

#[test]
fn story_segmentation_cannot_be_cast_to_an_instance_or_transition() {
    let story = segmentation(4, &[false, true, false]);
    assert_eq!(
        refuse_story_segmentation_as_instance(&story),
        Err(EventError::StorySegmentationIsNotEventInstance)
    );
    assert_eq!(
        refuse_story_segmentation_as_transition(&story),
        Err(EventError::StorySegmentationIsNotStateTransition)
    );
}

#[test]
fn window_diff_and_pk_are_computed_from_known_truth_boundaries() {
    let truth = segmentation(
        10,
        &[false, false, false, false, true, false, false, false, false],
    );
    let calibrated = segmentation(
        10,
        &[false, false, false, true, false, false, false, false, false],
    );
    let always_cut = segmentation(10, &[true, true, true, true, true, true, true, true, true]);

    let calibrated_wd = story_window_diff(&truth, &calibrated, 3).expect("window-diff");
    let naive_wd = story_window_diff(&truth, &always_cut, 3).expect("naive window-diff");
    let calibrated_pk = story_pk(&truth, &calibrated, 3).expect("pk");
    let naive_pk = story_pk(&truth, &always_cut, 3).expect("naive pk");

    assert!((calibrated_wd - (2.0 / 7.0)).abs() < 1.0e-12);
    assert!((naive_wd - 1.0).abs() < f64::EPSILON);
    assert!((calibrated_pk - (2.0 / 7.0)).abs() < 1.0e-12);
    assert!((naive_pk - (4.0 / 7.0)).abs() < 1.0e-12);
    assert!(
        story_window_diff(&truth, &truth, 3)
            .expect("identity")
            .abs()
            < f64::EPSILON
    );
    assert!(story_pk(&truth, &truth, 3).expect("identity pk").abs() < f64::EPSILON);
}

#[test]
fn boundary_precision_and_recall_are_computed_from_known_truth() {
    let truth = segmentation(8, &[false, false, true, false, false, true, false]);
    let calibrated = segmentation(8, &[false, false, true, false, false, false, false]);
    let always_cut = segmentation(8, &[true, true, true, true, true, true, true]);

    let calibrated_precision = story_boundary_precision(&truth, &calibrated).expect("precision");
    let naive_precision = story_boundary_precision(&truth, &always_cut).expect("naive precision");
    let calibrated_recall = story_boundary_recall(&truth, &calibrated).expect("recall");
    let naive_recall = story_boundary_recall(&truth, &always_cut).expect("naive recall");

    assert!((calibrated_precision - 1.0).abs() < f64::EPSILON);
    assert!((naive_precision - (2.0 / 7.0)).abs() < 1.0e-12);
    assert!((calibrated_recall - 0.5).abs() < f64::EPSILON);
    assert!((naive_recall - 1.0).abs() < f64::EPSILON);
}

#[test]
fn calibrated_boundary_scores_have_lower_rmse_than_always_cut() {
    let truth = [0.0_f64, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0];
    let calibrated = [0.10_f64, 0.15, 0.90, 0.20, 0.05, 0.85, 0.10];
    let always_cut = [1.0_f64, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    let calibrated_rmse = computed_rmse(&truth, &calibrated);
    let naive_rmse = computed_rmse(&truth, &always_cut);
    assert!(
        calibrated_rmse < naive_rmse,
        "computed calibrated RMSE {calibrated_rmse} must be below always-cut RMSE {naive_rmse}"
    );
}

#[test]
fn segmentation_helpers_fail_closed_on_empty_mismatched_and_oversize_windows() {
    assert_eq!(
        StorySegmentation::new(1, vec![]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        StorySegmentation::new(4, vec![true]),
        Err(EventError::InvalidWirePayload)
    );
    let truth = segmentation(4, &[false, true, false]);
    let recovered = segmentation(5, &[false, true, false, false]);
    assert_eq!(
        story_window_diff(&truth, &recovered, 2),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        story_window_diff(&truth, &truth, 0),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        story_window_diff(&truth, &truth, 4),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        story_boundary_precision(&truth, &segmentation(4, &[false, false, false])),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        story_boundary_recall(&segmentation(4, &[false, false, false]), &truth),
        Err(EventError::InvalidWirePayload)
    );
}

#[test]
fn labels_round_trip_and_threshold_is_inclusive() {
    assert_eq!(StoryBoundaryLabel::Boundary.wire_name(), "boundary");
    assert_eq!(StoryBoundaryLabel::Continuation.wire_name(), "continuation");
    assert_eq!(
        StoryBoundaryLabel::from_wire_name("boundary").expect("parse"),
        StoryBoundaryLabel::Boundary
    );
    assert_eq!(
        StoryBoundaryLabel::from_wire_name("continuation").expect("parse"),
        StoryBoundaryLabel::Continuation
    );
    assert_eq!(
        StoryBoundaryLabel::from_wire_name("cut"),
        Err(EventError::UnknownStoryBoundaryLabel)
    );
    assert!(StoryBoundaryLabel::Boundary.is_boundary());
    assert!(!StoryBoundaryLabel::Continuation.is_boundary());
    assert!((StoryBoundaryLabel::Boundary.as_probability_target() - 1.0).abs() < f64::EPSILON);
    assert!((StoryBoundaryLabel::Continuation.as_probability_target() - 0.0).abs() < f64::EPSILON);

    let half = EventConfidence::new(0.5).expect("half");
    assert_eq!(
        decide_story_boundary(half, half),
        StoryBoundaryLabel::Boundary
    );
    assert_eq!(
        decide_story_boundary(EventConfidence::new(0.49).expect("below"), half),
        StoryBoundaryLabel::Continuation
    );
}
