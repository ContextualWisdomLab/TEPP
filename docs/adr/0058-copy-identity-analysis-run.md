# ADR 0058 — Template-copy identity refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0003 (copy-versus-source identity) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0057 (simulation method-effect census), ADR 0056 (case-deletion), ADR 0055 (composed fitted-K+lineage), ADR 0054 (export GET), ADR 0053 (Pareto candidate-`K`), or ADR 0052 (joint posterior Laplace draws).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat a template copy as the source
document identity or as a state transition via
`copy_identity::refuse_copy_as_source_identity` and
`refuse_copy_as_transition`. Operators still cannot request that refusal
census as a digest-bound analysis-run output. Simulation method-effect
labels (#415 / ADR 0057) count `DocumentMethodEffect::TemplateCopy` as a
generated-document census and do not replace `copy_identity`.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `copy_identity_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `CopyIdentityDocument` rows with closed
  `CopyKind` values;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- invokes `refuse_copy_as_source_identity` and `refuse_copy_as_transition`
  without reimplementing the copy/source vocabulary;
- emits a canonical SHA-256-digested `tepp.copy_identity.v1` artifact with
  source/template-copy counts, matching refusal counts, and inference
  status `template_copy_is_not_source_identity_not_transition`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate simulation method-effect labels (#415) — rejected because
   that profile counts generated `DocumentMethodEffect` variants and does
   not bind `copy_identity` refusals.
2. Put `identity_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
3. Bind the existing copy-identity refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe template-copy identity refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, method-effect estimation, or topic birth/split/merge.
Snapshot/profile/cutoff mismatch, empty or single-kind corpora, and
duplicate document identities fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed source/copy
corpora, empty/source-only/copy-only/duplicate refusal, snapshot /
profile / cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `copy_identity_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps
template-copy identity distinct from simulation method-effect
labels and from `identity_recovery_rate` inspect metrics.
