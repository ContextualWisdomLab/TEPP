//! Integration contract for the `copy_identity` package identity.

#[test]
fn package_identity_is_stable() {
    let observed = std::hint::black_box(env!("CARGO_PKG_NAME"));
    assert_eq!(observed, "copy_identity");
}
