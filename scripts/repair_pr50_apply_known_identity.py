"""Apply the PR 50 event-status and known-identity baseline repair."""

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    """Replace one exact fragment or fail closed."""
    file_path = Path(path)
    text = file_path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement target, found {count}")
    file_path.write_text(text.replace(old, new, 1), encoding="utf-8")


INTELLIGENCE = r'''//! Evidence-status gates and oracle-assisted story-identity baselines.

use crate::EventError;

/// Epistemic layer of an event-intelligence output.
///
/// Only [`EventEvidenceLayer::PromotedTransition`] may enter the forward
/// state/input-process-outcome graph. TDT detections and CHRONOS predictions
/// remain measurement or hypothesis artifacts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EventEvidenceLayer {
    /// Fallible textual mention grounded in evidence.
    ObservedMention,
    /// TDT-style detection, link, or track output.
    TdtDetection,
    /// CHRONOS-style schema completion or predicted event.
    ChronosPrediction,
    /// Symbolic temporal-consistency judgment.
    TemporalConsistency,
    /// Independently promoted forward state transition.
    PromotedTransition,
}

impl EventEvidenceLayer {
    /// Stable wire name for this layer.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ObservedMention => "observed_mention",
            Self::TdtDetection => "tdt_detection",
            Self::ChronosPrediction => "chronos_prediction",
            Self::TemporalConsistency => "temporal_consistency",
            Self::PromotedTransition => "promoted_transition",
        }
    }

    /// Whether this layer may admit a forward state-transition edge.
    #[must_use]
    pub const fn may_admit_state_transition(self) -> bool {
        matches!(self, Self::PromotedTransition)
    }
}

/// Admit a layer into the forward state graph or fail closed.
///
/// # Errors
///
/// Returns [`EventError::PredictionIsNotFact`] for CHRONOS predictions and
/// [`EventError::DetectionIsNotTransition`] for every other non-promoted layer.
pub fn admit_state_transition(layer: EventEvidenceLayer) -> Result<(), EventError> {
    if layer.may_admit_state_transition() {
        Ok(())
    } else if matches!(layer, EventEvidenceLayer::ChronosPrediction) {
        Err(EventError::PredictionIsNotFact)
    } else {
        Err(EventError::DetectionIsNotTransition)
    }
}

/// Oracle-assisted identity-baseline decision for one candidate story.
///
/// This baseline assumes a stable gold or externally adjudicated story identity.
/// It is not a detector over raw text, embeddings, or document metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnownIdentityStoryDecision {
    /// The externally supplied story identity has not appeared before.
    FirstOccurrence,
    /// The externally supplied story identity already appeared in the stream.
    RepeatedIdentity,
}

/// Classify one externally identified story against identities already observed.
///
/// This function is an oracle-assisted baseline for scoring and regression
/// tests. Product code must not present it as first-story detection from raw
/// documents.
#[must_use]
pub fn classify_known_identity_baseline(
    seen_story_ids: &[u64],
    candidate_story_id: u64,
) -> KnownIdentityStoryDecision {
    if seen_story_ids.contains(&candidate_story_id) {
        KnownIdentityStoryDecision::RepeatedIdentity
    } else {
        KnownIdentityStoryDecision::FirstOccurrence
    }
}

/// Known-truth first-story detection counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirstStoryRates {
    hits: usize,
    misses: usize,
    false_alarms: usize,
    first_story_truth: usize,
    continuation_truth: usize,
}

impl FirstStoryRates {
    /// Correct first-story detections.
    #[must_use]
    pub const fn hits(self) -> usize {
        self.hits
    }

    /// Missed first stories.
    #[must_use]
    pub const fn misses(self) -> usize {
        self.misses
    }

    /// Continuations labeled as first stories.
    #[must_use]
    pub const fn false_alarms(self) -> usize {
        self.false_alarms
    }

    /// Miss rate among true first stories.
    #[must_use]
    pub fn miss_rate(self) -> f64 {
        if self.first_story_truth == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                self.misses as f64 / self.first_story_truth as f64
            }
        }
    }

    /// False-alarm rate among true continuations.
    #[must_use]
    pub fn false_alarm_rate(self) -> f64 {
        if self.continuation_truth == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                self.false_alarms as f64 / self.continuation_truth as f64
            }
        }
    }
}

/// Score predicted first-story labels against an independently supplied truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the streams are empty or
/// have unequal length.
pub fn first_story_detection_rates(
    truth_is_first: &[bool],
    predicted_is_first: &[bool],
) -> Result<FirstStoryRates, EventError> {
    if truth_is_first.is_empty() || truth_is_first.len() != predicted_is_first.len() {
        return Err(EventError::InvalidWirePayload);
    }
    let mut hits = 0;
    let mut misses = 0;
    let mut false_alarms = 0;
    let mut first_story_truth = 0;
    let mut continuation_truth = 0;
    for (&truth, &predicted) in truth_is_first.iter().zip(predicted_is_first) {
        if truth {
            first_story_truth += 1;
            if predicted {
                hits += 1;
            } else {
                misses += 1;
            }
        } else {
            continuation_truth += 1;
            if predicted {
                false_alarms += 1;
            }
        }
    }
    Ok(FirstStoryRates {
        hits,
        misses,
        false_alarms,
        first_story_truth,
        continuation_truth,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        EventEvidenceLayer, FirstStoryRates, KnownIdentityStoryDecision,
        classify_known_identity_baseline, first_story_detection_rates,
    };

    #[test]
    fn zero_denominator_rates_are_zero_and_identity_baseline_is_explicit() {
        assert_eq!(
            classify_known_identity_baseline(&[7], 7),
            KnownIdentityStoryDecision::RepeatedIdentity
        );
        assert_eq!(
            classify_known_identity_baseline(&[7], 8),
            KnownIdentityStoryDecision::FirstOccurrence
        );
        let empty_classes = FirstStoryRates {
            hits: 0,
            misses: 0,
            false_alarms: 0,
            first_story_truth: 0,
            continuation_truth: 0,
        };
        assert!(empty_classes.miss_rate() < 1e-15);
        assert!(empty_classes.false_alarm_rate() < 1e-15);
        let all_first = first_story_detection_rates(&[true, true], &[true, false]).expect("all");
        assert!((all_first.miss_rate() - 0.5).abs() < 1e-15);
        assert!(all_first.false_alarm_rate() < 1e-15);
        assert_eq!(
            EventEvidenceLayer::TdtDetection.wire_name(),
            "tdt_detection"
        );
    }
}
'''

Path("crates/event_core/src/intelligence.rs").write_text(INTELLIGENCE, encoding="utf-8")

replace_once(
    "crates/event_core/src/error.rs",
    """    /// An unknown event-role name was supplied.
    UnknownEventRole,
""",
    """    /// An unknown event-role name was supplied.
    UnknownEventRole,
    /// A TDT detection or mention was treated as a state transition.
    DetectionIsNotTransition,
    /// A CHRONOS prediction was treated as an observed or promoted fact.
    PredictionIsNotFact,
""",
)
replace_once(
    "crates/event_core/src/error.rs",
    """            Self::UnknownEventRole => \"unknown event role\",
""",
    """            Self::UnknownEventRole => \"unknown event role\",
            Self::DetectionIsNotTransition => \"detection is not a state transition\",
            Self::PredictionIsNotFact => \"prediction is not an observed fact\",
""",
)
replace_once(
    "crates/event_core/src/error.rs",
    """            (EventError::UnknownEventRole, \"unknown event role\"),
""",
    """            (EventError::UnknownEventRole, \"unknown event role\"),
            (
                EventError::DetectionIsNotTransition,
                \"detection is not a state transition\",
            ),
            (
                EventError::PredictionIsNotFact,
                \"prediction is not an observed fact\",
            ),
""",
)

replace_once(
    "crates/event_core/src/lib.rs",
    """mod identifier;
mod instance;
""",
    """mod identifier;
mod intelligence;
mod instance;
""",
)
replace_once(
    "crates/event_core/src/lib.rs",
    """/// Opaque event-instance identifier.
pub use identifier::EventInstanceId;
""",
    """/// Opaque event-instance identifier.
pub use identifier::EventInstanceId;
/// Admit only independently promoted state transitions.
pub use intelligence::admit_state_transition;
/// Oracle-assisted classification using externally supplied story identities.
pub use intelligence::classify_known_identity_baseline;
/// Score first-story predictions against independent known truth.
pub use intelligence::first_story_detection_rates;
/// Epistemic layer for event-intelligence output.
pub use intelligence::EventEvidenceLayer;
/// First-story miss and false-alarm summary.
pub use intelligence::FirstStoryRates;
/// Decision from the oracle-assisted known-identity baseline.
pub use intelligence::KnownIdentityStoryDecision;
""",
)

RESEARCH = r'''# Event-intelligence status gates and known-identity baseline

## Scope

This note doctors the first ADR 0016 production slice in `event_core`:

1. every event-intelligence output carries an epistemic layer (`observed_mention`, `tdt_detection`, `chronos_prediction`, `temporal_consistency`, `promoted_transition`);
2. only an independently promoted transition may enter the forward state graph;
3. CHRONOS predictions are never treated as observed fact;
4. generic first-story miss and false-alarm rates are scored against an independently supplied truth vector;
5. the committed story-identity classifier is explicitly an oracle-assisted known-identity baseline, not a detector over raw documents.

Full raw-text TDT detection, linking, tracking, calibration, and CHRONOS schema extraction remain accepted-target. No database migration is allocated.

## Authoritative sources

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

## Application

Allan (2002) defines first-story detection as a scored measurement task with miss and false-alarm rates, not as automatic promotion into a chronology. A repeated externally supplied story identifier can provide a deterministic oracle baseline for regression testing, but it cannot establish detection performance from text because the identity already contains the answer. Anagnostopoulos et al. (2013) keep qualitative temporal reasoning distinct from asserted event identity. TEPP therefore refuses to admit TDT detections or CHRONOS predictions as state transitions, exposes generic rate scoring against independent truth, and labels identity-membership logic as a baseline rather than a detector.

## Verification

- `admit_state_transition(PromotedTransition)` succeeds;
- TDT/mention/consistency layers return `DetectionIsNotTransition`;
- CHRONOS predictions return `PredictionIsNotFact`;
- the identity stream `[10,20,10,30,20]` is scored against the independently fixed truth vector `[true,true,false,true,false]`;
- always-first and always-continuation predictions exercise false-alarm and miss paths;
- empty and unequal truth/prediction vectors fail closed;
- no product or scientific claim treats the known-identity baseline as raw-text first-story detection.
'''
Path("docs/research/event-intelligence-status-gates.md").write_text(RESEARCH, encoding="utf-8")

changelog_path = Path("CHANGELOG.md")
changelog = changelog_path.read_text(encoding="utf-8")
bullet = "- `event_core` ADR 0016 evidence-status gates: TDT detections and CHRONOS predictions cannot admit a forward state transition; generic first-story miss/false-alarm scoring is paired with an explicitly oracle-assisted known-identity baseline.\n"
if bullet not in changelog:
    marker = "### Added\n\n"
    if changelog.count(marker) != 1:
        raise SystemExit("CHANGELOG Added marker mismatch")
    changelog = changelog.replace(marker, marker + bullet, 1)
changelog_path.write_text(changelog, encoding="utf-8")

replace_once(
    "DOCUMENTATION.md",
    """| Actions fleet research doctoring | [`docs/research/actions-workflow-fleet.md`](docs/research/actions-workflow-fleet.md) |
""",
    """| Actions fleet research doctoring | [`docs/research/actions-workflow-fleet.md`](docs/research/actions-workflow-fleet.md) |
| Event-intelligence status-gate doctoring | [`docs/research/event-intelligence-status-gates.md`](docs/research/event-intelligence-status-gates.md) |
""",
)
replace_once(
    "docs/TRACEABILITY.md",
    """| event ontology/evidence mentions | PRD; ADR 0003 | `event_core` mention/instance separation on protected main; full intelligence stack remaining | partial |
""",
    """| event ontology/evidence mentions | PRD; ADR 0003 | `event_core` mention/instance separation on protected main; ADR 0016 evidence-status gates on the active PR | partial |
""",
)
replace_once(
    "docs/TRACEABILITY.md",
    """| TDT detection/tracking vs CHRONOS schema/prediction/temporal consistency | ADR 0016; PRD/research | future `event_intelligence` | accepted-target |
""",
    """| TDT detection/tracking vs CHRONOS schema/prediction/temporal consistency | ADR 0016; PRD/research | `event_core` admission gates, generic rates, and a known-identity baseline on the active PR; raw-text TDT/CHRONOS stack remaining | partial |
""",
)
replace_once(
    "docs/adr/0016-tdt-chronos-event-intelligence-boundary.md",
    "**Implementation maturity:** accepted-target  \n",
    "**Implementation maturity:** partial — evidence-layer admission gates, generic first-story rate scoring, and an oracle-assisted known-identity baseline are implemented on the active PR; raw-text TDT tracking/calibration and CHRONOS schema extraction remain accepted-target\n",
)
replace_once(
    "docs/adr/README.md",
    """| [0016](0016-tdt-chronos-event-intelligence-boundary.md) | TDT, CHRONOS, and Event Ontology intelligence boundary | Accepted | accepted-target | Separates observed evidence, detection/tracking, prediction/schema inference, temporal consistency, and promoted transition authority. |
""",
    """| [0016](0016-tdt-chronos-event-intelligence-boundary.md) | TDT, CHRONOS, and Event Ontology intelligence boundary | Accepted | partial | Admission gates, generic first-story rates, and a known-identity baseline are on the active PR; raw-text TDT/CHRONOS work remains accepted-target. |
""",
)
replace_once(
    "docs/validation/temporal-event-foundation.md",
    """| Versioned API/export contracts | `tepp_api` | implemented-main | — | unknown-field/version/limit tests | Task 12 / PR #21; HTTP service remaining |
""",
    """| Versioned API/export contracts | `tepp_api` | implemented-main | — | unknown-field/version/limit tests | Task 12 / PR #21; HTTP service remaining |
| TDT/CHRONOS evidence-status gates | `event_core` | active-PR | admission + generic first-story rates | independent truth + known-identity baseline | ADR 0016; `docs/research/event-intelligence-status-gates.md` |
""",
)
