# ADR 0033 — Longitudinal CWC composition as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31; reviewed 2026-09-19
**Supersedes:** None; complements ADR 0005 (ESEM/DSEM interpretation) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already recovers Enders and Tofighi (2007) cluster-mean-centered within/between OLS and the CWC contextual effect inside `psychometric_core`. Operators still cannot request that composition as a digest-bound analysis-run output. Recovery primitives alone are not the ESEM/DSEM engine (GAP-006 / #169), and branch-local implementation does not make this ADR protected-main authority.

Review found several application-boundary defects: equivalent RFC 3339 spellings of one cutoff instant were rejected; completed artifacts did not enforce `contextual_effect = between_slope - within_slope`; artifact evidence counts could exceed the executor population bound; the causal-refusal provider result was discarded; clustered rows lacked immutable source snapshot and evidence identities; a public `excluded_after_cutoff_count` caused later-only corpus existence to alter an earlier digest-bound result; snapshot provenance was checked before availability admission, so a future-only row from another snapshot could turn an earlier successful replay into `SnapshotMismatch`; an all-singleton count shape could pass artifact validation even though the CWC executor cannot produce a successful within slope for it; and the completed artifact dropped the admitted evidence identity/numeric provenance after validation, so two different cutoff-visible populations could produce the same artifact digest whenever their counts and slopes coincided.

## Decision

Add the `longitudinal_cwc_v1` analysis-run output profile to `analysis_engine`, while keeping the reusable numerical estimator in `psychometric_core`.

The executor:

- requires every clustered score to carry bounded opaque `evidence_id`, immutable `snapshot_id`, and `AvailableTime` provenance;
- preserves the raw `MAX_EVIDENCE_UNITS` operational admission ceiling;
- excludes rows with `AvailableTime > knowledge_cutoff` before snapshot, evidence-identity, or scientific-domain admission;
- rejects a cross-snapshot row as a provenance violation when that row is cutoff-visible;
- rejects duplicate `evidence_id` among cutoff-visible evidence so one observation cannot acquire extra scientific weight;
- does not infer identity from cluster/predictor/outcome/time tuples, so numerically equal observations with distinct identities remain distinct evidence;
- keeps future-unavailable evidence out of public historical counts, admitted-evidence commitment, artifact digest, and terminal result; it does not emit an excluded-future counter;
- commits the exact cutoff-visible evidence population to `admitted_evidence_sha256` before scientific composition using a versioned, domain-separated, length-delimited binary encoding of `evidence_id`, `snapshot_id`, `cluster_key`, exact predictor/outcome binary64 bits, and canonical availability time;
- canonicalizes only that provenance commitment by immutable `evidence_id`, so source enumeration does not alter the commitment; estimator inputs are not sorted or numerically rewritten as a workaround for #596;
- rejects malformed or non-lowercase SHA-256 values when a standalone artifact is imported;
- binds request and executor cutoffs by parsed `KnowledgeCutoff::instant()` equality rather than RFC 3339 text;
- invokes `recover_cluster_mean_within_between_slopes` without reimplementing CWC arithmetic;
- requires the exact `claim_causal_effect(CausalHeuristic::TemporalPrecedence)` refusal and fails closed if that provider contract drifts;
- requires at least one non-singleton cluster in a successful artifact (`cluster_count < row_count`) while not claiming that this count rule alone proves nonsingularity;
- validates the contextual-effect identity exactly against the recovered within/between slopes;
- emits terminal provider status `validated` separately from artifact inference status `composed_cwc_slopes_not_causal`;
- emits canonical SHA-256-bound `tepp.longitudinal_cwc.v1` output and does not persist raw rows.

The admitted-evidence digest is a content/provenance commitment, not source-generator attestation, construct-validity evidence, or scientific acceptance. This is two-level OLS composition, not DSEM, RI-CLPM, a random-effects sampler, or causal identification.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those recoveries do not bind composition to an analysis run.
2. Put CWC execution into `tepp_api` — rejected because transport contracts and scientific composition would become one service boundary.
3. Trust a run-level snapshot without per-row snapshot/evidence provenance — rejected because replay and cross-snapshot misattribution would be indistinguishable from legitimate scientific weight.
4. Validate snapshot provenance before availability — rejected because a row that did not exist at the historical cutoff would then be able to alter that replay solely through future corpus state (#595).
5. Deduplicate by predictor/outcome/cluster/time tuple — rejected because equal observed values do not imply the same evidence unit.
6. Count rows excluded after the historical cutoff in the public artifact — rejected because later corpus existence would change an earlier replay and its digest.
7. Treat equivalent RFC 3339 text as different cutoffs — rejected because textual representation is not temporal identity.
8. Reimplement CWC arithmetic in the adapter — rejected; `psychometric_core` remains the canonical numerical owner.
9. Treat counts and recovered slopes as sufficient artifact provenance — rejected because different admitted evidence identities or coordinates can yield the same summary and therefore the same result digest (#600).
10. Hash rows in caller/source order — rejected because enumeration order is not evidence identity and would introduce a second order-dependence while #596 is already tracking the separate numerical-order defect.
11. Sort estimator inputs to make #596 pass — rejected because that would hide rather than repair reusable finite-binary64 arithmetic ownership; only the provenance digest is canonicalized by identity.

## Scientific acceptance boundary

The identity, leakage, structural-wire, and provenance-commitment repairs establish input/output integrity, not commercial scientific acceptance. Issue #501 owns repeated true-parameter recovery for within, between and contextual slopes; RMSE/bias with Monte Carlo uncertainty; attempted/recovered/failed denominators; cluster-size and signal/noise variation; unequal follow-up/time-varying availability; and leakage-safe rolling-origin evaluation. Issue #596 separately owns the unresolved row-permutation numerical invariant and may turn GREEN only after TEPP consumes an immutable released reusable finite-binary64 mean contract from fast-mlsirm.

The profile must not be described as scientifically accepted or release-ready while #501 remains open without equivalent checked-in evidence or while #596 remains unresolved.

## Consequences

A historical CWC run is invariant to evidence that was unavailable at its cutoff, including a future-only row carrying another snapshot identity. Snapshot provenance still fails closed for rows in the evidence population that was actually observable then. Duplicate identities in that cutoff-visible population fail closed before CWC arithmetic, while distinct evidence with equal values remains admissible. The artifact exposes only cutoff-visible scientific counts and slopes plus a source-text-free SHA-256 commitment to the exact admitted population; future-only census information is not part of the digest-bound historical result. Changing a cutoff-visible evidence identity or numeric/provenance field changes that commitment even when the recovered summary statistics happen to remain equal.

Shared ADR index, TRACEABILITY and product-gap currentization belong to the canonical documentation/consolidation lane. This branch-local ADR remains `Proposed` until the implementation is inherited by protected-main authority and its merge/release gates are satisfied.

## Verification

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

Regression contracts cover equivalent cutoff instants, cutoff-before-snapshot admission, visible cross-snapshot refusal, snapshot/evidence provenance, cutoff-visible duplicate refusal, future-unavailable replay invariance, equal-value distinct identities, admitted-evidence digest sensitivity, benign permutation stability of the commitment, contextual-effect tampering, impossible visible counts and all-singleton success shapes, provider/domain status separation, and causal-refusal fail-closed behavior. The pathological finite-binary64 permutation contract remains intentionally RED under #596 until the released numerical owner is available. Scientific acceptance remains #501.

## Rollback and supersession

Rollback removes the `longitudinal_cwc_v1` profile. No persisted schema migration is introduced. Supersede only with an ADR that keeps CWC distinct from between-cluster effects and causal identification, preserves immutable evidence/snapshot/availability provenance, retains cutoff-visible historical replay invariance, and keeps the digest-bound result tied to the exact admitted evidence population without making source enumeration part of the estimand.
