# Event-link detection calibration

## Scope

This note doctors the `event_core` gate that keeps TDT link detection distinct from event-instance promotion and state-transition authority:

1. a linked versus unlinked mention pair is detection evidence, not a promoted instance or transition;
2. precision and recall are computed from known-truth pair sets;
3. calibrated link probabilities recover the binary same-event target with lower RMSE than an always-link detector.

No database migration is allocated. A later TDT tracker may consume these scores as measurement evidence only.

## Authoritative sources

Allan, J., Carbonell, J., Doddington, G., Yamron, J., & Yang, Y. (1998). Topic detection and tracking pilot study: Final report. In *Proceedings of the DARPA Broadcast News Transcription and Understanding Workshop* (pp. 194–218).

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Fiscus, J. G., & Doddington, G. R. (2002). Topic detection and tracking evaluation overview. In J. Allan (Ed.), *Topic detection and tracking: Event-based information organization* (pp. 17–31). Kluwer Academic Publishers.

## Application

Allan et al. (1998) and Allan (2002) define link detection as a *same-event / same-story* decision over story pairs. Fiscus and Doddington (2002) keep official TDT link scores in the measurement layer and report miss and false-alarm trade-offs rather than instance identity. TEPP therefore refuses to cast a detected link as an event instance or a forward state transition and requires computed precision, recall, and RMSE against known truth (Allan et al., 1998; Allan, 2002; Fiscus & Doddington, 2002).

## Verification

- `refuse_event_link_as_instance` always returns `EventLinkIsNotEventInstance`;
- `refuse_event_link_as_transition` always returns `EventLinkIsNotStateTransition`;
- `EventLinkPair::new` refuses a self-link and normalizes pair order;
- `event_link_precision` and `event_link_recall` fail closed on empty recovered or truth sets;
- computed RMSE of known link targets is lower under calibrated probabilities than under an always-link detector.
