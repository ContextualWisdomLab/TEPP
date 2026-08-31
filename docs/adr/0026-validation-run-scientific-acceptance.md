# ADR 0026 — Durable validation-run scientific acceptance evidence

**Decision status:** Accepted
**Implementation maturity:** active-PR — library-level `analysis_engine` binding on this PR; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0002, ADR 0008, ADR 0013, ADR 0014, and ADR 0022.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

GAP-003A records that immutable evidence cannot yet be submitted to a durable
validation run that produces operator-usable scientific acceptance evidence.
`validation_core` already computes RMSE, bias, interval coverage, temporal-order
accuracy, and SE-aware gates, but those metrics remain library-level. An
accepted analysis run is a transport receipt; it must never carry scientific
results. Operators therefore have no hash-stable identity that binds cutoff-safe
evidence, model, seed, backend, and precision to one completion that emits
scientific acceptance evidence.

Postgres persistence, restart/recovery, and Compose E2E remain GAP-003B.

## Decision

Add a bounded `analysis_engine` validation-run executor:

- `submit_validation_run` binds sorted cutoff-eligible evidence identities,
  tenant workspace, snapshot, knowledge cutoff, model `validation_cpu_f64_v1`,
  seed, backend `cpu`, precision `f64`, and output profile
  `scientific_acceptance_v1` into a canonical SHA-256 digest. The durable
  `run_id` is `tepp-validation-{32 hex}`. The receipt carries no RMSE, bias,
  coverage, or gate fields.
- `complete_validation_run` rebinds the same scientific identity, requires
  recovery vectors stamped to that `run_id` and binding digest, refuses
  LLM-authored recovery, computes `validation_core` recovery metrics, applies
  the SE-aware gate `|RMSE − 0| ≤ k · SE(RMSE)`, and emits
  `tepp.scientific_acceptance.v1` under output profile
  `scientific_acceptance_v1`. The artifact records a SHA-256 of the stamped
  recovery vectors. Evidence fields are private after completion.
- Empty corpora, duplicate evidence identities, snapshot mismatch, invalid
  profiles, non-finite inputs, oversized recovery vectors, a different run /
  tenant / seed / eligible evidence set, a tampered output profile, and
  cutoff-empty eligibility fail closed.
- A computed recovery that fails the SE-aware gate still emits evidence with
  `se_gate_accepted = false` so operators can read the metrics. Invalid or
  LLM-authored recovery never emits evidence.
- The engine does not persist rows, claim implemented-main, or replace ADR 0014
  exact-head claim promotion.

## Alternatives considered

1. Keep metrics library-only — rejected because GAP-003A is the operator-visible
   product-completion gap and has no live implementation PR.
2. Return scientific metrics on `AnalysisRunAccepted` — rejected because
   accepted/running receipts must never carry scientific results.
3. Persist the run in PostgreSQL in this slice — rejected because durable
   storage, restart, and Compose recovery belong to GAP-003B / issue #287.
4. Bind evidence, cutoff, model, seed, backend, and precision in
   `analysis_engine` and emit scientific acceptance evidence through
   `validation_core` — accepted because it is independently testable and does
   not weaken fail-closed gates.

## Consequences

Operators can submit one immutable evidence snapshot and later complete it with
known-truth recovery to obtain a digest-bound scientific acceptance artifact.
The run identity is hash-stable for the scientific binding, independent of
evidence input order and of ineligible post-cutoff units. Persistence, HTTP
ingress, and release promotion remain later slices. LLM output still cannot
become scientific authority.

## Verification

The stacked PR includes unit and integration tests for hash-stable identity,
cutoff exclusion, metric-free receipts, SE-aware accept and refuse, recovery
stamped to a foreign run or tenant, a tampered output profile, oversized
vectors, and fail-closed LLM, NaN, empty, duplicate, snapshot, profile, and
cutoff-empty paths. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

Doctoring and APA 7th citations are in
`docs/research/validation-run-scientific-acceptance.md`.

## Rollback and supersession

Rollback removes `validation_run` from `analysis_engine` and stops exporting
the scientific-acceptance artifact. No persisted schema migration is
introduced. Supersession requires a new ADR if execution changes cutoff
semantics, binding identity, LLM-refusal, or scientific estimands.
