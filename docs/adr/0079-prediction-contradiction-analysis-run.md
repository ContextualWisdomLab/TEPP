# ADR 0079 — Prediction-contradiction refusals as an analysis-run output profile

**Decision status:** Proposed

**Implementation maturity:** fold-child — source/tests/doctoring live on Draft #487 and are not implemented-main; canonical landing authority remains #416 pending repository-wide ADR normalization under #437.

**Date:** 2026-09-03

**Supersedes:** None. This is implementation lineage under ADR 0002 (six-clock eligibility and Allen algebra), ADR 0016 (TDT/CHRONOS prediction remains hypothetical until supported), and ADR 0022 (cutoff-safe analysis-run execution). It does not mint an independently accepted bounded context.

**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.

**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to promote unmatched predicted mass to observed fact through `prediction_contradiction::refuse_promotion`. Coverage requires Allen `during`, `starts`, `finishes`, or `equals`; partial overlap, `meets` / `met_by`, and Allen `before` / `after` remain hypothetical or contradictory as defined by the owning domain crate. `refuse_contradiction_or_adjacency` is not promotion authority.

The Analysis Run application boundary needs a digest-bound, cutoff-safe profile for those existing domain refusals without duplicating Allen semantics. `contradiction_agreement_rate` stays library-side and is not projected into operator inspect payloads. GPU kernels, MCMC, and topic birth/split/merge remain outside this slice.

An earlier branch version made a scientific admission error: it required a deliberately mixed four-class fixture to appear in production data. Covered, partial-overlap, adjacent, and contradictory classes are observations, not design strata. A valid historical census may contain only covered predictions, only contradictions, or any other nonempty combination. Zero counts for absent classes are evidence and must not become missing-data failures.

## Decision

Keep `prediction_contradiction_v1` / `tepp.prediction_contradiction.v1` as a Draft Validation / Analysis Run profile to be folded into #416. The profile:

- consumes bounded `PredictionContradictionAssignment` rows with predicted and observed closed event-time intervals plus availability time;
- requires request snapshot, output profile, and knowledge cutoff to match the execution context, comparing cutoffs through `KnowledgeCutoff::instant()` rather than RFC 3339 string identity;
- excludes rows whose `AvailableTime` is later than the knowledge cutoff **before** duplicate-identity admission, so future-unavailable evidence cannot perturb an earlier historical result;
- invokes `prediction_contradiction::refuse_promotion` rather than reimplementing Allen classification;
- admits any nonempty cutoff-eligible census up to `MAX_EVIDENCE_UNITS`; no relation class has a required minimum count;
- preserves exact count invariants: the four class counts sum to `assignment_count`, and `refused_promotion_count + covered_count == assignment_count`;
- emits canonical SHA-256-digested artifact identity and fixed inference status `unmatched_prediction_is_not_observed`;
- keeps `contradiction_agreement_rate` library-side and keeps inspect payloads metric-free;
- remains a fold child until #416 or a verified successor inherits the unique source, tests, fixtures, contract, and doctoring and reacquires exact-head gates.

## Alternatives considered

1. Treat all four Allen support classes as mandatory design strata — rejected. Relation classes are outcomes observed in the admitted census, and requiring every class would reject truthful sparse historical data.
2. Duplicate support-edge semantics — rejected because support-edge classifies evidence-versus-transition roles, not predicted-versus-observed promotion.
3. Bind `refuse_contradiction_or_adjacency` as promotion authority — rejected because partial overlap can still leave unmatched predicted mass.
4. Put `contradiction_agreement_rate` on the operator artifact — rejected because the inspect contract is metric-free and the agreement helper is not a generative known-truth recovery metric.
5. Create a new bounded-context authority for this profile — rejected. The profile composes existing Prediction Contradiction domain truth inside the existing Validation / Analysis Run application boundary.

## Consequences

Operators can eventually request cutoff-safe prediction-contradiction refusals as a digest-bound terminal result once the profile lands through #416. Sparse covered-only or contradiction-only censuses remain valid and expose zero counts for absent classes. Snapshot/profile/cutoff mismatch, no cutoff-eligible evidence, duplicate identities among evidence available at the cutoff, count-invariant violations, and oversized corpora fail closed.

This profile does not claim MCMC, GPU parity, support-edge, summarizes-edge, retrospective-edge, role-contradiction, subevent-containment, inferred-status, episode-membership, relation-absence, outcome-order, membership-target, location-membership, copy-identity, citation-edge, method-effect estimation, or topic birth/split/merge authority.

## Verification and traceability

Scientific RED `a2892b6ad4f632882f63da39eed2d4706ddf9213` adds covered-only and contradiction-only production-shaped censuses. Causal repair `a64020152300232cb3214a66b45d97225b6d2b5b` removes the artificial minimum/four-class gate while preserving nonempty evidence, bounded counts, exact count sums, digest validation, cutoff-before-identity admission, and the fixed claim boundary. Commit `6b0c8de64f41bc11f8bf908e0f9cbe854c1e213c` removes predecessor tests that encoded the rejected mixed-fixture eligibility rule.

The four-class fixture remains test coverage because it exercises every `PredictionContradictionError` mapping; it is not a scientific eligibility condition. The eventual #416 survivor must run the focused sparse-census, cutoff-semantics, and execution contracts plus full `analysis_engine` tests, Clippy, documentation validation, and all live required workflows on its own exact head.

## Rollback and supersession

Until protected-main integration, rollback is simply removal of the fold-child profile; no persisted schema migration exists. Supersession must preserve leakage-safe cutoff admission, truthful zero relation-class counts, Prediction Contradiction owner semantics, and the metric-free inspect boundary. ADR identity/status normalization remains owned by #437.
