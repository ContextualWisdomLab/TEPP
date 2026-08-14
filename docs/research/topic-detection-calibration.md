# Topic detection calibration

## Scope

This note doctors the `event_core` gate that keeps TDT topic detection distinct from event-instance promotion:

1. a topic cluster identifier is detection evidence, not a promoted instance;
2. new-topic versus existing-topic false-alarm and miss rates are computed from known-truth labels;
3. pair precision and recall recover co-topic assignments against known clusters;
4. calibrated new-topic probabilities recover the binary new-topic target with lower RMSE than an always-new detector.

No database migration is allocated. Topic detection is not first-story detection: a new topic may contain many events, and a known topic may still receive a first story. A later TDT tracker may consume these scores as measurement evidence only.

## Authoritative sources

Allan, J., Carbonell, J., Doddington, G., Yamron, J., & Yang, Y. (1998). Topic detection and tracking pilot study: Final report. In *Proceedings of the DARPA Broadcast News Transcription and Understanding Workshop* (pp. 194–218).

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Fiscus, J. G., & Doddington, G. R. (2002). Topic detection and tracking evaluation overview. In J. Allan (Ed.), *Topic detection and tracking: Event-based information organization* (pp. 17–31). Kluwer Academic Publishers.

## Application

Allan et al. (1998) and Fiscus and Doddington (2002) define topic detection as an unsupervised clustering task that may open a new topic or assign a story to an existing one. Official evaluation reports detection cost, false-alarm and miss rates, and cluster-mapping scores rather than event identity. Allan (2002) keeps those outputs in the measurement layer. TEPP therefore refuses to cast a topic cluster as an event instance and requires computed new-topic FAR, miss, pair precision/recall, and RMSE against known truth (Allan et al., 1998; Allan, 2002; Fiscus & Doddington, 2002).

## Verification

- `refuse_topic_cluster_as_event_instance` always returns `TopicClusterIsNotEventInstance`;
- `decide_topic_detection` uses an inclusive probability threshold;
- `new_topic_false_alarm_rate` and `new_topic_miss_rate` fail closed on empty, mismatched, or single-class streams;
- `topic_cluster_pair_precision` and `topic_cluster_pair_recall` fail closed when no pair exists;
- computed RMSE of known new-topic targets is lower under calibrated probabilities than under an always-new detector.
