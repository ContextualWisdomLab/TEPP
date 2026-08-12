//! Document generation with reporting delays, method effects, and memberships.

use crate::SimulationError;
use temporal_core::{AvailableTime, DocumentTime, EventTime};
use uuid::Uuid;

/// Governed method-effect labels for generated document variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DocumentMethodEffect {
    /// Original report of an event.
    Original,
    /// Revision of an earlier document.
    Revision,
    /// Translation of an earlier document.
    Translation,
    /// Template- or copy-derived variant.
    TemplateCopy,
}

impl DocumentMethodEffect {
    /// Stable wire name for the method effect.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Revision => "revision",
            Self::Translation => "translation",
            Self::TemplateCopy => "template_copy",
        }
    }

    /// Return whether this effect is a derivative of a parent document.
    #[must_use]
    pub const fn is_derivative(self) -> bool {
        !matches!(self, Self::Original)
    }
}

/// One simulated membership assignment attached to a document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulatedMembership {
    group_id: Uuid,
    role_label: &'static str,
    weight_bps: u32,
}

impl SimulatedMembership {
    /// Construct a membership truth row.
    #[must_use]
    pub const fn new(group_id: Uuid, role_label: &'static str, weight_bps: u32) -> Self {
        Self {
            group_id,
            role_label,
            weight_bps,
        }
    }

    /// Group identity.
    #[must_use]
    pub const fn group_id(&self) -> Uuid {
        self.group_id
    }

    /// Contextual role label (not a permanent entity class).
    #[must_use]
    pub const fn role_label(&self) -> &'static str {
        self.role_label
    }

    /// Membership weight in basis points.
    #[must_use]
    pub const fn weight_bps(&self) -> u32 {
        self.weight_bps
    }
}

/// One simulated document observation of a latent event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulatedDocument {
    document_id: Uuid,
    event_id: Uuid,
    document_time: DocumentTime,
    available_time: AvailableTime,
    method_effect: DocumentMethodEffect,
    parent_document_id: Option<Uuid>,
    observed_event_time: Option<EventTime>,
    memberships: Vec<SimulatedMembership>,
}

impl SimulatedDocument {
    /// Construct a validated simulated document.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::TemporalInvariantViolation`] when availability
    /// precedes document time, or when a derivative lacks a parent / an original
    /// carries a parent.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document_id: Uuid,
        event_id: Uuid,
        document_time: DocumentTime,
        available_time: AvailableTime,
        method_effect: DocumentMethodEffect,
        parent_document_id: Option<Uuid>,
        observed_event_time: Option<EventTime>,
        memberships: Vec<SimulatedMembership>,
    ) -> Result<Self, SimulationError> {
        if available_time.instant() < document_time.instant() {
            return Err(SimulationError::TemporalInvariantViolation);
        }
        match (method_effect.is_derivative(), parent_document_id.is_some()) {
            (true, false) | (false, true) => {
                return Err(SimulationError::TemporalInvariantViolation);
            }
            (true, true) | (false, false) => {}
        }
        Ok(Self::trusted(
            document_id,
            event_id,
            document_time,
            available_time,
            method_effect,
            parent_document_id,
            observed_event_time,
            memberships,
        ))
    }

    /// Construct a document when the caller already enforces domain invariants.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn trusted(
        document_id: Uuid,
        event_id: Uuid,
        document_time: DocumentTime,
        available_time: AvailableTime,
        method_effect: DocumentMethodEffect,
        parent_document_id: Option<Uuid>,
        observed_event_time: Option<EventTime>,
        memberships: Vec<SimulatedMembership>,
    ) -> Self {
        Self {
            document_id,
            event_id,
            document_time,
            available_time,
            method_effect,
            parent_document_id,
            observed_event_time,
            memberships,
        }
    }

    /// Document identity.
    #[must_use]
    pub const fn document_id(&self) -> Uuid {
        self.document_id
    }

    /// Linked latent event identity.
    #[must_use]
    pub const fn event_id(&self) -> Uuid {
        self.event_id
    }

    /// Document creation/reporting time.
    #[must_use]
    pub const fn document_time(&self) -> DocumentTime {
        self.document_time
    }

    /// Availability time for historical eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }

    /// Method-effect label.
    #[must_use]
    pub const fn method_effect(&self) -> DocumentMethodEffect {
        self.method_effect
    }

    /// Parent document for derivative variants.
    #[must_use]
    pub const fn parent_document_id(&self) -> Option<Uuid> {
        self.parent_document_id
    }

    /// Observed event time after missingness (if any).
    #[must_use]
    pub const fn observed_event_time(&self) -> Option<EventTime> {
        self.observed_event_time
    }

    /// Multiple-membership assignments.
    #[must_use]
    pub fn memberships(&self) -> &[SimulatedMembership] {
        &self.memberships
    }
}

/// Hours spanned by the synthetic non-wrapping calendar (`12 * 28 * 24`).
pub const SYNTHETIC_YEAR_HOURS: u32 = 12 * 28 * 24;

/// Construct event, document, and available times with non-negative delays.
///
/// Delays are whole hours after a linear hour index so all clocks remain exact
/// RFC 3339 instants without private temporal constructors. Indices must remain
/// inside the synthetic non-wrapping calendar so order is total.
///
/// # Errors
///
/// Returns [`SimulationError::TemporalInvariantViolation`] when the schedule
/// would wrap the synthetic year or timestamps cannot be parsed.
pub fn delayed_clocks(
    event_hour_index: u32,
    report_delay_hours: u32,
    availability_delay_hours: u32,
) -> Result<(EventTime, DocumentTime, AvailableTime), SimulationError> {
    let document_index = event_hour_index.saturating_add(report_delay_hours);
    let available_index = document_index.saturating_add(availability_delay_hours);
    if available_index >= SYNTHETIC_YEAR_HOURS {
        return Err(SimulationError::TemporalInvariantViolation);
    }
    let event = EventTime::parse_rfc3339(&hour_stamp(event_hour_index))
        .map_err(|_| SimulationError::TemporalInvariantViolation)?;
    let document = DocumentTime::parse_rfc3339(&hour_stamp(document_index))
        .map_err(|_| SimulationError::TemporalInvariantViolation)?;
    let available = AvailableTime::parse_rfc3339(&hour_stamp(available_index))
        .map_err(|_| SimulationError::TemporalInvariantViolation)?;
    Ok((event, document, available))
}

fn hour_stamp(hour_index: u32) -> String {
    debug_assert!(hour_index < SYNTHETIC_YEAR_HOURS);
    let hour_of_day = hour_index % 24;
    let day_index = hour_index / 24;
    let day = (day_index % 28) + 1;
    let month = (day_index / 28) + 1;
    format!("2026-{month:02}-{day:02}T{hour_of_day:02}:00:00Z")
}

/// Role vocabulary used for multilevel membership generation.
#[must_use]
pub fn membership_role_at(index: usize) -> &'static str {
    const ROLES: [&str; 6] = [
        "author",
        "department",
        "organization",
        "project",
        "template",
        "episode",
    ];
    ROLES[index % ROLES.len()]
}

#[cfg(test)]
mod tests {
    use super::{
        DocumentMethodEffect, SYNTHETIC_YEAR_HOURS, SimulatedDocument, SimulatedMembership,
        delayed_clocks, membership_role_at,
    };
    use crate::SimulationError;
    use temporal_core::{AvailableTime, DocumentTime, EventTime};
    use uuid::Uuid;

    #[test]
    fn reporting_and_availability_delays_preserve_forward_order() {
        let (event, document, available) = delayed_clocks(10, 5, 3).expect("times");
        assert!(document.instant() >= event.instant());
        assert!(available.instant() >= document.instant());
        assert_eq!(membership_role_at(0), "author");
        assert_eq!(membership_role_at(6), "author");
        assert_eq!(DocumentMethodEffect::Original.wire_name(), "original");
        assert_eq!(DocumentMethodEffect::Revision.wire_name(), "revision");
        assert_eq!(DocumentMethodEffect::Translation.wire_name(), "translation");
        assert_eq!(
            DocumentMethodEffect::TemplateCopy.wire_name(),
            "template_copy"
        );
        assert!(!DocumentMethodEffect::Original.is_derivative());
        assert!(DocumentMethodEffect::Revision.is_derivative());
        assert_eq!(
            delayed_clocks(SYNTHETIC_YEAR_HOURS - 1, 1, 0),
            Err(SimulationError::TemporalInvariantViolation)
        );
        assert_eq!(
            delayed_clocks(SYNTHETIC_YEAR_HOURS, 0, 0),
            Err(SimulationError::TemporalInvariantViolation)
        );
    }

    #[test]
    fn document_rejects_bad_parent_and_availability_order() {
        let event = EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("e");
        let document_time = DocumentTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("d");
        let available_time = AvailableTime::parse_rfc3339("2026-01-03T00:00:00Z").expect("a");
        let early_available = AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("early");
        let membership = SimulatedMembership::new(Uuid::nil(), "author", 10_000);
        assert_eq!(membership.group_id(), Uuid::nil());
        assert_eq!(membership.role_label(), "author");
        assert_eq!(membership.weight_bps(), 10_000);

        assert_eq!(
            SimulatedDocument::new(
                Uuid::nil(),
                Uuid::nil(),
                document_time,
                early_available,
                DocumentMethodEffect::Original,
                None,
                Some(event),
                vec![membership.clone()],
            ),
            Err(SimulationError::TemporalInvariantViolation)
        );
        assert_eq!(
            SimulatedDocument::new(
                Uuid::nil(),
                Uuid::nil(),
                document_time,
                available_time,
                DocumentMethodEffect::Revision,
                None,
                Some(event),
                vec![membership.clone()],
            ),
            Err(SimulationError::TemporalInvariantViolation)
        );
        assert_eq!(
            SimulatedDocument::new(
                Uuid::nil(),
                Uuid::nil(),
                document_time,
                available_time,
                DocumentMethodEffect::Original,
                Some(Uuid::nil()),
                Some(event),
                vec![membership.clone()],
            ),
            Err(SimulationError::TemporalInvariantViolation)
        );

        let doc = SimulatedDocument::new(
            Uuid::nil(),
            Uuid::nil(),
            document_time,
            available_time,
            DocumentMethodEffect::Original,
            None,
            Some(event),
            vec![membership],
        )
        .expect("doc");
        assert_eq!(doc.document_id(), Uuid::nil());
        assert_eq!(doc.event_id(), Uuid::nil());
        assert_eq!(doc.document_time(), document_time);
        assert_eq!(doc.available_time(), available_time);
        assert_eq!(doc.method_effect(), DocumentMethodEffect::Original);
        assert_eq!(doc.parent_document_id(), None);
        assert_eq!(doc.observed_event_time(), Some(event));
        assert_eq!(doc.memberships().len(), 1);

        let revision = SimulatedDocument::new(
            Uuid::nil(),
            Uuid::nil(),
            document_time,
            available_time,
            DocumentMethodEffect::Revision,
            Some(Uuid::max()),
            None,
            Vec::new(),
        )
        .expect("revision");
        assert_eq!(revision.parent_document_id(), Some(Uuid::max()));
        assert_eq!(revision.observed_event_time(), None);
    }
}
