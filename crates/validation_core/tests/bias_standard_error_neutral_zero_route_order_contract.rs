#![forbid(unsafe_code)]
//! Guards the exact bias-SE proof route against quadratic work without demonstrated
//! represented-input admission value.

const SOURCE: &str = include_str!("../src/bias_se.rs");

#[test]
fn neutral_zero_linear_proof_precedes_only_the_pairwise_reference() {
    let start = SOURCE
        .find("fn exact_pair_distance_standard_error(")
        .expect("production exact-proof entrypoint must exist");
    let end = SOURCE[start..]
        .find("pub fn bias_standard_error(")
        .map(|offset| start + offset)
        .expect("public bias-SE entrypoint must follow the exact-proof helper");
    let route = &SOURCE[start..end];

    let neutral_zero = route
        .find("exact_neutral_zero_linear_pair_square_sum(&residuals)")
        .expect("neutral-zero O(n) proof must be a production route");
    let pairwise = route
        .find("exact_pairwise_pair_square_sum(&residuals)")
        .expect("pairwise fail-closed reference must remain available");

    assert!(
        !route.contains("exact_anchor_linear_pair_square_sum(&residuals)"),
        "do not retain an O(n²) conditioned-anchor scan without a represented fixture that uniquely recovers a neutral-zero refusal"
    );
    assert!(
        neutral_zero < pairwise,
        "production proof order must be neutral-zero linear -> pairwise reference"
    );
}
