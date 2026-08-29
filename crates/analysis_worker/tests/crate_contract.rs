//! Package identity contract for the one-shot analysis worker.

#[test]
fn package_identity_is_stable() {
    assert_eq!(env!("CARGO_PKG_NAME"), "analysis_worker");
}
