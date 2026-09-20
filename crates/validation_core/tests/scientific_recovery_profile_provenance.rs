//! Scientific recovery authority stays bound to one immutable, versioned design profile.

use validation_core::{
    ClaimAuthority, ScientificRecoveryExactHeadReceiptStatusV1,
    ScientificRecoveryExactHeadReceiptV1, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileV1, ValidationError, promote_scientific_recovery,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const RECEIPT: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SEEDS: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const ESTIMAND: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";

fn profile(planned_replications: usize, max_rmse: f64) -> ScientificRecoveryProfileV1 {
    ScientificRecoveryProfileV1::new(
        planned_replications,
        max_rmse,
        3.0,
        DGP,
        SEEDS,
        ESTIMAND,
        STATE,
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
    )
    .expect("valid recovery profile")
}

fn exact_head_receipt() -> ScientificRecoveryExactHeadReceiptV1 {
    ScientificRecoveryExactHeadReceiptV1::new(
        HEAD,
        RECEIPT,
        ScientificRecoveryExactHeadReceiptStatusV1::Passed,
    )
    .expect("valid exact-head receipt")
}

#[test]
fn recovery_profile_identity_changes_with_predeclared_design_and_target() {
    let baseline = profile(2, 0.08);
    let different_denominator = profile(3, 0.08);
    let different_target = profile(2, 0.081);

    assert_eq!(baseline.planned_replications(), 2);
    assert_eq!(baseline.max_rmse().to_bits(), 0.08_f64.to_bits());
    assert_eq!(baseline.se_multiplier().to_bits(), 3.0_f64.to_bits());
    assert_eq!(baseline.dgp_sha256(), DGP);
    assert_eq!(baseline.seed_manifest_sha256(), SEEDS);
    assert_eq!(baseline.estimand_sha256(), ESTIMAND);
    assert_eq!(baseline.state_composition_sha256(), STATE);
    assert_eq!(
        baseline.failure_policy(),
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered
    );
    assert_eq!(
        baseline.failure_policy().wire_name(),
        "require_all_planned_recovered"
    );

    assert_ne!(baseline.sha256(), different_denominator.sha256());
    assert_ne!(baseline.sha256(), different_target.sha256());

    let encoded = baseline.to_json().expect("canonical profile json");
    let decoded = ScientificRecoveryProfileV1::from_json(&encoded).expect("profile roundtrip");
    assert_eq!(decoded, baseline);
    assert_eq!(decoded.sha256(), baseline.sha256());
}

#[test]
fn malformed_or_noncanonical_profile_input_fails_closed() {
    let uppercase = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    assert_eq!(
        ScientificRecoveryProfileV1::new(
            2,
            0.08,
            3.0,
            uppercase,
            SEEDS,
            ESTIMAND,
            STATE,
            ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        ScientificRecoveryProfileV1::new(
            2,
            0.08,
            3.0,
            "abcd",
            SEEDS,
            ESTIMAND,
            STATE,
            ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        ScientificRecoveryProfileV1::from_json("not-json"),
        Err(ValidationError::InvalidInput)
    );

    let baseline = profile(2, 0.08);
    let encoded = baseline.to_json().expect("profile json");
    let wrong_schema = encoded.replace(
        "tepp.scientific_recovery_profile.v1",
        "tepp.scientific_recovery_profile.v2",
    );
    assert_eq!(
        ScientificRecoveryProfileV1::from_json(&wrong_schema),
        Err(ValidationError::InvalidInput)
    );

    let unknown_field = encoded.replacen('{', "{\"unexpected\":true,", 1);
    assert_eq!(
        ScientificRecoveryProfileV1::from_json(&unknown_field),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn negative_zero_uncertainty_multiplier_is_not_a_second_canonical_identity() {
    let canonical_zero = ScientificRecoveryProfileV1::new(
        2,
        0.08,
        0.0,
        DGP,
        SEEDS,
        ESTIMAND,
        STATE,
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
    )
    .expect("positive zero is the canonical zero multiplier");
    assert_eq!(canonical_zero.se_multiplier().to_bits(), 0.0_f64.to_bits());

    assert_eq!(
        ScientificRecoveryProfileV1::new(
            2,
            0.08,
            -0.0,
            DGP,
            SEEDS,
            ESTIMAND,
            STATE,
            ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
        ),
        Err(ValidationError::InvalidConfiguration)
    );
}

#[test]
fn promoted_scientific_authority_retains_profile_and_exact_head_receipt_identity() {
    let profile = profile(2, 0.08);
    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[0.0], [0.0]];
    let truth: Vec<&[f64]> = truth_rows.iter().map(|row| row.as_slice()).collect();
    let recovered: Vec<&[f64]> = recovered_rows.iter().map(|row| row.as_slice()).collect();

    let promotion = promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head_receipt(),
    )
    .expect("exact recovery under immutable profile and exact-head receipt");

    assert_eq!(
        promotion.claim().authority(),
        ClaimAuthority::ScientificallySupported
    );
    assert_eq!(promotion.profile_sha256(), profile.sha256());
    assert_eq!(promotion.exact_head_receipt_sha256(), RECEIPT);
}
