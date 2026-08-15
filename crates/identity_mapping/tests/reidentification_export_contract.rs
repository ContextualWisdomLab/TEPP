//! Re-identification export cannot be replaced by analytical purpose or a blanket mask.

use identity_mapping::{
    IdentityMapRecord, IdentityMappingError, MappingPurpose, export_reidentification,
    mapping_recovery_rate, refuse_blanket_mask_as_reidentification,
};

fn record(analytical: u128, source: u128) -> IdentityMapRecord {
    IdentityMapRecord::new(analytical, source)
}

#[test]
fn analytical_purpose_cannot_export_source_identities() {
    let records = [record(1, 11), record(2, 22)];
    assert_eq!(
        export_reidentification(&records, MappingPurpose::AnalyticalComputation),
        Err(IdentityMappingError::UnauthorizedReidentification)
    );
    assert_eq!(
        refuse_blanket_mask_as_reidentification(),
        Err(IdentityMappingError::BlanketMaskIsNotAuthorization)
    );
}

#[test]
fn authorized_export_recovers_known_pairs_better_than_a_collapsed_map() {
    let truth = [record(1, 11), record(2, 22), record(3, 33)];
    let exported = export_reidentification(&truth, MappingPurpose::ReidentificationExport)
        .expect("authorized export");
    let collapsed = [record(1, 11), record(2, 11), record(3, 11)];
    let recovered_rate = mapping_recovery_rate(&truth, &exported).expect("recovered");
    let collapsed_rate = mapping_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_record, decided_record) in truth.iter().zip(exported.iter()) {
            if truth_record == decided_record {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_mapping_payloads_fail_closed() {
    assert_eq!(
        mapping_recovery_rate(&[], &[]),
        Err(IdentityMappingError::InvalidMappingPayload)
    );
    assert_eq!(
        mapping_recovery_rate(&[record(1, 11)], &[]),
        Err(IdentityMappingError::InvalidMappingPayload)
    );
    assert_eq!(
        mapping_recovery_rate(&[record(1, 11), record(2, 22)], &[record(1, 11)]),
        Err(IdentityMappingError::InvalidMappingPayload)
    );
    assert_eq!(
        export_reidentification(&[], MappingPurpose::ReidentificationExport),
        Err(IdentityMappingError::InvalidMappingPayload)
    );
}
