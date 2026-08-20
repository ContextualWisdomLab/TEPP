# Simulation cutoff eligibility

## Scope

This note doctors the `tepp_simulation` historical-fit filter:

1. a delayed-reporting document is eligible only when `available_time <= knowledge_cutoff`;
2. eligible counts must match the known-truth count computed from generated clocks;
3. a late document fails closed.

No database migration is allocated. Interval-aware cutoff on uncertain availability remains on the `temporal_core` active PR.

## Authoritative sources

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

Dyreson, C. E., & Snodgrass, R. T. (1998). Supporting valid-time indeterminacy. *ACM Transactions on Database Systems, 23*(1), 1–57. https://doi.org/10.1145/288086.288087

## Application

Jensen and Snodgrass (1999) distinguish valid time from transaction time. TEPP defines `available_time` as an application-specific knowledge-availability clock for historical fits. This mapping is specified by ADR 0002 and the `tepp_simulation` eligibility and fail-closed validation APIs. A document written at `document_time` but released later cannot enter a cutoff that precedes `available_time` (Jensen & Snodgrass, 1999; Dyreson & Snodgrass, 1998).

## Verification

- a delayed corpus has fewer eligible documents than total documents at an early cutoff;
- eligible count equals the known-truth count from generated clocks;
- `refuse_unavailable_document` admits early documents and denies late ones.
