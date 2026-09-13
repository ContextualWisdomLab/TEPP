# ADR 0057 — Simulation method-effect labels as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0004/0012 (method-effect truth factors) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0049–0056.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already labels generated documents with
`DocumentMethodEffect` (`original`, `revision`, `translation`,
`template_copy`) through `tepp_simulation::generate` and admits them with
`refuse_unavailable_document`. Operators still cannot request that
cutoff-safe census as a digest-bound analysis-run output. Estimator-side
method models, GPU kernels, MCMC, and topic birth/split/merge remain later
GAP-004 work and are not this slice.

This decision remains Proposed while the implementation is an unmerged Draft.
Branch-local implementation evidence does not make an architectural decision
Accepted or protected-main authority.

## Decision

Add the `method_effects_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes an already-validated `SimulationConfig`;
- requires the request snapshot to match the offered construction and compares
  request/executor knowledge cutoffs by parsed temporal instant rather than RFC
  3339 text spelling;
- generates the CPU truth corpus and admits documents through
  `refuse_unavailable_document` without reimplementing method-effect labels;
- rejects configurations whose conservative event/document/membership/relation
  row bound exceeds 1,000,000 before generator allocation;
- emits a canonical SHA-256-digested `tepp.method_effects.v1` artifact with
  seed, config digest, a digest over exactly the cutoff-admitted documents,
  original/revision/translation/template-copy counts, and inference status
  `simulation_method_effect_labels_not_estimator_model`;
- keeps provider-authored terminal validation status (`validated`) separate
  from the domain inference-status claim;
- does not invent an estimator-side method model, select GPU backends, draw
  MCMC, or emit topic birth/split/merge events.

This is a simulation method-effect census, not an estimator-side method
model.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those
   recoveries are already a live micro-PR family and do not bind method-effect
   labels to an analysis run.
2. Duplicate exhaustive case-deletion (#413) — rejected because that profile
   refits deleted corpora and does not census method-effect labels.
3. Invent an estimator-side method model — rejected because that remains
   later GAP-004 work; this slice only binds the library already on main.
4. Compare RFC 3339 cutoff strings literally — rejected because legal textual
   representations of the same instant must not produce different scientific
   admission decisions.
5. Reuse the domain inference label as terminal validation state — rejected
   because validation status is provider-authored execution evidence, whereas
   `simulation_method_effect_labels_not_estimator_model` is a scientific claim
   boundary.
6. Bind existing `generate` + `refuse_unavailable_document` to ADR 0022's
   analysis-run profile — proposed here.

## Consequences

Operators can request cutoff-safe method-effect counts as a digest-bound
terminal result. Equivalent RFC 3339 spellings of the same cutoff instant are
semantically identical. The artifact does not claim an estimator-side method
model, GPU parity, MCMC, or topic birth/split/merge. Snapshot/profile/cutoff
mismatch, empty/undersized available corpora, and missing originals fail
closed. Malformed digests and over-budget generation also fail closed.

## Verification

The PR includes Rust unit and integration tests for digest-bound mixed
method-effect corpora, empty-available and singleton-original refusal,
snapshot / profile / cutoff mismatch, equivalent-instant cutoff binding,
provider-validation/domain-inference separation, admitted-population digest
binding, pre-allocation row limits, and artifact tampering. The temporal
contract was exposed first at `861d6a720f89068691e82932873f093c90c1511b`
and repaired at `6d7db6d89ac76f7cde69dd1b5c877ef8cbc362f6`.
Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `method_effects_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps simulation
method-effect labels distinct from an estimator-side method model, GPU
kernels, and topic birth/split/merge.
