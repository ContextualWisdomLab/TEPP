//! Selective disclosure cannot leak unauthorized fields or blanket-mask measurement.

use selective_disclosure::{
    DisclosedFieldSet, DisclosurePurpose, FIELD_AUTHOR_ROLE, FIELD_DIRECT_IDENTITY,
    FIELD_EVENT_TIME, FIELD_MEMBERSHIP_ROLE, FIELD_OPAQUE_ID, FIELD_SOURCE_TEXT,
    SelectiveDisclosureError, disclose, disclosure_recovery_rate, refuse_blanket_mask,
};

fn scientific_source() -> [u16; 5] {
    [
        FIELD_AUTHOR_ROLE,
        FIELD_EVENT_TIME,
        FIELD_MEMBERSHIP_ROLE,
        FIELD_DIRECT_IDENTITY,
        FIELD_SOURCE_TEXT,
    ]
}

#[test]
fn scientific_purpose_keeps_linkage_and_refuses_identity() {
    let disclosed = disclose(
        DisclosurePurpose::ScientificValidation,
        &scientific_source(),
        &[
            FIELD_AUTHOR_ROLE,
            FIELD_EVENT_TIME,
            FIELD_MEMBERSHIP_ROLE,
            FIELD_OPAQUE_ID,
        ],
    );
    assert_eq!(disclosed, Err(SelectiveDisclosureError::MissingSourceField));
}

#[test]
fn scientific_purpose_keeps_present_linkage() {
    let source = [
        FIELD_AUTHOR_ROLE,
        FIELD_EVENT_TIME,
        FIELD_MEMBERSHIP_ROLE,
        FIELD_DIRECT_IDENTITY,
        FIELD_SOURCE_TEXT,
        FIELD_OPAQUE_ID,
    ];
    let disclosed = disclose(
        DisclosurePurpose::ScientificValidation,
        &source,
        &[
            FIELD_AUTHOR_ROLE,
            FIELD_EVENT_TIME,
            FIELD_MEMBERSHIP_ROLE,
            FIELD_OPAQUE_ID,
        ],
    )
    .expect("scientific linkage");
    assert_eq!(disclosed.purpose(), DisclosurePurpose::ScientificValidation);
    assert_eq!(
        disclosed.fields(),
        &[
            FIELD_AUTHOR_ROLE,
            FIELD_EVENT_TIME,
            FIELD_MEMBERSHIP_ROLE,
            FIELD_OPAQUE_ID
        ]
    );
}

#[test]
fn omitting_scientific_linkage_is_a_blanket_mask() {
    assert_eq!(
        disclose(
            DisclosurePurpose::ScientificValidation,
            &scientific_source(),
            &[FIELD_OPAQUE_ID],
        ),
        Err(SelectiveDisclosureError::MissingSourceField)
    );
    let source = [
        FIELD_AUTHOR_ROLE,
        FIELD_EVENT_TIME,
        FIELD_MEMBERSHIP_ROLE,
        FIELD_OPAQUE_ID,
    ];
    assert_eq!(
        disclose(
            DisclosurePurpose::ScientificValidation,
            &source,
            &[FIELD_OPAQUE_ID],
        ),
        Err(SelectiveDisclosureError::BlanketMaskDestroysMeasurement)
    );
    assert_eq!(
        refuse_blanket_mask(),
        Err(SelectiveDisclosureError::BlanketMaskDestroysMeasurement)
    );
}

#[test]
fn identity_and_source_text_require_reidentification_purpose() {
    let source = scientific_source();
    assert_eq!(
        disclose(
            DisclosurePurpose::ScientificValidation,
            &source,
            &[
                FIELD_AUTHOR_ROLE,
                FIELD_EVENT_TIME,
                FIELD_MEMBERSHIP_ROLE,
                FIELD_DIRECT_IDENTITY
            ],
        ),
        Err(SelectiveDisclosureError::UnauthorizedField)
    );
    assert_eq!(
        disclose(
            DisclosurePurpose::OperationalMonitoring,
            &[FIELD_SOURCE_TEXT, FIELD_OPAQUE_ID],
            &[FIELD_SOURCE_TEXT],
        ),
        Err(SelectiveDisclosureError::UnauthorizedField)
    );
    let exported = disclose(
        DisclosurePurpose::ReidentificationExport,
        &source,
        &[FIELD_DIRECT_IDENTITY, FIELD_SOURCE_TEXT],
    )
    .expect("re-id");
    assert_eq!(
        exported.fields(),
        &[FIELD_DIRECT_IDENTITY, FIELD_SOURCE_TEXT]
    );
}

#[test]
fn operational_monitoring_may_omit_linkage() {
    let disclosed = disclose(
        DisclosurePurpose::OperationalMonitoring,
        &[FIELD_AUTHOR_ROLE, FIELD_OPAQUE_ID],
        &[FIELD_OPAQUE_ID],
    )
    .expect("ops");
    assert_eq!(
        disclosed.purpose(),
        DisclosurePurpose::OperationalMonitoring
    );
    assert_eq!(disclosed.fields(), &[FIELD_OPAQUE_ID]);
}

#[test]
fn unknown_empty_or_duplicate_payloads_fail_closed() {
    assert_eq!(
        disclose(
            DisclosurePurpose::ScientificValidation,
            &[],
            &[FIELD_OPAQUE_ID]
        ),
        Err(SelectiveDisclosureError::InvalidDisclosurePayload)
    );
    assert_eq!(
        disclose(
            DisclosurePurpose::ScientificValidation,
            &[FIELD_OPAQUE_ID],
            &[]
        ),
        Err(SelectiveDisclosureError::InvalidDisclosurePayload)
    );
    assert_eq!(
        disclose(
            DisclosurePurpose::ScientificValidation,
            &[FIELD_OPAQUE_ID, 99],
            &[FIELD_OPAQUE_ID],
        ),
        Err(SelectiveDisclosureError::InvalidDisclosurePayload)
    );
    assert_eq!(
        disclose(
            DisclosurePurpose::ScientificValidation,
            &[FIELD_OPAQUE_ID],
            &[FIELD_OPAQUE_ID, FIELD_OPAQUE_ID],
        ),
        Err(SelectiveDisclosureError::InvalidDisclosurePayload)
    );
}

#[test]
fn recovered_field_sets_match_known_truth_better_than_a_mask_collapse() {
    let source = [
        FIELD_AUTHOR_ROLE,
        FIELD_EVENT_TIME,
        FIELD_MEMBERSHIP_ROLE,
        FIELD_OPAQUE_ID,
    ];
    let truth =
        [disclose(DisclosurePurpose::ScientificValidation, &source, &source).expect("truth")];
    let recovered =
        [disclose(DisclosurePurpose::ScientificValidation, &source, &source).expect("recovered")];
    let collapsed =
        [
            DisclosedFieldSet::new(DisclosurePurpose::ScientificValidation, &[FIELD_OPAQUE_ID])
                .expect("collapse"),
        ];
    let recovered_rate = disclosure_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = disclosure_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_record, decided_record) in truth.iter().zip(recovered.iter()) {
            if truth_record == decided_record {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
    assert_eq!(
        disclosure_recovery_rate(&[], &[]),
        Err(SelectiveDisclosureError::InvalidDisclosurePayload)
    );
    assert_eq!(
        disclosure_recovery_rate(&truth, &[]),
        Err(SelectiveDisclosureError::InvalidDisclosurePayload)
    );
}
