# Event-intelligence status gates

## Scope

This note doctors the first ADR 0016 production slice in `event_core`:

1. every event-intelligence output carries an epistemic layer (`observed_mention`, `tdt_detection`, `chronos_prediction`, `temporal_consistency`, `promoted_transition`);
2. only an independently promoted transition may enter the forward state graph;
3. CHRONOS predictions are never treated as observed fact;
4. the concrete instance-promotion API retains the accepted layer and rejects every non-promoted layer;
5. a first-story detector is scored with miss and false-alarm rates against a known story stream.

Full TDT tracking/calibration and CHRONOS schema extraction remain accepted-target. No database migration is allocated.

## Authoritative sources

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

## Application

Allan (2002) defines first-story detection as a scored measurement task with miss and false-alarm rates, not as automatic promotion into a chronology. Anagnostopoulos et al. (2013) keep qualitative temporal reasoning distinct from asserted event identity. TEPP therefore refuses to admit TDT detections or CHRONOS predictions as state transitions and reports computed first-story rates on a known stream (Allan, 2002; Anagnostopoulos et al., 2013).

## Verification

- `admit_state_transition(PromotedTransition)` succeeds;
- `EventInstance::promote_from_mentions` retains `PromotedTransition` and rejects observed mentions, TDT detections, CHRONOS predictions, and temporal-consistency judgments;
- TDT/mention/consistency layers return `DetectionIsNotTransition`;
- CHRONOS predictions return `PredictionIsNotFact`;
- stream `[10,20,10,30,20]` recovers three first stories with miss rate 0 and false-alarm rate 0;
- an always-first detector yields a computed false-alarm rate of 1.0 on the two continuations.
