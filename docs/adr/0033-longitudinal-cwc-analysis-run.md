# ADR 0033 — Longitudinal CWC composition as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0005 (ESEM/DSEM interpretation) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already recovers Enders and Tofighi (2007) cluster-mean-centered within/between OLS and the CWC contextual effect inside `psychometric_core`. Operators still cannot request that composition as a digest-bound analysis-run output. Recovery primitives alone are not the ESEM/DSEM engine (GAP-006 / #169), and branch-local implementation does not make this ADR protected-main authority.

The first profile implementation also exposed four contract defects during review: equivalent RFC 3339 spellings of one cutoff instant were rejected, completed artifacts did not enforce `contextual_effect = between_slope - within_slope`, artifact evidence counts could exceed the executor population bound, and the causal-refusal provider result was discarded. A fresh scientific review additionally found that rows carried availability but no immutable source snapshot identity, so a caller could attribute another snapshot's coordinates to the requested snapshot.

## Decision

Add the `longitudinal_cwc_v1` analysis-run output profile to `analysis_engine`, while keeping the reusable numerical estimator in `psychometric_core`.

The executor:

- requires every clustered score to carry immutable `snapshot_id` and `AvailableTime` provenance;
- rejects cross-snapshot evidence before scientific composition and excludes same-snapshot rows whose availability is later than the requested knowledge cutoff;
- binds request and executor cutoffs by parsed `KnowledgeCutoff::instant()` equality rather than RFC 3339 text;
- preserves the raw `MAX_EVIDENCE_UNITS` admission ceiling and validates completed row/exclusion counts against the same executable population bound;
- invokes `recover_cluster_mean_within_between_slopes` without reimplementing CWC arithmetic;
- requires the exact `claim_causal_effect(CausalHeuristic::TemporalPrecedence)` refusal and fails closed if that provider contract drifts;
- validates the contextual-effect identity exactly against the recovered within/between slopes;
- emits terminal provider status `validated` separately from artifact inference status `composed_cwc_slopes_not_causal`;
- emits canonical SHA-256-bound `tepp.longitudinal_cwc.v1` output and does not persist raw rows.

This is two-level OLS composition, not DSEM, RI-CLPM, a random-effects sampler, or causal identification.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those recoveries do not bind composition to an analysis run.
2. Put CWC execution into `tepp_api` — rejected because transport contracts and scientific composition would become one service boundary.
3. Trust the run-level snapshot label without per-row provenance — rejected because it cannot prove that supplied coordinates belong to the requested immutable snapshot.
4. Treat equivalent RFC 3339 text as different cutoffs — rejected because textual representation is not temporal identity.
5. Reimplement CWC arithmetic in the adapter — rejected; `psychometric_core` remains the canonical numerical owner.

## Scientific acceptance boundary

A noiseless fixture and existing owner-level known-truth tests are regression evidence, not commercial scientific acceptance for this profile. Issue #501 owns the remaining profile-level recovery evidence: repeated true-parameter recovery for within, between and contextual slopes; RMSE/bias with Monte Carlo uncertainty; attempted/recovered/failed denominators; cluster-size and signal/noise variation; and leakage-safe temporal evaluation where availability changes over time.

The profile must not be described as scientifically accepted or release-ready while #501 remains open without equivalent checked-in evidence.

## Consequences

Operators can eventually request a historical, snapshot-bound within/between/contextual composition without allowing future evidence, another snapshot, or a provider-contract drift to silently change the scientific result. The artifact remains associational and explicitly separates provider validation from the scientific claim boundary.

Shared ADR index, TRACEABILITY and product-gap currentization belong to the canonical documentation/consolidation lane. This branch-local ADR remains `Proposed` until the implementation is inherited by protected-main authority and its merge/release gates are satisfied.

## Verification

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

Regression contracts cover equivalent cutoff instants, snapshot provenance, cross-snapshot refusal, impossible artifact counts, contextual-effect tampering, provider/domain status separation and causal-refusal fail-closed behavior. Scientific acceptance remains #501.

## Rollback and supersession

Rollback removes the `longitudinal_cwc_v1` profile. No persisted schema migration is introduced. Supersede only with an ADR that keeps CWC distinct from between-cluster effects and causal identification, preserves immutable snapshot/availability provenance, and retains leakage-safe historical replay semantics.
