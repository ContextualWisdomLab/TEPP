//! Integration contract for the `causal_language` package identity.

#[test]
fn package_identity_is_stable() {
    let observed = std::hint::black_box(env!("CARGO_PKG_NAME"));
    assert_eq!(observed, "causal_language");
}
