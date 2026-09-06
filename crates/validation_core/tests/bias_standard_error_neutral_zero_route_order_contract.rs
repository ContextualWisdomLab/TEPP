#![forbid(unsafe_code)]
//! Guards the exact bias-SE proof route against reintroducing quadratic work ahead
//! of the neutral-zero linear kernel.

const SOURCE: &str = include_str!("../src/bias_se.rs");

#[test]
fn neutral_zero_linear_proof_precedes_quadratic_fallbacks() {
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
    let conditioned_anchor = route
        .find("exact_anchor_linear_pair_square_sum(&residuals)")
        .expect("conditioned observed-anchor fallback must remain available");
    let pairwise = route
        .find("exact_pairwise_pair_square_sum(&residuals)")
        .expect("pairwise reference fallback must remain available");

    assert!(
        neutral_zero < conditioned_anchor && conditioned_anchor < pairwise,
        "production proof order must be neutral-zero linear -> conditioned anchor -> pairwise reference"
    );
}
