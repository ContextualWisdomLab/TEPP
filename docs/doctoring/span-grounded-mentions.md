# Span-grounded event mentions (doctoring)

## Claim boundary

`SpanGroundedMention` binds a fallible event mention to one exact
`evidence_core::SourceSpan`, six typed clocks, an extractor/model version, and
a proposed-or-reviewed inspection status. Exact-extent precision and recall
are computed against known-truth spans. A reviewed mention is not a promoted
event instance. This slice does not run TDT tracking, CHRONOS schema
extraction, interval consistency, persistence, or GraphML/JSON-LD export, and
it does not close issue #170.

## Numeric constants

| Constant | Value | Provenance |
|---|---|---|
| extent identity | `(document_id, byte_start, byte_end)` | Doddington et al. (2004) map a system mention to a reference mention when their extents match. TEPP uses the already-validated `SourceSpan` byte bounds plus document identity. |
| availability eligibility | `available_time ≤ knowledge_cutoff` | ADR 0002 / `evaluate_historical_eligibility`: every available instant must be at or before cutoff. Instant equality is eligible. |
| precision | `\|recovered ∩ truth\| / \|recovered\|` | Standard set precision of mapped extents; no extra weight. |
| recall | `\|recovered ∩ truth\| / \|truth\|` | Standard set recall of mapped extents; no extra weight. |

There is no probability threshold, similarity weight, or overlap heuristic.
Partial or shifted extents do not count. Empty or duplicated extent sets fail
closed.

## Primary sources

Doddington, G., Mitchell, A., Przybocki, M., Ramshaw, L., Strassel, S., &
Weischedel, R. (2004). The Automatic Content Extraction (ACE) program—Tasks,
data, and evaluation. In *Proceedings of the Fourth International Conference
on Language Resources and Evaluation (LREC’04)* (pp. 837–840). European
Language Resources Association.

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based
information organization*. Kluwer Academic Publishers.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613
