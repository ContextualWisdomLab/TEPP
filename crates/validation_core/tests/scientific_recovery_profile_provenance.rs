//! Scientific recovery authority stays bound to one immutable, versioned design profile.

use validation_core::{
    ClaimAuthority, ScientificRecoveryFailurePolicyV1, ScientificRecoveryProfileV1,
    ValidationError, promote_scientific_recovery,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
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

#[test]
fn recovery_profile_identity_changes_with_predeclared_design_and_target() {
    let baseline = profile(2, 0.08);
    let different_denominator = profile(3, 0.08);
    let different_target = profile(2, 0.081);

    assert_ne!(baseline.sha256(), different_denominator.sha256());
    assert_ne!(baseline.sha256(), different_target.sha256());
    assert_eq!(baseline.failure_policy().wire_name(), "require_all_planned_recovered");

    let encoded = baseline.to_json().expect("canonical profile json");
    let decoded = ScientificRecoveryProfileV1::from_json(&encoded).expect("profile roundtrip");
    assert_eq!(decoded, baseline);
    assert_eq!(decoded.sha256(), baseline.sha256());
}

#[test]
fn malformed_or_noncanonical_dependency_digest_fails_closed() {
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
}

#[test]
fn promoted_scientific_authority_retains_exact_profile_identity() {
    let profile = profile(2, 0.08);
    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[0.0], [0.0]];
    let truth: Vec<&[f64]> = truth_rows.iter().map(|row| row.as_slice()).collect();
    let recovered: Vec<&[f64]> = recovered_rows.iter().map(|row| row.as_slice()).collect();

    let promotion = promote_scientific_recovery(HEAD, HEAD, &truth, &recovered, &profile)
        .expect("exact recovery under immutable profile");

    assert_eq!(promotion.claim().authority(), ClaimAuthority::ScientificallySupported);
    assert_eq!(promotion.profile_sha256(), profile.sha256());
}
