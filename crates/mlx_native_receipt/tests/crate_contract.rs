//! Integration contract for the `mlx_native_receipt` package identity.

#[test]
fn package_identity_is_stable() {
    let observed = std::hint::black_box(env!("CARGO_PKG_NAME"));
    assert_eq!(observed, "mlx_native_receipt");
}
