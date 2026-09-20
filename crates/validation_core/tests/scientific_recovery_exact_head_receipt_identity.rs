//! Exact-head receipt identity must commit every field that changes receipt meaning.

use validation_core::{
    ScientificRecoveryExactHeadReceiptStatusV1, ScientificRecoveryExactHeadReceiptV1,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const OTHER_HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ARTIFACT: &str = "5555555555555555555555555555555555555555555555555555555555555555";

fn receipt(
    head: &str,
    status: ScientificRecoveryExactHeadReceiptStatusV1,
) -> ScientificRecoveryExactHeadReceiptV1 {
    ScientificRecoveryExactHeadReceiptV1::new(head, ARTIFACT, status)
        .expect("canonical receipt input")
}

#[test]
fn receipt_identity_binds_head_and_terminal_state() {
    let passed = receipt(HEAD, ScientificRecoveryExactHeadReceiptStatusV1::Passed);
    let failed = receipt(HEAD, ScientificRecoveryExactHeadReceiptStatusV1::Failed);
    let other_head = receipt(
        OTHER_HEAD,
        ScientificRecoveryExactHeadReceiptStatusV1::Passed,
    );
    let repeated = receipt(HEAD, ScientificRecoveryExactHeadReceiptStatusV1::Passed);

    assert_eq!(passed.receipt_sha256(), repeated.receipt_sha256());
    assert_ne!(passed.receipt_sha256(), failed.receipt_sha256());
    assert_ne!(passed.receipt_sha256(), other_head.receipt_sha256());
}
