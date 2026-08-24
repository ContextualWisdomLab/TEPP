//! Integration contract for the `document_clocks` package identity.

#[test]
fn package_identity_is_stable() {
    let observed = std::hint::black_box(env!("CARGO_PKG_NAME"));
    assert_eq!(observed, "document_clocks");
}
