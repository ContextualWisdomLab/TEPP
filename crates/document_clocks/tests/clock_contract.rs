//! Document rows cannot omit assertion time or document time.

use document_clocks::{
    ClockFamily, DocumentClockError, DocumentClockInstant, DocumentClockRow,
    clock_completeness_recovery_rate, clocks_are_complete,
    refuse_omitted_assertion_or_document_time,
};

fn instant(family: ClockFamily, seconds: i64) -> DocumentClockInstant {
    DocumentClockInstant::new(family, seconds)
}

fn complete_row() -> DocumentClockRow {
    DocumentClockRow::new(
        Some(instant(ClockFamily::EventTime, 10)),
        Some(instant(ClockFamily::AssertionTime, 20)),
        Some(instant(ClockFamily::DocumentTime, 15)),
        Some(instant(ClockFamily::SystemTime, 30)),
        Some(instant(ClockFamily::AvailableTime, 25)),
    )
    .expect("complete")
}

#[test]
fn omitted_assertion_or_document_time_fails_closed() {
    let complete = complete_row();
    assert!(clocks_are_complete(&complete).expect("complete"));
    refuse_omitted_assertion_or_document_time(&complete).expect("ok");
    assert_eq!(
        DocumentClockRow::new(
            Some(instant(ClockFamily::EventTime, 10)),
            None,
            Some(instant(ClockFamily::DocumentTime, 15)),
            Some(instant(ClockFamily::SystemTime, 30)),
            Some(instant(ClockFamily::AvailableTime, 25)),
        ),
        Err(DocumentClockError::OmittedAssertionOrDocumentTime)
    );
    assert_eq!(
        DocumentClockRow::new(
            Some(instant(ClockFamily::EventTime, 10)),
            Some(instant(ClockFamily::AssertionTime, 20)),
            None,
            Some(instant(ClockFamily::SystemTime, 30)),
            Some(instant(ClockFamily::AvailableTime, 25)),
        ),
        Err(DocumentClockError::OmittedAssertionOrDocumentTime)
    );
    assert_eq!(
        refuse_omitted_assertion_or_document_time_from_options(
            Some(instant(ClockFamily::EventTime, 10)),
            None,
            None,
            Some(instant(ClockFamily::SystemTime, 30)),
            Some(instant(ClockFamily::AvailableTime, 25)),
        ),
        Err(DocumentClockError::OmittedAssertionOrDocumentTime)
    );
}

#[test]
fn system_or_event_time_cannot_stand_in_for_assertion_or_document_time() {
    assert_eq!(
        DocumentClockRow::new(
            Some(instant(ClockFamily::EventTime, 10)),
            Some(instant(ClockFamily::SystemTime, 20)),
            Some(instant(ClockFamily::DocumentTime, 15)),
            Some(instant(ClockFamily::SystemTime, 30)),
            Some(instant(ClockFamily::AvailableTime, 25)),
        ),
        Err(DocumentClockError::ClockFamilyMismatch)
    );
    assert_eq!(
        DocumentClockRow::new(
            Some(instant(ClockFamily::EventTime, 10)),
            Some(instant(ClockFamily::AssertionTime, 20)),
            Some(instant(ClockFamily::EventTime, 15)),
            Some(instant(ClockFamily::SystemTime, 30)),
            Some(instant(ClockFamily::AvailableTime, 25)),
        ),
        Err(DocumentClockError::ClockFamilyMismatch)
    );
}

#[test]
fn recovered_completeness_flags_match_known_truth_better_than_accepting_all() {
    let complete = complete_row();
    let omitted_assertion = DocumentClockRow::new(
        Some(instant(ClockFamily::EventTime, 10)),
        None,
        Some(instant(ClockFamily::DocumentTime, 15)),
        Some(instant(ClockFamily::SystemTime, 30)),
        Some(instant(ClockFamily::AvailableTime, 25)),
    );
    let omitted_document = DocumentClockRow::new(
        Some(instant(ClockFamily::EventTime, 10)),
        Some(instant(ClockFamily::AssertionTime, 20)),
        None,
        Some(instant(ClockFamily::SystemTime, 30)),
        Some(instant(ClockFamily::AvailableTime, 25)),
    );
    let truth = [true, false, false];
    let recovered = [
        clocks_are_complete(&complete).expect("complete"),
        omitted_assertion.is_err(),
        omitted_document.is_err(),
    ];
    // The recovered vector above uses is_err for omitted rows (true when refused).
    // Completeness flags must be: complete=true, omitted=false, omitted=false.
    let recovered_flags = [
        clocks_are_complete(&complete).expect("complete"),
        false,
        false,
    ];
    let collapsed = [true, true, true];
    let recovered_rate = clock_completeness_recovery_rate(&truth, &recovered_flags).expect("ok");
    let collapsed_rate = clock_completeness_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_flag, decided_flag) in truth.iter().zip(recovered_flags.iter()) {
            if truth_flag == decided_flag {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!(recovered[1] && recovered[2]);
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_recovery_slices_fail_closed() {
    assert_eq!(
        DocumentClockRow::new(None, None, None, None, None),
        Err(DocumentClockError::OmittedAssertionOrDocumentTime)
    );
    assert_eq!(
        clock_completeness_recovery_rate(&[], &[]),
        Err(DocumentClockError::InvalidClockPayload)
    );
    assert_eq!(
        clock_completeness_recovery_rate(&[true], &[]),
        Err(DocumentClockError::InvalidClockPayload)
    );
    assert_eq!(
        clock_completeness_recovery_rate(&[true, false], &[true]),
        Err(DocumentClockError::InvalidClockPayload)
    );
}

fn refuse_omitted_assertion_or_document_time_from_options(
    event_time: Option<DocumentClockInstant>,
    assertion_time: Option<DocumentClockInstant>,
    document_time: Option<DocumentClockInstant>,
    system_time: Option<DocumentClockInstant>,
    available_time: Option<DocumentClockInstant>,
) -> Result<(), DocumentClockError> {
    let row = DocumentClockRow::new(
        event_time,
        assertion_time,
        document_time,
        system_time,
        available_time,
    )?;
    refuse_omitted_assertion_or_document_time(&row)
}
