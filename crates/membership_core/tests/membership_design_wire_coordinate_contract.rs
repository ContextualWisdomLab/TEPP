use membership_core::{
    MEMBERSHIP_DESIGN_WIRE_VERSION, MembershipDesign, MembershipDesignWire, MembershipError,
};

#[test]
fn owner_wire_coordinate_binds_supported_version_and_design_name() {
    let designs = [
        MembershipDesign::Nested,
        MembershipDesign::CrossClassified,
        MembershipDesign::MultipleMembership,
        MembershipDesign::CrossClassifiedMultipleMembership,
        MembershipDesign::HeterogeneousClassification,
        MembershipDesign::HeterogeneousClassificationMultipleMembership,
    ];

    for design in designs {
        let coordinate = MembershipDesignWire::from_design(design);
        assert_eq!(coordinate.version(), MEMBERSHIP_DESIGN_WIRE_VERSION);
        assert_eq!(coordinate.name(), design.wire_name());
        assert_eq!(coordinate.design(), design);
        assert_eq!(
            MembershipDesignWire::parse(coordinate.version(), coordinate.name())
                .expect("owner coordinate must round trip"),
            coordinate,
        );
    }
}

#[test]
fn owner_wire_coordinate_refuses_mismatched_version_even_for_valid_name() {
    assert_eq!(
        MembershipDesignWire::parse("tepp.membership_design.v2", "nested"),
        Err(MembershipError::UnsupportedWireVersion),
    );
}

#[test]
fn owner_wire_coordinate_refuses_unknown_name_under_supported_version() {
    assert_eq!(
        MembershipDesignWire::parse(MEMBERSHIP_DESIGN_WIRE_VERSION, "nested_probably"),
        Err(MembershipError::UnknownMembershipDesign),
    );
}
