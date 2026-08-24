//! Integration contract for the `prediction_contradiction` package identity.

#[test]
fn package_identity_is_stable() {
    let observed = std::hint::black_box(env!("CARGO_PKG_NAME"));
    assert_eq!(observed, "prediction_contradiction");
}
