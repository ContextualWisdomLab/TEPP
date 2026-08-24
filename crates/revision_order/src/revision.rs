//! Document revisions stamped with system time.

use crate::RevisionOrderError;

/// One document revision with a positive revision number and system time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentRevision {
    revision_number: u32,
    system_time_seconds: i64,
}

impl DocumentRevision {
    /// Construct a revision whose number is at least one.
    ///
    /// # Errors
    ///
    /// Returns [`RevisionOrderError::InvalidRevisionPayload`] when
    /// `revision_number` is zero.
    pub const fn new(
        revision_number: u32,
        system_time_seconds: i64,
    ) -> Result<Self, RevisionOrderError> {
        if revision_number == 0 {
            return Err(RevisionOrderError::InvalidRevisionPayload);
        }
        Ok(Self {
            revision_number,
            system_time_seconds,
        })
    }

    /// Positive revision number.
    #[must_use]
    pub const fn revision_number(self) -> u32 {
        self.revision_number
    }

    /// System/record time in seconds.
    #[must_use]
    pub const fn system_time_seconds(self) -> i64 {
        self.system_time_seconds
    }
}

/// Return whether `later` has a greater revision number and later system time.
///
/// # Errors
///
/// Returns [`RevisionOrderError::InvalidRevisionPayload`] when `later` is not
/// a strictly greater revision number than `earlier`.
pub fn revisions_are_increasing(
    earlier: DocumentRevision,
    later: DocumentRevision,
) -> Result<bool, RevisionOrderError> {
    if later.revision_number <= earlier.revision_number {
        return Err(RevisionOrderError::InvalidRevisionPayload);
    }
    Ok(later.system_time_seconds > earlier.system_time_seconds)
}

/// Refuse a later revision whose system time did not increase.
///
/// # Errors
///
/// Returns revision-construction errors, or
/// [`RevisionOrderError::SystemTimeDidNotIncrease`] when the system times
/// are not strictly increasing.
pub fn refuse_nonincreasing_system_time(
    earlier: DocumentRevision,
    later: DocumentRevision,
) -> Result<(), RevisionOrderError> {
    if revisions_are_increasing(earlier, later)? {
        return Ok(());
    }
    Err(RevisionOrderError::SystemTimeDidNotIncrease)
}

/// Fraction of recovered order flags that match known truth.
///
/// # Errors
///
/// Returns [`RevisionOrderError::InvalidRevisionPayload`] when either slice
/// is empty or the lengths differ.
pub fn order_recovery_rate(truth: &[bool], decided: &[bool]) -> Result<f64, RevisionOrderError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(RevisionOrderError::InvalidRevisionPayload);
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
        DocumentRevision, order_recovery_rate, refuse_nonincreasing_system_time,
        revisions_are_increasing,
    };
    use crate::RevisionOrderError;

    #[test]
    fn local_branches_cover_order_and_payloads() {
        let first = DocumentRevision::new(1, 10).expect("first");
        let second = DocumentRevision::new(2, 20).expect("second");
        assert_eq!(first.revision_number(), 1);
        assert_eq!(first.system_time_seconds(), 10);
        assert!(revisions_are_increasing(first, second).expect("increasing"));
        refuse_nonincreasing_system_time(first, second).expect("ok");
        let same_time = DocumentRevision::new(3, 20).expect("same");
        assert!(!revisions_are_increasing(second, same_time).expect("flat"));
        assert_eq!(
            refuse_nonincreasing_system_time(second, same_time),
            Err(RevisionOrderError::SystemTimeDidNotIncrease)
        );
        assert_eq!(
            revisions_are_increasing(second, first),
            Err(RevisionOrderError::InvalidRevisionPayload)
        );
        assert_eq!(
            DocumentRevision::new(0, 1),
            Err(RevisionOrderError::InvalidRevisionPayload)
        );
        let matched = order_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            order_recovery_rate(&[], &[]),
            Err(RevisionOrderError::InvalidRevisionPayload)
        );
        assert_eq!(
            order_recovery_rate(&[true], &[]),
            Err(RevisionOrderError::InvalidRevisionPayload)
        );
    }
}
