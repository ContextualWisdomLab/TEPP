//! CHRONOS schema-slot predictions are not instances; accuracy is computed from truth.

use event_core::{
    EventConfidence, EventError, EventRoleKind, SchemaPredictionId, SchemaSlotAssignment,
    SchemaSlotLabel, decide_schema_slot, refuse_schema_prediction_as_instance,
    refuse_schema_prediction_as_transition, schema_slot_precision, schema_slot_recall,
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

fn slot(role: EventRoleKind, argument: &str) -> SchemaSlotAssignment {
    SchemaSlotAssignment::new(role, argument).expect("slot")
}

#[test]
fn schema_prediction_cannot_be_cast_to_an_instance_or_transition() {
    let prediction = SchemaPredictionId::from_raw(1);
    assert_eq!(
        refuse_schema_prediction_as_instance(prediction),
        Err(EventError::SchemaPredictionIsNotEventInstance)
    );
    assert_eq!(
        refuse_schema_prediction_as_transition(prediction),
        Err(EventError::SchemaPredictionIsNotStateTransition)
    );
}

#[test]
fn slot_precision_and_recall_are_computed_from_known_truth_fills() {
    let truth = [
        slot(EventRoleKind::Agent, "procurement office"),
        slot(EventRoleKind::Product, "contract award"),
    ];
    let calibrated = [
        slot(EventRoleKind::Agent, "procurement office"),
        slot(EventRoleKind::Product, "contract award"),
        slot(EventRoleKind::Place, "seoul"),
    ];
    let always_fill = [
        slot(EventRoleKind::Agent, "procurement office"),
        slot(EventRoleKind::Product, "contract award"),
        slot(EventRoleKind::Place, "seoul"),
        slot(EventRoleKind::Patient, "vendor"),
        slot(EventRoleKind::Factor, "budget"),
        slot(EventRoleKind::Instrument, "tender"),
    ];

    let calibrated_precision = schema_slot_precision(&truth, &calibrated).expect("precision");
    let naive_precision = schema_slot_precision(&truth, &always_fill).expect("naive p");
    let calibrated_recall = schema_slot_recall(&truth, &calibrated).expect("recall");
    let naive_recall = schema_slot_recall(&truth, &always_fill).expect("naive r");

    assert!(
        calibrated_precision > naive_precision,
        "computed precision {calibrated_precision} must exceed always-fill precision {naive_precision}"
    );
    assert!((calibrated_recall - naive_recall).abs() < f64::EPSILON);
}

#[test]
fn calibrated_slot_occupancy_scores_have_lower_rmse_than_always_fill() {
    let truth = [1.0_f64, 1.0, 0.0, 0.0, 0.0, 1.0];
    let calibrated = [0.90_f64, 0.85, 0.15, 0.10, 0.20, 0.88];
    let always_fill = [1.0_f64, 1.0, 1.0, 1.0, 1.0, 1.0];
    let calibrated_rmse = computed_rmse(&truth, &calibrated);
    let naive_rmse = computed_rmse(&truth, &always_fill);
    assert!(
        calibrated_rmse < naive_rmse,
        "computed calibrated RMSE {calibrated_rmse} must be below always-fill RMSE {naive_rmse}"
    );
}

#[test]
fn assignment_helpers_fail_closed_on_empty_duplicate_and_blank_arguments() {
    let one = [slot(EventRoleKind::Agent, "procurement office")];
    assert_eq!(
        schema_slot_precision(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        schema_slot_recall(&[], &one),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        schema_slot_precision(&one, &[]),
        Err(EventError::InvalidWirePayload)
    );
    let duplicate = [
        slot(EventRoleKind::Agent, "procurement office"),
        slot(EventRoleKind::Agent, "procurement office"),
    ];
    assert_eq!(
        schema_slot_recall(&duplicate, &duplicate),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        SchemaSlotAssignment::new(EventRoleKind::Agent, "   "),
        Err(EventError::InvalidWirePayload)
    );
    assert!((schema_slot_precision(&one, &one).expect("singleton") - 1.0).abs() < f64::EPSILON);
}

#[test]
fn labels_round_trip_and_threshold_is_inclusive() {
    assert_eq!(SchemaSlotLabel::Filled.wire_name(), "filled");
    assert_eq!(SchemaSlotLabel::Empty.wire_name(), "empty");
    assert_eq!(
        SchemaSlotLabel::from_wire_name("filled").expect("parse"),
        SchemaSlotLabel::Filled
    );
    assert_eq!(
        SchemaSlotLabel::from_wire_name("empty").expect("parse"),
        SchemaSlotLabel::Empty
    );
    assert_eq!(
        SchemaSlotLabel::from_wire_name("maybe_slot"),
        Err(EventError::UnknownSchemaSlotLabel)
    );
    assert!(SchemaSlotLabel::Filled.is_filled());
    assert!(!SchemaSlotLabel::Empty.is_filled());
    assert!((SchemaSlotLabel::Filled.as_probability_target() - 1.0).abs() < f64::EPSILON);
    assert!((SchemaSlotLabel::Empty.as_probability_target() - 0.0).abs() < f64::EPSILON);

    let half = EventConfidence::new(0.5).expect("half");
    assert_eq!(decide_schema_slot(half, half), SchemaSlotLabel::Filled);
    assert_eq!(
        decide_schema_slot(EventConfidence::new(0.49).expect("below"), half),
        SchemaSlotLabel::Empty
    );

    let assigned = slot(EventRoleKind::Place, "seoul");
    assert_eq!(assigned.role(), EventRoleKind::Place);
    assert_eq!(assigned.argument(), "seoul");
    assert_eq!(SchemaPredictionId::from_raw(7).raw(), 7);
}
