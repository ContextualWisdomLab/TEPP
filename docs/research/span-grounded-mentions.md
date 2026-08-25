# Span-grounded event mentions

## Scope

This note doctors the `event_core` contract that keeps documentary event mentions
span-grounded and distinct from promoted instances:

1. a mention cites one exact source extent, document identity, six-clock
   evidence, extractor/model version, and review status;
2. exact-extent precision and recall are computed from known-truth spans;
3. a reviewed mention remains observed evidence and cannot promote an instance.

No database migration is allocated. Interval consistency, persistence, and
versioned GraphML/JSON-LD exports remain later #170 work.

## Authoritative sources

Doddington, G., Mitchell, A., Przybocki, M., Ramshaw, L., Strassel, S., & Weischedel, R. (2004). The Automatic Content Extraction (ACE) program—Tasks, data, and evaluation. In *Proceedings of the Fourth International Conference on Language Resources and Evaluation (LREC’04)* (pp. 837–840). European Language Resources Association.

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

## Application

Doddington et al. (2004) evaluate ACE mentions by mapping system mentions to
reference mentions when their extents match. Allan (2002) keeps TDT detection
outputs in the measurement layer. Jensen and Snodgrass (1999) separate valid
time from transaction/availability time, so event time cannot replace the
cutoff test. TEPP therefore requires exact `(document, byte_start, byte_end)`
identity, refuses availability after cutoff, and refuses to cast a
span-grounded mention as an event instance (Doddington et al., 2004; Allan,
2002; Jensen & Snodgrass, 1999). Numeric provenance is recorded in
[`docs/doctoring/span-grounded-mentions.md`](../doctoring/span-grounded-mentions.md).

## Verification

- `refuse_span_mention_as_instance` always returns `SpanMentionIsNotEventInstance`;
- `SpanGroundedMention::new` derives the surface from the exact document span;
- availability after cutoff returns `MentionIneligibleAtCutoff`;
- delayed reporting at or before cutoff is kept;
- `mention_span_precision` and `mention_span_recall` fail closed on empty or duplicate extents;
- an exact extractor recovers precision 1 and recall 1, with occupancy RMSE 0, against a two-mention documentary fixture, while a whole-document extractor recovers precision 0, recall 0, and RMSE 1.
