//! Half-open event-time intervals and parent containment.

use crate::SubeventContainmentError;

/// One half-open event-time interval `[start, end)` in seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventInterval {
    start_seconds: i64,
    end_seconds: i64,
}

impl EventInterval {
    /// Construct a half-open interval with a strictly positive length.
    ///
    /// # Errors
    ///
    /// Returns [`SubeventContainmentError::InvalidIntervalPayload`] when
    /// `end_seconds` is not greater than `start_seconds`.
    pub const fn new(
        start_seconds: i64,
        end_seconds: i64,
    ) -> Result<Self, SubeventContainmentError> {
        if end_seconds <= start_seconds {
            return Err(SubeventContainmentError::InvalidIntervalPayload);
        }
        Ok(Self {
            start_seconds,
            end_seconds,
        })
    }

    /// Inclusive start bound in seconds.
    #[must_use]
    pub const fn start_seconds(self) -> i64 {
        self.start_seconds
    }

    /// Exclusive end bound in seconds.
    #[must_use]
    pub const fn end_seconds(self) -> i64 {
        self.end_seconds
    }
}

/// Return whether `child` lies entirely inside `parent`.
///
/// # Errors
///
/// This function is infallible for validated intervals and exists to keep the
/// public comparison surface explicit.
#[allow(clippy::unnecessary_wraps)]
pub fn interval_contains(
    parent: EventInterval,
    child: EventInterval,
) -> Result<bool, SubeventContainmentError> {
    Ok(child.start_seconds >= parent.start_seconds && child.end_seconds <= parent.end_seconds)
}

/// Refuse to attach a subevent that escapes the parent interval.
///
/// # Errors
///
/// Returns [`SubeventContainmentError::SubeventEscapesParent`] when the child
/// is not contained.
pub fn refuse_escaped_subevent(
    parent: EventInterval,
    child: EventInterval,
) -> Result<(), SubeventContainmentError> {
    if interval_contains(parent, child)? {
        return Ok(());
    }
    Err(SubeventContainmentError::SubeventEscapesParent)
}

/// Fraction of recovered containment flags that match known truth.
///
/// # Errors
///
/// Returns [`SubeventContainmentError::InvalidIntervalPayload`] when either
/// slice is empty or the lengths differ.
pub fn containment_recovery_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, SubeventContainmentError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(SubeventContainmentError::InvalidIntervalPayload);
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
        EventInterval, containment_recovery_rate, interval_contains, refuse_escaped_subevent,
    };
    use crate::SubeventContainmentError;

    #[test]
    fn local_branches_cover_containment_and_payloads() {
        let parent = EventInterval::new(10, 40).expect("parent");
        let inside = EventInterval::new(15, 30).expect("inside");
        assert_eq!(parent.start_seconds(), 10);
        assert_eq!(parent.end_seconds(), 40);
        assert!(interval_contains(parent, inside).expect("inside"));
        refuse_escaped_subevent(parent, inside).expect("contained");
        let early = EventInterval::new(0, 20).expect("early");
        assert!(!interval_contains(parent, early).expect("early"));
        assert_eq!(
            refuse_escaped_subevent(parent, early),
            Err(SubeventContainmentError::SubeventEscapesParent)
        );
        assert_eq!(
            EventInterval::new(4, 4),
            Err(SubeventContainmentError::InvalidIntervalPayload)
        );
        let matched = containment_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            containment_recovery_rate(&[], &[]),
            Err(SubeventContainmentError::InvalidIntervalPayload)
        );
        assert_eq!(
            containment_recovery_rate(&[true], &[]),
            Err(SubeventContainmentError::InvalidIntervalPayload)
        );
    }
}
