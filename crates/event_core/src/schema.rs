//! CHRONOS schema-slot predictions stay distinct from instances and transitions.

use crate::{EventConfidence, EventError, EventInstanceId, EventRoleKind};
use std::collections::BTreeSet;

/// Opaque CHRONOS schema-prediction identity.
///
/// A schema prediction is a hypothesized slot-fill. It is never a promoted
/// event instance and cannot create a forward state transition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchemaPredictionId(u32);

impl SchemaPredictionId {
    /// Reconstruct a prediction identity from a raw fixture or estimator label.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Return the raw prediction label.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// CHRONOS filled-versus-empty occupancy for one schema slot.
///
/// A fill decision is prediction evidence. It is never a promoted event
/// instance and cannot create a forward state transition by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchemaSlotLabel {
    /// The slot is scored as occupied by a filler.
    Filled,
    /// The slot is scored as unoccupied.
    Empty,
}

impl SchemaSlotLabel {
    /// Return the stable wire label name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Filled => "filled",
            Self::Empty => "empty",
        }
    }

    /// Parse a stable wire schema-slot occupancy label.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::UnknownSchemaSlotLabel`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, EventError> {
        match name {
            "filled" => Ok(Self::Filled),
            "empty" => Ok(Self::Empty),
            _ => Err(EventError::UnknownSchemaSlotLabel),
        }
    }

    /// Return whether this label marks a filled slot.
    #[must_use]
    pub const fn is_filled(self) -> bool {
        matches!(self, Self::Filled)
    }

    /// Return the binary probability target used for RMSE.
    ///
    /// Filled truth is `1.0`; empty truth is `0.0`.
    #[must_use]
    pub const fn as_probability_target(self) -> f64 {
        match self {
            Self::Filled => 1.0,
            Self::Empty => 0.0,
        }
    }
}

/// Predicted or observed filler for one CHRONOS schema slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaSlotAssignment {
    role: EventRoleKind,
    argument: String,
}

impl SchemaSlotAssignment {
    /// Bind a role to a hypothesized filler argument.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::InvalidWirePayload`] when the argument is empty
    /// or whitespace-only.
    pub fn new(role: EventRoleKind, argument: impl Into<String>) -> Result<Self, EventError> {
        let argument = argument.into();
        let argument = argument.trim();
        if argument.is_empty() {
            return Err(EventError::InvalidWirePayload);
        }
        Ok(Self {
            role,
            argument: argument.to_string(),
        })
    }

    /// Return the typed role for this slot.
    #[must_use]
    pub const fn role(&self) -> EventRoleKind {
        self.role
    }

    /// Return the hypothesized filler argument.
    #[must_use]
    pub fn argument(&self) -> &str {
        &self.argument
    }
}

/// Threshold a slot-occupancy probability into a filled/empty label.
///
/// The threshold is inclusive: `probability >= threshold` fills the slot.
#[must_use]
pub fn decide_schema_slot(
    probability: EventConfidence,
    threshold: EventConfidence,
) -> SchemaSlotLabel {
    if probability.value() >= threshold.value() {
        SchemaSlotLabel::Filled
    } else {
        SchemaSlotLabel::Empty
    }
}

/// Explicit refusal to treat a CHRONOS schema prediction as an event instance.
///
/// # Errors
///
/// Always returns [`EventError::SchemaPredictionIsNotEventInstance`].
pub fn refuse_schema_prediction_as_instance(
    _prediction: SchemaPredictionId,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::SchemaPredictionIsNotEventInstance)
}

/// Explicit refusal to treat a CHRONOS schema prediction as a state transition.
///
/// # Errors
///
/// Always returns [`EventError::SchemaPredictionIsNotStateTransition`].
pub fn refuse_schema_prediction_as_transition(
    _prediction: SchemaPredictionId,
) -> Result<(), EventError> {
    Err(EventError::SchemaPredictionIsNotStateTransition)
}

/// Precision of recovered filled slots against known truth fills.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when either fill set is empty
/// or a `(role, argument)` pair is duplicated.
pub fn schema_slot_precision(
    truth: &[SchemaSlotAssignment],
    recovered: &[SchemaSlotAssignment],
) -> Result<f64, EventError> {
    let truth_slots = unique_slot_set(truth)?;
    let recovered_slots = unique_slot_set(recovered)?;
    counted_rate(
        recovered_slots.intersection(&truth_slots).count(),
        recovered_slots.len(),
    )
}

/// Recall of recovered filled slots against known truth fills.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when either fill set is empty
/// or a `(role, argument)` pair is duplicated.
pub fn schema_slot_recall(
    truth: &[SchemaSlotAssignment],
    recovered: &[SchemaSlotAssignment],
) -> Result<f64, EventError> {
    let truth_slots = unique_slot_set(truth)?;
    let recovered_slots = unique_slot_set(recovered)?;
    counted_rate(
        recovered_slots.intersection(&truth_slots).count(),
        truth_slots.len(),
    )
}

fn unique_slot_set(
    assignments: &[SchemaSlotAssignment],
) -> Result<BTreeSet<(EventRoleKind, String)>, EventError> {
    if assignments.is_empty() {
        return Err(EventError::InvalidWirePayload);
    }
    let mut slots = BTreeSet::new();
    for assignment in assignments {
        if !slots.insert((assignment.role(), assignment.argument().to_string())) {
            return Err(EventError::InvalidWirePayload);
        }
    }
    Ok(slots)
}

fn counted_rate(numerator: usize, denominator: usize) -> Result<f64, EventError> {
    let numerator = u32::try_from(numerator).map_err(|_| EventError::InvalidWirePayload)?;
    let denominator = u32::try_from(denominator).map_err(|_| EventError::InvalidWirePayload)?;
    if denominator == 0 {
        return Err(EventError::InvalidWirePayload);
    }
    Ok(f64::from(numerator) / f64::from(denominator))
}

#[cfg(test)]
mod tests {
    use super::{
        SchemaPredictionId, SchemaSlotAssignment, SchemaSlotLabel, counted_rate,
        decide_schema_slot, refuse_schema_prediction_as_instance,
        refuse_schema_prediction_as_transition, schema_slot_precision, schema_slot_recall,
    };
    use crate::{EventConfidence, EventError, EventRoleKind};

    fn filled(role: EventRoleKind, argument: &str) -> SchemaSlotAssignment {
        SchemaSlotAssignment::new(role, argument).expect("slot")
    }

    #[test]
    fn schema_helpers_cover_local_branches() {
        let prediction = SchemaPredictionId::from_raw(3);
        assert_eq!(
            refuse_schema_prediction_as_instance(prediction),
            Err(EventError::SchemaPredictionIsNotEventInstance)
        );
        assert_eq!(
            refuse_schema_prediction_as_transition(prediction),
            Err(EventError::SchemaPredictionIsNotStateTransition)
        );
        let high = EventConfidence::new(0.8).expect("high");
        let low = EventConfidence::new(0.2).expect("low");
        assert_eq!(decide_schema_slot(high, low), SchemaSlotLabel::Filled);
        assert_eq!(decide_schema_slot(low, high), SchemaSlotLabel::Empty);
        let truth = [filled(EventRoleKind::Agent, "procurement office")];
        assert!((schema_slot_precision(&truth, &truth).expect("p") - 1.0).abs() < f64::EPSILON);
        assert!((schema_slot_recall(&truth, &truth).expect("r") - 1.0).abs() < f64::EPSILON);
        assert_eq!(counted_rate(0, 0), Err(EventError::InvalidWirePayload));
        assert_eq!(
            counted_rate(usize::MAX, 1),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            counted_rate(1, usize::MAX),
            Err(EventError::InvalidWirePayload)
        );
        assert!((counted_rate(1, 2).expect("half") - 0.5).abs() < f64::EPSILON);
    }
}
