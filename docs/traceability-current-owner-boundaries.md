# Current numerical-owner and Validation Evidence traceability

**Status:** Draft traceability supplement
**Reviewed:** 2026-09-19
**Canonical documentation lane:** PR #435

This supplement records two exact-head obligations that are not yet represented directly in the legacy `docs/TRACEABILITY.md` matrix. It does not replace that matrix and does not convert mutable pull-request heads into release authority. The next safe matrix fold must preserve these rows verbatim in substance and then remove this supplement only after verified inheritance.

| Requirement / decision | Canonical basis | Exact source/evidence boundary | Maturity |
|---|---|---|---|
| Validation Evidence bias-SE resource/cardinality admission is distinct from numerical sample cardinality | PRD; Test Strategy; issue #491; PR #488 | `TEPP#488@520df488fd86ba48008af8ee5a2e112b4587fc22`; `analysis_engine::MAX_EVIDENCE_UNITS = 100_000` is an Analysis Run admission bound, not proof that one evidence unit maps to one `validation_core::bias_standard_error` truth/recovered pair. No typed/versioned buyer-path contract currently proves that mapping. Exact authored-production evidence remains 12,344/12,346 lines and 4,450/4,452 branch outcomes; the two unresolved obligations are `bias.rs:68-69` helper false-zero/resource reachability and `bias.rs:591-592` general translated-dispersion floating-positivity proof. | active-PR / unresolved scientific-product proof |
| Reusable correctly-rounded finite binary64 mean remains fast-mlsirm-owned and TEPP may consume only an immutable released contract | ADR 0001; TEPP #310/#495; fast-mlsirm #1717/#1816 | TEPP consumer RED: `TEPP#310@ba10820e0d28cc33d1b91ef37f6f6d163b3d91e9`, mixed-sign half-ULP actual bits `5080060379673919488` vs correctly-rounded expected `5080060379673919487`. Upstream prerequisite: `fast-mlsirm#1717@81d3ed2275a785cd3b136f267b208c0d2c166dd3`, open/Ready/mergeable on protected `main@a712995b1c22230bc7fcc7f693ae4ad88cb363f4`; exact-head CI/security/CodeQL/Semgrep/fuzz are still queued or pending and are not GREEN evidence. Numerical owner candidate: `fast-mlsirm#1816@3a1176998103332d26c9fb211fe2bf954dabd428`, open/Draft/mergeable; candidate contract `fast_mlsirm.binary64_mean@1.0.0` is mutable and non-consumable. Latest immutable owner release is `v0.11.4`, published `2026-09-18T08:14:12Z`, and predates both current candidate deltas. | blocked on immutable upstream release |

## Required owner sequence

The only accepted consumption sequence is:

`fast-mlsirm #1717 exact-head hosted acceptance + qualifying independent review` → normal protected-main merge → immutable fast-mlsirm release containing the landed prerequisite → `#1816` non-force restack and correctly-rounded binary64 numerical/oracle acceptance → normal merge → new immutable fast-mlsirm release with version/CHANGELOG/tag/package, SBOM/provenance, reproducibility, rollback, and numerical recovery evidence → TEPP #310 released-contract bump → TEPP mixed-sign RED turns GREEN without copying the upstream arithmetic or changing the estimand.

`v0.11.4` is release authority only for content actually present in that release. It is not evidence for #1717 post-restack repairs or #1816's binary64 mean candidate.

## Resource/cardinality decision record

The `100_000` Analysis Run evidence-unit ceiling must not be copied into `validation_core` or treated as a public numerical-domain cutoff. Before a finite product-side metric-population bound can discharge the remaining bias-SE floating proof, the supported buyer path must state and enforce how admitted evidence becomes truth/recovered metric pairs. Until then, the source-domain `n = 2^54` false-zero witness remains valid even though it is resource-extreme, and the two authored coverage obligations remain real rather than suppressible.

## Promotion boundary

This document is traceability evidence, not acceptance evidence. It authorizes no self-approval, gate weakening, mutable-owner consumption, foreign-owner source copy, coverage denominator reduction, skip/xfail, or predecessor-head receipt transfer. `docs/TRACEABILITY.md` remains the canonical matrix and must absorb these rows ordinary-forward in PR #435 before the documentation lane can claim fully code-current traceability.
