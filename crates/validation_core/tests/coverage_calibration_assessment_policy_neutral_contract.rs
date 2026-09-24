#[test]
fn assessment_public_surface_does_not_mint_an_aggregate_calibration_claim() {
    let source = include_str!("../src/coverage.rs");

    assert!(
        !source.contains("pub const fn supports_calibration_claim"),
        "CoverageCalibrationAssessment must expose component criteria and denominator evidence without an aggregate claim boolean while numerical-failure acceptability is undeclared"
    );
}
