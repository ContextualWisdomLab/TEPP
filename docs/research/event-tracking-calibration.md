# Event-tracking calibration

## Scope

This note doctors the `event_core` gate that keeps TDT tracking distinct from event-instance promotion and state-transition authority:

1. a hypothesized track assignment is measurement evidence, not a promoted instance or transition;
2. pair precision, pair recall, and identity-switch rate are computed from known-truth assignments;
3. calibrated same-track probabilities recover the binary same-track target with lower RMSE than an always-one-track detector.

No database migration is allocated. Later TDT/CHRONOS layers may consume these scores as measurement evidence only.

## Authoritative sources

Allan, J., Carbonell, J., Doddington, G., Yamron, J., & Yang, Y. (1998). Topic detection and tracking pilot study: Final report. In *Proceedings of the DARPA Broadcast News Transcription and Understanding Workshop* (pp. 194–218).

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Fiscus, J. G., & Doddington, G. R. (2002). Topic detection and tracking evaluation overview. In J. Allan (Ed.), *Topic detection and tracking: Event-based information organization* (pp. 17–31). Kluwer Academic Publishers.

## Application

Allan et al. (1998) and Allan (2002) define topic tracking as a longitudinal *same-topic / same-story* assignment task whose official evaluation reports miss, false-alarm, and tracking-cost trade-offs rather than instance identity. Fiscus and Doddington (2002) keep those tracking scores in the measurement layer. TEPP therefore refuses to cast a track assignment as an event instance or a forward state transition and requires computed pair precision, pair recall, identity-switch rate, and RMSE against known truth (Allan et al., 1998; Allan, 2002; Fiscus & Doddington, 2002).

## Verification

- `refuse_track_as_instance` always returns `EventTrackIsNotEventInstance`;
- `refuse_track_as_transition` always returns `EventTrackIsNotStateTransition`;
- `tracking_pair_precision` and `tracking_pair_recall` fail closed on empty, mismatched, duplicate-mention, or pairless assignment streams;
- `tracking_identity_switch_rate` fails closed when no consecutive truth pair stays on the same track;
- computed RMSE of known same-track pair targets is lower under calibrated probabilities than under an always-one-track detector.
