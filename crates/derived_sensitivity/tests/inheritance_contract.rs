//! Derived artifacts cannot be declassified by derivation or a blanket mask.

use derived_sensitivity::{
    DerivedArtifact, DerivedSensitivityError, SensitivityClass, inherit_sensitivity,
    refuse_blanket_mask_as_declassification, refuse_derivation_as_public,
    sensitivity_recovery_rate,
};

fn artifact(kind: u16, source: SensitivityClass) -> DerivedArtifact {
    DerivedArtifact::new(kind, source)
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
fn inherited_classes_match_known_truth_better_than_a_public_collapse() {
    let truth = [
        artifact(1, SensitivityClass::Restricted),
        artifact(2, SensitivityClass::Restricted),
        artifact(3, SensitivityClass::Internal),
    ];
    let inherited = [
        inherit_sensitivity(SensitivityClass::Restricted, 1).expect("t0"),
        inherit_sensitivity(SensitivityClass::Restricted, 2).expect("t1"),
        inherit_sensitivity(SensitivityClass::Internal, 3).expect("t2"),
    ];
    let collapsed = [
        artifact(1, SensitivityClass::Public),
        artifact(2, SensitivityClass::Public),
        artifact(3, SensitivityClass::Public),
    ];
    let recovered_rate = sensitivity_recovery_rate(&truth, &inherited).expect("recovered");
    let collapsed_rate = sensitivity_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_record, decided_record) in truth.iter().zip(inherited.iter()) {
            if truth_record == decided_record {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
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
