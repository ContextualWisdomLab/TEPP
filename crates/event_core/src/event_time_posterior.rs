//! Exact event-time draws from a producer-owned discrete posterior measure.
//!
//! This module materializes, but does not invent, a CHRONOS event-time
//! posterior. The owning temporal estimator supplies integer posterior mass
//! over identified event-time atoms. Integer mass avoids consumer-selected
//! probability tolerances and makes every emitted draw auditable.

use temporal_core::EventTime;

/// One identified event-time atom and its exact posterior multiplicity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventTimePosteriorAtom {
    /// Event-clock instant, distinct from document and availability clocks.
    pub event_time: EventTime,
    /// Exact number of posterior draws assigned to this atom.
    pub multiplicity: u32,
}

/// A complete discrete event-time posterior draw set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventTimePosteriorDraws {
    /// Draws in canonical event-time order, preserving exact ties.
    pub draws: Vec<EventTime>,
}

/// Fail-closed event-time posterior materialization errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventTimePosteriorError {
    /// No posterior atom was supplied.
    EmptyPosterior,
    /// An atom carried zero posterior mass.
    ZeroMassAtom,
    /// Two atoms referred to the same event-clock instant.
    DuplicateEventTime,
    /// Total posterior multiplicity overflowed the supported address space.
    DrawCountOverflow,
}

/// Materialize exact event-time draws from identified integer posterior mass.
///
/// Atoms are sorted by event time so producer input order cannot change the
/// canonical draw digest. Equal instants remain repeated draws, representing
/// posterior mass and exact simultaneity rather than being collapsed.
///
/// # Errors
///
/// Fails closed for empty, zero-mass, duplicate, or overflowing input.
pub fn materialize_event_time_posterior(
    atoms: &[EventTimePosteriorAtom],
) -> Result<EventTimePosteriorDraws, EventTimePosteriorError> {
    if atoms.is_empty() {
        return Err(EventTimePosteriorError::EmptyPosterior);
    }
    if atoms.iter().any(|atom| atom.multiplicity == 0) {
        return Err(EventTimePosteriorError::ZeroMassAtom);
    }
    let mut ordered = atoms.to_vec();
    ordered.sort_unstable_by_key(|atom| atom.event_time);
    if ordered
        .windows(2)
        .any(|pair| pair[0].event_time == pair[1].event_time)
    {
        return Err(EventTimePosteriorError::DuplicateEventTime);
    }
    let draw_count = ordered.iter().try_fold(0_usize, |total, atom| {
        let multiplicity = usize::try_from(atom.multiplicity)
            .map_err(|_| EventTimePosteriorError::DrawCountOverflow)?;
        total
            .checked_add(multiplicity)
            .ok_or(EventTimePosteriorError::DrawCountOverflow)
    })?;
    let mut draws = Vec::with_capacity(draw_count);
    for atom in ordered {
        let multiplicity = usize::try_from(atom.multiplicity)
            .map_err(|_| EventTimePosteriorError::DrawCountOverflow)?;
        draws.extend(std::iter::repeat_n(atom.event_time, multiplicity));
    }
    Ok(EventTimePosteriorDraws { draws })
}
