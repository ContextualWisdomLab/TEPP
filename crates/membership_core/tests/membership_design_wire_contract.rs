use membership_core::{MEMBERSHIP_DESIGN_WIRE_VERSION, MembershipDesign, MembershipError};

#[test]
fn membership_design_wire_contract_has_explicit_owner_version() {
    assert_eq!(MEMBERSHIP_DESIGN_WIRE_VERSION, "tepp.membership_design.v1");
}

#[test]
fn membership_design_wire_names_are_owner_issued_and_round_trip() {
    let cases = [
        (MembershipDesign::Nested, "nested"),
        (MembershipDesign::CrossClassified, "cross_classified"),
        (MembershipDesign::MultipleMembership, "multiple_membership"),
        (
            MembershipDesign::CrossClassifiedMultipleMembership,
            "cross_classified_multiple_membership",
        ),
        (
            MembershipDesign::HeterogeneousClassification,
            "heterogeneous_classification",
        ),
        (
            MembershipDesign::HeterogeneousClassificationMultipleMembership,
            "heterogeneous_classification_multiple_membership",
        ),
    ];

    for (design, expected_name) in cases {
        assert_eq!(design.wire_name(), expected_name);
        assert_eq!(MembershipDesign::from_wire_name(expected_name), Ok(design));
    }
}

#[test]
fn unknown_membership_design_wire_name_fails_closed() {
    assert_eq!(
        MembershipDesign::from_wire_name("nested_but_probably_fine"),
        Err(MembershipError::UnknownMembershipDesign),
    );
}
