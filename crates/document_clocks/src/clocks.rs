//! Document-scoped clocks that cannot silently drop assertion or document time.

use crate::DocumentClockError;

/// Closed vocabulary of document-scoped clock families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockFamily {
    /// Event/valid time of the documented state.
    EventTime,
    /// Time the assertion was made.
    AssertionTime,
    /// Time stated by the document itself.
    DocumentTime,
    /// System/record time of the stored version.
    SystemTime,
    /// Time the evidence became available.
    AvailableTime,
}

/// One typed instant on a document-scoped clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentClockInstant {
    family: ClockFamily,
    epoch_seconds: i64,
}

impl DocumentClockInstant {
    /// Construct a typed instant.
    #[must_use]
    pub const fn new(family: ClockFamily, epoch_seconds: i64) -> Self {
        Self {
            family,
            epoch_seconds,
        }
    }

    /// Clock family label.
    #[must_use]
    pub const fn family(self) -> ClockFamily {
        self.family
    }

    /// Instant in seconds.
    #[must_use]
    pub const fn epoch_seconds(self) -> i64 {
        self.epoch_seconds
    }
}

/// One document row with the five document-scoped clocks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::struct_field_names)]
pub struct DocumentClockRow {
    event_time: DocumentClockInstant,
    assertion_time: DocumentClockInstant,
    document_time: DocumentClockInstant,
    system_time: DocumentClockInstant,
    available_time: DocumentClockInstant,
}

impl DocumentClockRow {
    /// Construct a document row that carries every required clock family.
    ///
    /// Event/valid time, system time, and availability time cannot stand in
    /// for omitted assertion time or document time.
    ///
    /// # Errors
    ///
    /// Returns [`DocumentClockError::OmittedAssertionOrDocumentTime`] when
    /// assertion time or document time is missing, [`DocumentClockError::InvalidClockPayload`]
    /// when event, system, or available time is missing, and
    /// [`DocumentClockError::ClockFamilyMismatch`] when a supplied instant
    /// uses the wrong family label.
    pub fn new(
        event_time: Option<DocumentClockInstant>,
        assertion_time: Option<DocumentClockInstant>,
        document_time: Option<DocumentClockInstant>,
        system_time: Option<DocumentClockInstant>,
        available_time: Option<DocumentClockInstant>,
    ) -> Result<Self, DocumentClockError> {
        let assertion_time =
            assertion_time.ok_or(DocumentClockError::OmittedAssertionOrDocumentTime)?;
        let document_time =
            document_time.ok_or(DocumentClockError::OmittedAssertionOrDocumentTime)?;
        let event_time = event_time.ok_or(DocumentClockError::InvalidClockPayload)?;
        let system_time = system_time.ok_or(DocumentClockError::InvalidClockPayload)?;
        let available_time = available_time.ok_or(DocumentClockError::InvalidClockPayload)?;
        require_family(event_time, ClockFamily::EventTime)?;
        require_family(assertion_time, ClockFamily::AssertionTime)?;
        require_family(document_time, ClockFamily::DocumentTime)?;
        require_family(system_time, ClockFamily::SystemTime)?;
        require_family(available_time, ClockFamily::AvailableTime)?;
        Ok(Self {
            event_time,
            assertion_time,
            document_time,
            system_time,
            available_time,
        })
    }

    /// Event/valid time.
    #[must_use]
    pub const fn event_time(self) -> DocumentClockInstant {
        self.event_time
    }

    /// Assertion time.
    #[must_use]
    pub const fn assertion_time(self) -> DocumentClockInstant {
        self.assertion_time
    }

    /// Document time.
    #[must_use]
    pub const fn document_time(self) -> DocumentClockInstant {
        self.document_time
    }

    /// System/record time.
    #[must_use]
    pub const fn system_time(self) -> DocumentClockInstant {
        self.system_time
    }

    /// Availability time.
    #[must_use]
    pub const fn available_time(self) -> DocumentClockInstant {
        self.available_time
    }
}

fn require_family(
    instant: DocumentClockInstant,
    expected: ClockFamily,
) -> Result<(), DocumentClockError> {
    if instant.family == expected {
        Ok(())
    } else {
        Err(DocumentClockError::ClockFamilyMismatch)
    }
}

/// Return whether every required clock family is present and labeled.
///
/// # Errors
///
/// Returns [`DocumentClockError::ClockFamilyMismatch`] when a stored instant
/// does not match its field.
pub fn clocks_are_complete(row: &DocumentClockRow) -> Result<bool, DocumentClockError> {
    require_family(row.event_time, ClockFamily::EventTime)?;
    require_family(row.assertion_time, ClockFamily::AssertionTime)?;
    require_family(row.document_time, ClockFamily::DocumentTime)?;
    require_family(row.system_time, ClockFamily::SystemTime)?;
    require_family(row.available_time, ClockFamily::AvailableTime)?;
    Ok(true)
}

/// Validate that a constructed document row retains every required clock.
///
/// This validator receives a constructed row, so omission is rejected by
/// [`DocumentClockRow::new`] before a row can reach this function.
///
/// # Errors
///
/// Returns the same family-mismatch error as [`clocks_are_complete`].
pub fn validate_complete_document_clock_row(
    row: &DocumentClockRow,
) -> Result<(), DocumentClockError> {
    clocks_are_complete(row).map(|_| ())
}

/// Fraction of recovered completeness flags that match known truth.
///
/// # Errors
///
/// Returns [`DocumentClockError::InvalidClockPayload`] when either slice is
/// empty or the lengths differ.
pub fn clock_completeness_recovery_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, DocumentClockError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(DocumentClockError::InvalidClockPayload);
    }
    let mut matches = 0_u32;
    for (truth_flag, decided_flag) in truth.iter().zip(decided) {
        if truth_flag == decided_flag {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        ClockFamily, DocumentClockInstant, DocumentClockRow, clock_completeness_recovery_rate,
        clocks_are_complete, validate_complete_document_clock_row,
    };
    use crate::DocumentClockError;

    fn sample_row() -> DocumentClockRow {
        DocumentClockRow::new(
            Some(DocumentClockInstant::new(ClockFamily::EventTime, 1)),
            Some(DocumentClockInstant::new(ClockFamily::AssertionTime, 2)),
            Some(DocumentClockInstant::new(ClockFamily::DocumentTime, 3)),
            Some(DocumentClockInstant::new(ClockFamily::SystemTime, 4)),
            Some(DocumentClockInstant::new(ClockFamily::AvailableTime, 5)),
        )
        .expect("row")
    }

    #[test]
    fn local_branches_cover_accessors() {
        let event = DocumentClockInstant::new(ClockFamily::EventTime, 1);
        assert_eq!(event.family(), ClockFamily::EventTime);
        assert_eq!(event.epoch_seconds(), 1);
        let row = sample_row();
        assert_eq!(row.event_time().epoch_seconds(), 1);
        assert_eq!(row.assertion_time().epoch_seconds(), 2);
        assert_eq!(row.document_time().epoch_seconds(), 3);
        assert_eq!(row.system_time().epoch_seconds(), 4);
        assert_eq!(row.available_time().epoch_seconds(), 5);
        assert!(clocks_are_complete(&row).expect("complete"));
        validate_complete_document_clock_row(&row).expect("ok");
    }

    #[test]
    fn omitted_required_clocks_fail_closed() {
        assert_eq!(
            DocumentClockRow::new(None, None, None, None, None),
            Err(DocumentClockError::OmittedAssertionOrDocumentTime)
        );
        assert_eq!(
            DocumentClockRow::new(
                None,
                Some(DocumentClockInstant::new(ClockFamily::AssertionTime, 2)),
                Some(DocumentClockInstant::new(ClockFamily::DocumentTime, 3)),
                Some(DocumentClockInstant::new(ClockFamily::SystemTime, 4)),
                Some(DocumentClockInstant::new(ClockFamily::AvailableTime, 5)),
            ),
            Err(DocumentClockError::InvalidClockPayload)
        );
        assert_eq!(
            DocumentClockRow::new(
                Some(DocumentClockInstant::new(ClockFamily::EventTime, 1)),
                Some(DocumentClockInstant::new(ClockFamily::AssertionTime, 2)),
                Some(DocumentClockInstant::new(ClockFamily::DocumentTime, 3)),
                None,
                Some(DocumentClockInstant::new(ClockFamily::AvailableTime, 5)),
            ),
            Err(DocumentClockError::InvalidClockPayload)
        );
        assert_eq!(
            DocumentClockRow::new(
                Some(DocumentClockInstant::new(ClockFamily::EventTime, 1)),
                Some(DocumentClockInstant::new(ClockFamily::AssertionTime, 2)),
                Some(DocumentClockInstant::new(ClockFamily::DocumentTime, 3)),
                Some(DocumentClockInstant::new(ClockFamily::SystemTime, 4)),
                None,
            ),
            Err(DocumentClockError::InvalidClockPayload)
        );
    }

    #[test]
    fn family_mismatches_fail_closed() {
        let event = DocumentClockInstant::new(ClockFamily::EventTime, 1);
        assert_eq!(
            DocumentClockRow::new(
                Some(event),
                Some(DocumentClockInstant::new(ClockFamily::SystemTime, 2)),
                Some(DocumentClockInstant::new(ClockFamily::DocumentTime, 3)),
                Some(DocumentClockInstant::new(ClockFamily::SystemTime, 4)),
                Some(DocumentClockInstant::new(ClockFamily::AvailableTime, 5)),
            ),
            Err(DocumentClockError::ClockFamilyMismatch)
        );
        assert_eq!(
            DocumentClockRow::new(
                Some(DocumentClockInstant::new(ClockFamily::SystemTime, 1)),
                Some(DocumentClockInstant::new(ClockFamily::AssertionTime, 2)),
                Some(DocumentClockInstant::new(ClockFamily::DocumentTime, 3)),
                Some(DocumentClockInstant::new(ClockFamily::SystemTime, 4)),
                Some(DocumentClockInstant::new(ClockFamily::AvailableTime, 5)),
            ),
            Err(DocumentClockError::ClockFamilyMismatch)
        );
        assert_eq!(
            DocumentClockRow::new(
                Some(event),
                Some(DocumentClockInstant::new(ClockFamily::AssertionTime, 2)),
                Some(DocumentClockInstant::new(ClockFamily::EventTime, 3)),
                Some(DocumentClockInstant::new(ClockFamily::SystemTime, 4)),
                Some(DocumentClockInstant::new(ClockFamily::AvailableTime, 5)),
            ),
            Err(DocumentClockError::ClockFamilyMismatch)
        );
        assert_eq!(
            DocumentClockRow::new(
                Some(event),
                Some(DocumentClockInstant::new(ClockFamily::AssertionTime, 2)),
                Some(DocumentClockInstant::new(ClockFamily::DocumentTime, 3)),
                Some(DocumentClockInstant::new(ClockFamily::EventTime, 4)),
                Some(DocumentClockInstant::new(ClockFamily::AvailableTime, 5)),
            ),
            Err(DocumentClockError::ClockFamilyMismatch)
        );
        assert_eq!(
            DocumentClockRow::new(
                Some(event),
                Some(DocumentClockInstant::new(ClockFamily::AssertionTime, 2)),
                Some(DocumentClockInstant::new(ClockFamily::DocumentTime, 3)),
                Some(DocumentClockInstant::new(ClockFamily::SystemTime, 4)),
                Some(DocumentClockInstant::new(ClockFamily::EventTime, 5)),
            ),
            Err(DocumentClockError::ClockFamilyMismatch)
        );
    }

    #[test]
    fn recovery_payloads_fail_closed() {
        let matched = clock_completeness_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            clock_completeness_recovery_rate(&[], &[]),
            Err(DocumentClockError::InvalidClockPayload)
        );
    }
}
