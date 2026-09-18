# ADR 0033 — Longitudinal CWC composition as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31; reviewed 2026-09-19
**Supersedes:** None; complements ADR 0005 (ESEM/DSEM interpretation) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already recovers Enders and Tofighi (2007) cluster-mean-centered within/between OLS and the CWC contextual effect inside `psychometric_core`. Operators still cannot request that composition as a digest-bound analysis-run output. Recovery primitives alone are not the ESEM/DSEM engine (GAP-006 / #169), and branch-local implementation does not make this ADR protected-main authority.

Review found several application-boundary defects: equivalent RFC 3339 spellings of one cutoff instant were rejected; completed artifacts did not enforce `contextual_effect = between_slope - within_slope`; artifact evidence counts could exceed the executor population bound; the causal-refusal provider result was discarded; clustered rows lacked immutable source snapshot and evidence identities; a public `excluded_after_cutoff_count` caused later-only corpus existence to alter an earlier digest-bound result; and snapshot provenance was checked before availability admission, so a future-only row from another snapshot could turn an earlier successful replay into `SnapshotMismatch`.

## Decision

Add the `longitudinal_cwc_v1` analysis-run output profile to `analysis_engine`, while keeping the reusable numerical estimator in `psychometric_core`.

The executor:

- requires every clustered score to carry bounded opaque `evidence_id`, immutable `snapshot_id`, and `AvailableTime` provenance;
- preserves the raw `MAX_EVIDENCE_UNITS` operational admission ceiling;
- excludes rows with `AvailableTime > knowledge_cutoff` before snapshot, evidence-identity, or scientific-domain admission;
- rejects a cross-snapshot row as a provenance violation when that row is cutoff-visible;
- rejects duplicate `evidence_id` among cutoff-visible evidence so one observation cannot acquire extra scientific weight;
- does not infer identity from cluster/predictor/outcome/time tuples, so numerically equal observations with distinct identities remain distinct evidence;
- keeps future-unavailable evidence out of public historical counts, artifact digest, and terminal result; it does not emit an excluded-future counter;
- binds request and executor cutoffs by parsed `KnowledgeCutoff::instant()` equality rather than RFC 3339 text;
- invokes `recover_cluster_mean_within_between_slopes` without reimplementing CWC arithmetic;
- requires the exact `claim_causal_effect(CausalHeuristic::TemporalPrecedence)` refusal and fails closed if that provider contract drifts;
- validates the contextual-effect identity exactly against the recovered within/between slopes;
- emits terminal provider status `validated` separately from artifact inference status `composed_cwc_slopes_not_causal`;
- emits canonical SHA-256-bound `tepp.longitudinal_cwc.v1` output and does not persist raw rows.

This is two-level OLS composition, not DSEM, RI-CLPM, a random-effects sampler, or causal identification.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those recoveries do not bind composition to an analysis run.
2. Put CWC execution into `tepp_api` — rejected because transport contracts and scientific composition would become one service boundary.
3. Trust a run-level snapshot without per-row snapshot/evidence provenance — rejected because replay and cross-snapshot misattribution would be indistinguishable from legitimate scientific weight.
4. Validate snapshot provenance before availability — rejected because a row that did not exist at the historical cutoff would then be able to alter that replay solely through future corpus state (#595).
5. Deduplicate by predictor/outcome/cluster/time tuple — rejected because equal observed values do not imply the same evidence unit.
6. Count rows excluded after the historical cutoff in the public artifact — rejected because later corpus existence would change an earlier replay and its digest.
7. Treat equivalent RFC 3339 text as different cutoffs — rejected because textual representation is not temporal identity.
8. Reimplement CWC arithmetic in the adapter — rejected; `psychometric_core` remains the canonical numerical owner.

## Scientific acceptance boundary

The identity and leakage repairs establish input/output integrity, not commercial scientific acceptance. Issue #501 owns repeated true-parameter recovery for within, between and contextual slopes; RMSE/bias with Monte Carlo uncertainty; attempted/recovered/failed denominators; cluster-size and signal/noise variation; unequal follow-up/time-varying availability; and leakage-safe rolling-origin evaluation.

The profile must not be described as scientifically accepted or release-ready while #501 remains open without equivalent checked-in evidence.

## Consequences

A historical CWC run is invariant to evidence that was unavailable at its cutoff, including a future-only row carrying another snapshot identity. Snapshot provenance still fails closed for rows in the evidence population that was actually observable then. Duplicate identities in that cutoff-visible population fail closed before CWC arithmetic, while distinct evidence with equal values remains admissible. The artifact exposes only cutoff-visible scientific counts and slopes; future-only census information is not part of the digest-bound historical result.

Shared ADR index, TRACEABILITY and product-gap currentization belong to the canonical documentation/consolidation lane. This branch-local ADR remains `Proposed` until the implementation is inherited by protected-main authority and its merge/release gates are satisfied.

## Verification

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

Regression contracts cover equivalent cutoff instants, cutoff-before-snapshot admission, visible cross-snapshot refusal, snapshot/evidence provenance, cutoff-visible duplicate refusal, future-unavailable replay invariance, equal-value distinct identities, contextual-effect tampering, impossible visible counts, provider/domain status separation, and causal-refusal fail-closed behavior. Scientific acceptance remains #501.

## Rollback and supersession

Rollback removes the `longitudinal_cwc_v1` profile. No persisted schema migration is introduced. Supersede only with an ADR that keeps CWC distinct from between-cluster effects and causal identification, preserves immutable evidence/snapshot/availability provenance, and retains cutoff-visible historical replay invariance.
