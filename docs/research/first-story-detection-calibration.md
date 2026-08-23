# First-story detection calibration

## Scope

This note doctors the `event_core` gate that keeps TDT first-story detection distinct from event-instance promotion:

1. a first-story versus follow-up label is detection evidence, not a promoted instance;
2. false-alarm and miss rates are computed from known-truth labels;
3. calibrated first-story probabilities recover the binary onset target with lower RMSE than an always-first detector.

No database migration is allocated. A later TDT tracker may consume these scores as measurement evidence only.

## Authoritative sources

Allan, J., Carbonell, J., Doddington, G., Yamron, J., & Yang, Y. (1998). Topic detection and tracking pilot study: Final report. In *Proceedings of the DARPA Broadcast News Transcription and Understanding Workshop* (pp. 194–218).

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Fiscus, J. G., & Doddington, G. R. (2002). Topic detection and tracking evaluation overview. In J. Allan (Ed.), *Topic detection and tracking: Event-based information organization* (pp. 17–31). Kluwer Academic Publishers.

## Application

Allan et al. (1998) and Allan (2002) define first-story detection as a *new-event onset* task whose official evaluation reports false-alarm and miss rates, not instance identity. Fiscus and Doddington (2002) keep those detection scores in the measurement layer. TEPP therefore refuses to cast a first-story label as an event instance and requires computed FAR, miss, and RMSE against known truth (Allan et al., 1998; Allan, 2002; Fiscus & Doddington, 2002).

## Verification

- `refuse_first_story_as_instance` always returns `FirstStoryIsNotEventInstance`;
- `decide_first_story` uses an inclusive probability threshold;
- `first_story_false_alarm_rate` and `first_story_miss_rate` fail closed on empty, mismatched, or single-class streams;
- computed RMSE of known first-story targets is lower under calibrated probabilities than under an always-first detector;
- a mixed stream with one false alarm and one miss recovers FAR `0.5` and miss `0.5` with residual RMSE below `1e-15`.
