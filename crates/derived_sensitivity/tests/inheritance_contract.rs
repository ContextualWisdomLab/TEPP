//! Derived artifacts cannot be declassified by derivation or a blanket mask.

use derived_sensitivity::{
    DerivedArtifact, DerivedSensitivityError, KIND_FACTOR, KIND_RELATION, KIND_TOPIC,
    SensitivityClass, inherit_sensitivity, refuse_blanket_mask_as_declassification,
    refuse_derivation_as_public, sensitivity_recovery_rate,
};

fn artifact(kind: u16, source: SensitivityClass) -> DerivedArtifact {
    DerivedArtifact::try_new(kind, source).expect("closed kind")
}

#[test]
fn derivation_and_blanket_mask_cannot_declassify() {
    assert_eq!(
        refuse_derivation_as_public(SensitivityClass::Restricted),
        Err(DerivedSensitivityError::DerivationIsNotDeclassification)
    );
    assert_eq!(
        refuse_blanket_mask_as_declassification(),
        Err(DerivedSensitivityError::BlanketMaskIsNotAuthorization)
    );
    let inherited = inherit_sensitivity(SensitivityClass::Restricted, 3).expect("inherit");
    assert_eq!(inherited.source_class(), SensitivityClass::Restricted);
    assert_eq!(inherited.kind_code(), 3);
}

#[test]
fn unvalidated_kind_codes_cannot_construct_artifacts() {
    assert_eq!(
        DerivedArtifact::try_new(99, SensitivityClass::Restricted),
        Err(DerivedSensitivityError::InvalidSensitivityPayload)
    );
    assert_eq!(
        DerivedArtifact::try_new(0, SensitivityClass::Public),
        Err(DerivedSensitivityError::InvalidSensitivityPayload)
    );
}

#[test]
fn inherited_classes_match_known_truth_better_than_a_public_collapse() {
    let kinds = [KIND_TOPIC, KIND_FACTOR, KIND_RELATION];
    let classes = [
        SensitivityClass::Restricted,
        SensitivityClass::Internal,
        SensitivityClass::Public,
    ];
    let mut truth = Vec::new();
    let mut inherited = Vec::new();
    let mut collapsed = Vec::new();
    for kind in kinds {
        for class in classes {
            truth.push(artifact(kind, class));
            inherited.push(inherit_sensitivity(class, kind).expect("inherit"));
            collapsed.push(artifact(kind, SensitivityClass::Public));
        }
    }
    let recovered_rate = sensitivity_recovery_rate(&truth, &inherited).expect("recovered");
    let collapsed_rate = sensitivity_recovery_rate(&truth, &collapsed).expect("collapsed");
    assert!((recovered_rate - 1.0).abs() < f64::EPSILON);
    assert!((collapsed_rate - (1.0 / 3.0)).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
    let mut reversed_truth = truth.clone();
    let mut reversed_inherited = inherited.clone();
    let mut reversed_collapsed = collapsed.clone();
    reversed_truth.reverse();
    reversed_inherited.reverse();
    reversed_collapsed.reverse();
    let reversed_rate =
        sensitivity_recovery_rate(&reversed_truth, &reversed_inherited).expect("reversed");
    let reversed_collapsed_rate =
        sensitivity_recovery_rate(&reversed_truth, &reversed_collapsed).expect("reversed");
    assert!((reversed_rate - recovered_rate).abs() < f64::EPSILON);
    assert!((reversed_collapsed_rate - collapsed_rate).abs() < f64::EPSILON);
    let kind_mismatch = [artifact(KIND_FACTOR, SensitivityClass::Restricted)];
    let topic_same_class = [artifact(KIND_TOPIC, SensitivityClass::Restricted)];
    let mismatch_rate =
        sensitivity_recovery_rate(&kind_mismatch, &topic_same_class).expect("kind mismatch");
    assert!((mismatch_rate - 1.0).abs() < f64::EPSILON);
}

#[test]
fn unknown_derived_kind_codes_fail_closed() {
    assert_eq!(
        inherit_sensitivity(SensitivityClass::Restricted, 99),
        Err(DerivedSensitivityError::InvalidSensitivityPayload)
    );
    assert_eq!(
        inherit_sensitivity(SensitivityClass::Internal, 0),
        Err(DerivedSensitivityError::InvalidSensitivityPayload)
    );
    assert_eq!(
        inherit_sensitivity(SensitivityClass::Public, 99),
        Err(DerivedSensitivityError::InvalidSensitivityPayload)
    );
}

#[test]
fn empty_or_mismatched_sensitivity_payloads_fail_closed() {
    assert_eq!(
        sensitivity_recovery_rate(&[], &[]),
        Err(DerivedSensitivityError::InvalidSensitivityPayload)
    );
    assert_eq!(
        sensitivity_recovery_rate(&[artifact(1, SensitivityClass::Restricted)], &[]),
        Err(DerivedSensitivityError::InvalidSensitivityPayload)
    );
    assert_eq!(
        sensitivity_recovery_rate(
            &[
                artifact(1, SensitivityClass::Restricted),
                artifact(2, SensitivityClass::Internal)
            ],
            &[artifact(1, SensitivityClass::Restricted)]
        ),
        Err(DerivedSensitivityError::InvalidSensitivityPayload)
    );
}
