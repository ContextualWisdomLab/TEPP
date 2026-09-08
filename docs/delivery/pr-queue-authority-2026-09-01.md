# PR Queue Delivery Authority — 2026-09-01

This is a live-recovery record for delivery issue #175. GitHub state supersedes this document whenever a head, review, check, or ruleset changes.

## Snapshot

- Protected default branch: `main`
- Protected-main SHA observed: `1bc02f580cf48e1d39da239f0e818453437c31c3`
- Snapshot stamp: `2026-09-01T12:12:35Z`
- Open pull requests: **132**
- Draft pull requests: **89**
- Non-draft pull requests: **43**
- Open issues: **14**
- Effective required-workflow ruleset: `18156473`, `CWL Central required workflows`
- GitHub releases: **0**
- `docs/product-technical-gap-baseline.md` is maintained on this recovery branch so it no longer presents a former zero-queue snapshot as current authority.

The initial observation was 141/99. The queue later peaked at 149/100, fell through 140 and 139, and is now 132/89. This reduction is useful, but new one-operation Analysis Run slices can still recreate WIP while the recovery vehicle is open; each fresh run therefore compares the live count with the immediately preceding snapshot rather than treating any historical count as authority. A queued, skipped, cancelled, predecessor-head, or stale check is not passing evidence.

## Queue authority classes

Every open PR must be classified before merge or closure as one of:

- `landing_vehicle`: the single current-main vehicle for a buyer-visible bounded-context outcome;
- `stacked_dependency`: unique work that must follow a named landing vehicle;
- `fold_into_landing_vehicle`: unique implementation/test/research evidence that belongs inside another bounded-context vehicle rather than remaining an independent product slice;
- `superseded`: no unique behavior remains after an exact-head replacement comparison;
- `duplicate`: same outcome and implementation authority as another PR, with unique evidence explicitly preserved or shown absent;
- `research_lineage_only`: useful scientific provenance that is not intended to ship as a separate runtime boundary;
- `blocked_external`: otherwise-valid work whose current blocker is external to TEPP and is recorded with an owning issue/PR.

A PR title, ADR number, local green test, or separate crate does not establish an independent product boundary. There is no `landing_vehicle_candidate` class; a vehicle is either selected as `landing_vehicle` or remains one of the declared non-authority classes until selection is justified.

## Landing order

The active queue must be reduced in this order:

1. **Delivery authority and DDD context ownership** — issue #175, this document, `docs/product-technical-gap-baseline.md`, and `docs/architecture/domain-context-map.md`.
2. **Evidence & Semantic Measurement vertical** — span-grounded semantic/concept input, method/source distinctions, immutable source offsets, concept dictionary versioning.
3. **Topic Measurement vertical** — real Rust CPU `f64` shared-latent estimator, uncertainty, true-parameter recovery, candidate-K fitting, topic lineage.
4. **Analysis Run / scientific evidence vertical** — durable lifecycle, estimator-bound validation evidence, persistence/recovery, terminal results, and operator retrieval. Scientific evidence production is separated from claim-promotion authority.
5. **Longitudinal Modeling vertical** — coherent ESEM/DSEM/multilevel measurement and event-time composition rather than scalar-equation PR proliferation.
6. **Event Intelligence vertical** — TDT/CHRONOS composition and event-time evidence under Temporal Event Knowledge.
7. **Commercial runtime vertical** — tenancy, durable jobs, observability, backup/restore, release provenance, support.

New unrelated micro-PRs are release-excluded while this queue exceeds the active-queue target in issue #175.

## First classification findings

### PR #441 — event-time lagged association repair

Classification: `landing_vehicle` under Longitudinal Modeling. The scientific root cause is repaired on exact head `6f483224b3a03e8237c6f4f098a8b0e85e0a91f5`, but the vehicle is not merge-ready until exact-head workflows succeed and a qualifying independent non-author APPROVE exists.

The predecessor head exposed `(trait + e^{aΔt} p + added) / (trait + p + added)` as `expected_autocorrelation`. Review showed that a one-sided covariance/earlier-variance ratio is not a correlation under nonstationary marginals and can exceed one. The repair removes that public claim and its `psychometric_core` changes from the final diff. The replacement belongs to `longitudinal_core` and standardizes a supplied lagged covariance only when both marginal variances are available:

`Cov(Y_t,Y_t+Δ) / sqrt(Var(Y_t) * Var(Y_t+Δ))`.

The invalid predecessor commit remains only as RED/scientific-failure lineage. Review threads tied to the removed implementation are resolved. Canonical `ARCHITECTURE.md` and `CHANGELOG.md` record the event-time lagged-correlation standardizer; the one-shot self-modifying capability-record workflow was removed. Hosted exact-head Rust/documentation/security checks were still queued at this snapshot, so this PR is not yet a merge authority.

### PR #356 — validation-run scientific evidence

Classification: **closed without merge**. Useful cutoff/run-binding/metric evidence is preserved in branch and review history for fold into the Scientific Validation landing work tracked by #166. Do not reopen this exact vehicle.

The closed head `df33bfa3e61ae4de3dbfae16df0deac12d2f4003` bound cutoff-safe evidence to a validation run, but review found a scientific-authority defect: the acceptance rule compared RMSE with a standard error derived from the same residual vector using a caller-selected preregistered multiplier. That gate can accept arbitrarily large recovery error for pathological residual shapes. The closed observation also accepted caller-supplied truth/recovered vectors and an `authored_by_llm` boolean rather than an estimator-owned, digest-bound provenance artifact.

DDD correction for the next Scientific Validation vehicle: the run produces **Validation Evidence**. A distinct **Claim Promotion Decision** aggregate, governed by ADR 0014 and a method-specific preregistered evidence contract, decides whether a scientific claim is promotable. No generic standard-error multiplier, maximum-k, or other rule-of-thumb threshold may substitute for a research- or model-derived acceptance design.

### PRs #443–#446 — Analysis Run export stack

Classification: `fold_into_landing_vehicle` under one coherent Analysis Run/export vehicle.

Preserve each slice's unique pagination, authorization, cancellation, CLI parsing, origin/credential/consumer refusal, and metric-free receipt tests. Do not retain one route or one binary as a separate product boundary or architecture authority merely because it has its own ADR or branch.

### PR #447 — Analysis Run project-history cancel

Classification: `fold_into_landing_vehicle` under the Analysis Run/project-history vehicle. Preserve its empty-body, path/credential refusal, cancellation-removal, and metric-free receipt evidence before folding.

### PR #451 — temporal-context GET by id

Classification: `fold_into_landing_vehicle` under the existing Analysis Run/temporal-context adapter vehicle. Its path parsing, control/NUL/slash refusal, no-header replay rule, LineageWeave-only metric-free identity contract, and backward-compatibility tests are unique evidence to preserve. A single GET route and branch-local ADR number do not create a new bounded context or architecture authority.

### PRs #352 and #355 — same Driver/ctsem TIPREDEFFECT rewrite

Classification: both `fold_into_landing_vehicle` under Longitudinal Modeling; neither is selected as a separate authority.

Both implement the scalar `-a * B` rewrite from the same current-main base. They differ in public naming, refusal guards, tests, doctoring, and documentation edits. Closing either solely because the title and core equation match would discard unique evidence. The consolidation vehicle must retain the stronger named-quantity refusal coverage and realistic published-example tests while avoiding unrelated architecture mega-row edits.

### Analysis Run transport slices

PRs for one GET/POST/CLI/status/cancel/retry/export/project-history/temporal-context operation are not separate bounded contexts. Classify them under the Analysis Run application context and `tepp_api` adapter. Parent/child stacks remain documented until a current-main landing vehicle preserves their unique contract tests and consumer compatibility.

### Evidence/method refusal slices

PRs binding prompt/style/modality/copied-text/corpus-background/template-copy/location/membership/citation refusals to separate analysis profiles are not automatically independent products. Classify the invariant under Evidence & Semantic Measurement or Temporal Event Knowledge, then fold compatible profiles into one coherent admission/method-effect landing vehicle where the runtime contract does not require an independently versioned lifecycle.

## Merge evidence required per landing vehicle

Before merge:

- re-read exact head/base and effective ruleset;
- compare current remote diff against all PRs marked fold/supersede/duplicate;
- preserve unique tests, public compatibility contracts, citations, and doctoring;
- resolve valid non-outdated review threads;
- obtain the approvals required by the effective ruleset and any stronger non-stale PR-specific bar;
- require all exact-head documentation, Rust, security, and other required workflows to succeed;
- for scientific/model vehicles, require realistic synthetic-truth evidence for the parameters the candidate actually claims: parameter/state recovery, RMSE, bias, interval/credible-interval coverage, temporal ordering and leakage safety, graph recovery where a graph is claimed, longitudinal invariance where longitudinal comparability is claimed, and CPU/GPU parity where an accelerator path is claimed;
- report Monte Carlo uncertainty for recovery summaries rather than applying arbitrary replication pass percentages;
- treat skipped, ignored, xfailed, source-rewritten, predecessor-head, or non-executed GPU tests as non-evidence;
- update `docs/product-technical-gap-baseline.md`, architecture/ADR/traceability/changelog when protected-main truth changes;
- do not use force push or protection bypass to simplify consolidation.

A scientific vehicle that does not claim a GPU implementation does not manufacture a GPU-parity result; it records the accelerator path as unavailable/research-only. Conversely, a vehicle that claims an accelerator implementation cannot satisfy the gate by skipping that backend.

## DDD delivery constraint

Directory and crate moves are part of the owning product-vertical replay. Do not preserve a technical-layer or one-rule path as canonical if it obscures the domain responsibility. Conversely, do not rename all crates in one sweeping PR while 132 remote heads are active. The target bounded contexts are fixed in `docs/architecture/domain-context-map.md`; migration proceeds through safe, reviewable landing vehicles with explicit anti-corruption adapters and replacement mappings.
