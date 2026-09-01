# PR Queue Delivery Authority — 2026-09-01

This is a live-recovery record for delivery issue #175. GitHub state supersedes this document whenever a head, review, check, or ruleset changes.

## Snapshot

- Protected default branch: `main`
- Protected-main SHA observed: `1bc02f580cf48e1d39da239f0e818453437c31c3`
- Open pull requests: **142**
- Draft pull requests: **100**
- Non-draft pull requests: **42**
- Open issues: **13**
- Effective required-workflow ruleset: `18156473`, `CWL Central required workflows`
- `docs/product-technical-gap-baseline.md` was replaced on this recovery branch so it no longer presents the former zero-queue snapshot as current authority.

The one-PR increase and one-draft increase from the initial 141/99 observation is this recovery PR itself. These counts are delivery evidence, not a completion claim. A queued, skipped, cancelled, predecessor-head, or stale check is not passing evidence.

## Queue authority classes

Every open PR must be classified before merge or closure as one of:

- `landing_vehicle`: the single current-main vehicle for a buyer-visible bounded-context outcome;
- `stacked_dependency`: unique work that must follow a named landing vehicle;
- `fold_into_landing_vehicle`: unique implementation/test/research evidence that belongs inside another bounded-context vehicle rather than remaining an independent product slice;
- `superseded`: no unique behavior remains after an exact-head replacement comparison;
- `duplicate`: same outcome and implementation authority as another PR, with unique evidence explicitly preserved or shown absent;
- `research_lineage_only`: useful scientific provenance that is not intended to ship as a separate runtime boundary;
- `blocked_external`: otherwise-valid work whose current blocker is external to TEPP and is recorded with an owning issue/PR.

A PR title, ADR number, local green test, or separate crate does not establish an independent product boundary.

## Landing order

The active queue must be reduced in this order:

1. **Delivery authority and DDD context ownership** — issue #175, this document, `docs/product-technical-gap-baseline.md`, and `docs/architecture/domain-context-map.md`.
2. **Evidence & Semantic Measurement vertical** — span-grounded semantic/concept input, method/source distinctions, immutable source offsets, concept dictionary versioning.
3. **Topic Measurement vertical** — real Rust CPU `f64` shared-latent estimator, uncertainty, true-parameter recovery, candidate-K fitting, topic lineage.
4. **Analysis Run / scientific evidence vertical** — durable lifecycle, estimator-bound validation evidence, persistence/recovery, terminal results, and operator retrieval. Scientific evidence production is separated from claim-promotion authority.
5. **Longitudinal Psychometrics vertical** — coherent ESEM/DSEM/multilevel measurement boundary rather than scalar-equation PR proliferation.
6. **Event Intelligence vertical** — TDT/CHRONOS composition and event-time evidence under Temporal Event Knowledge.
7. **Commercial runtime vertical** — tenancy, durable jobs, observability, backup/restore, release provenance, support.

New unrelated micro-PRs are release-excluded while this queue exceeds the active-queue target in issue #175.

## First classification findings

### PR #356 — validation-run scientific evidence

Classification: `landing_vehicle_candidate`, pending correction before it may become `landing_vehicle`.

Reason: it is direct from current protected main and binds cutoff-safe evidence to a validation run, but current review found a scientific-authority defect. The current acceptance rule compares RMSE with a standard error derived from the same residual vector using a caller-selected preregistered multiplier. This can accept arbitrarily large recovery error for pathological residual shapes. The current observation also accepts caller-supplied truth/recovered vectors and an `authored_by_llm` boolean rather than an estimator-owned, digest-bound provenance artifact. Graph recovery, invariance, convergence, and active-backend CPU/GPU parity applicability are not complete in the claimed scientific-acceptance artifact.

DDD correction: the run should produce **Validation Evidence**. A distinct **Claim Promotion Decision** aggregate, governed by ADR 0014 and a method-specific preregistered evidence contract, decides whether a scientific claim is promotable. No generic standard-error multiplier, maximum-k, or other rule-of-thumb threshold may substitute for a research- or model-derived acceptance design.

Exact-head hosted evidence is also not green: the current Product workflow failed its coverage-diagnostic jobs. The branch remains non-draft in GitHub metadata because the connector's draft-conversion mutation is currently broken; that metadata must not be interpreted as merge readiness.

Until the scientific and exact-head failures are corrected, #356 and downstream wire/HTTP slices must not be treated as a shippable scientific-acceptance vertical.

### PRs #352 and #355 — same Driver/ctsem TIPREDEFFECT rewrite

Classification: both `fold_into_landing_vehicle` candidates under Longitudinal Psychometrics; neither is selected as authority yet.

Both implement the scalar `-a * B` rewrite from the same current-main base. They differ in public naming, refusal guards, tests, doctoring, and documentation edits. Closing either solely because the title and core equation match would discard unique evidence. The consolidation vehicle must retain the stronger named-quantity refusal coverage and realistic published-example tests while avoiding unrelated architecture mega-row edits.

### Analysis-run transport slices

PRs for one GET/POST/CLI/status/cancel/retry/export/project-history operation are not separate bounded contexts. Classify them under the Analysis Run application context and `tepp_api` adapter. Parent/child stacks remain documented until a current-main landing vehicle preserves their unique contract tests and consumer compatibility.

### Evidence/method refusal slices

PRs binding prompt/style/modality/copied-text/corpus-background/template-copy/location/membership/citation refusals to separate analysis profiles are not automatically independent products. Classify the invariant under Evidence & Semantic Measurement or Temporal Event Knowledge, then fold compatible profiles into one coherent admission/method-effect landing vehicle where the runtime contract does not require an independently versioned lifecycle.

## Merge evidence required per landing vehicle

Before merge:

- re-read exact head/base and effective ruleset;
- compare current remote diff against all PRs marked fold/supersede/duplicate;
- preserve unique tests, public compatibility contracts, citations, and doctoring;
- resolve valid non-outdated review threads;
- obtain the approvals required by the effective ruleset and any stronger non-stale PR-specific bar;
- require all exact-head required workflows to succeed;
- update `docs/product-technical-gap-baseline.md`, architecture/ADR/traceability/changelog when protected-main truth changes;
- do not use force push or protection bypass to simplify consolidation.

## DDD delivery constraint

Directory and crate moves are part of the owning product-vertical replay. Do not preserve a technical-layer or one-rule path as canonical if it obscures the domain responsibility. Conversely, do not rename all 58 crates in one sweeping PR while 142 remote heads are active. The target bounded contexts are fixed in `docs/architecture/domain-context-map.md`; migration proceeds through safe, reviewable landing vehicles with explicit anti-corruption adapters and replacement mappings.
