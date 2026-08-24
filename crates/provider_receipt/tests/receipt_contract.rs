//! Provider receipts cannot carry source text, identity, or a blanket mask.

use provider_receipt::{
    ProviderReceipt, ProviderReceiptError, receipt_recovery_rate,
    refuse_blanket_mask_as_disclosure, refuse_source_identity_in_receipt,
    refuse_source_text_in_receipt,
};

fn receipt(purpose: u16, fields: &[u16]) -> ProviderReceipt {
    ProviderReceipt::new(purpose, fields).expect("receipt")
}

#[test]
fn source_text_identity_and_blanket_mask_cannot_enter_a_receipt() {
    assert_eq!(
        refuse_source_text_in_receipt(),
        Err(ProviderReceiptError::SourceTextNotDisclosable)
    );
    assert_eq!(
        refuse_source_identity_in_receipt(),
        Err(ProviderReceiptError::SourceIdentityNotDisclosable)
    );
    assert_eq!(
        refuse_blanket_mask_as_disclosure(),
        Err(ProviderReceiptError::BlanketMaskIsNotAuthorization)
    );
}

#[test]
fn recovered_field_codes_match_known_truth_better_than_a_collapsed_set() {
    let truth = receipt(7, &[1, 2, 3]);
    let recovered = receipt(7, &[1, 2, 3]);
    let collapsed = receipt(7, &[1, 1, 1]);
    let recovered_rate = receipt_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = receipt_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_field, decided_field) in truth.field_codes().iter().zip(recovered.field_codes())
        {
            if truth_field == decided_field {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.field_codes().len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_receipt_payloads_fail_closed() {
    assert_eq!(
        ProviderReceipt::new(7, &[]),
        Err(ProviderReceiptError::InvalidReceiptPayload)
    );
    let truth = receipt(7, &[1, 2]);
    let short = receipt(7, &[1]);
    assert_eq!(
        receipt_recovery_rate(&truth, &short),
        Err(ProviderReceiptError::InvalidReceiptPayload)
    );
}
