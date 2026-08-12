# TEPP Test and Scientific Validation Strategy

**Status:** Accepted quality baseline aligned to PRD v0.4  
**Last reviewed:** 2026-08-09

## Mandatory repository gates

- `cargo fmt --check`;
- warning-free stable Rust build/Clippy/tests/rustdoc;
- `cargo-nextest` without hidden retries plus doctests;
- production line coverage exactly 100%;
- production branch coverage exactly 100% in the pinned LLVM/nightly lane;
- public rustdoc/documentation-quality gates;
- dependency/license/advisory/source policy;
- current-head SAST/security/review;
- documentation-contract validation.

Queued, cancelled, skipped-required, absent, stale, predecessor-head, or synthetic-only evidence is not passing.

## Evidence-domain tests

Verify immutable source bytes/text, canonical SHA-256, UUIDv7 identifiers, size bounds, strict versioned JSON, unknown-field rejection, content-redacting errors, exact byte/scalar spans, UTF-8 boundary handling, cross-document rejection, and optional geometry validity.

## Temporal tests

PR #5 must prove six nominal clock types cannot be accidentally interchanged, strict RFC 3339/UTC normalization, precision retention, interval boundary semantics, unknown/open intervals, reversed/empty rejection, strict wire schemas, and non-reflecting errors.

PR #6 must prove all 13 Allen relations, inverse/composition laws, independent composition verification, proper-interval classification, bounded path-consistency, contradiction evidence, provenance, resource limits, and atomic rollback. It must not overclaim global satisfiability.

## Leakage tests

Construct retrospective and revised documents with event time earlier than availability time and assert rolling-origin/historical snapshots exclude future-available evidence. Related revision/translation/copied-template/event-episode variants must stay on one side of train/validation/test when leakage would inflate results.

## Event/graph recovery

Synthetic truth should define event instances, mentions, typed relations, roles, partial orders, and membership. Evaluate mention/relation precision/recall, relation sign/type recovery, contradiction detection, temporal ordering, and graph structure rather than only parser accuracy.

## Multilingual measurement validation

Use parallel/equivalent content and human-reviewed span/concept evidence to evaluate semantic-unit span F1, concept precision/recall, calibration/Brier score, language-specific error, and shared latent alignment. Topic/factor comparisons across language/time/template require appropriate invariance evidence.

## Topic true-parameter recovery

Generate corpora with known topic prevalence/content parameters, covariance, covariate effects, document coordinates, temporal drift, and method/background factors. Match recovered topics to truth before computing RMSE/bias/coverage. Evaluate seed/bootstrap stability and known-K recovery/acceptable-set behavior.

## Psychometric validation

For ESEM/DSEM simulations evaluate loading/factor/path recovery, bias, RMSE, confidence/credible interval coverage, convergence, configural/metric/scalar or partial invariance as required, multilevel/multiple-membership effects, within/between decomposition, irregular-time dynamics, and posterior plausible-value propagation.

## Network/cluster validation

Use known covariance/community structures. Evaluate CLR/log-ratio correlation recovery, edge sign/precision/recall/interval coverage/selection stability and cluster ARI/NMI/bootstrap stability. Raw compositional topic proportions are not validated through naïve Pearson correlation.

## CPU/GPU parity

CPU `f64` is reference. Required accelerator lanes execute real kernels; skipped GPU tests are failures. Compare objective values, parameters/posteriors, convergence, validation metrics, and deterministic artifacts under stated tolerances. Record peak VRAM, transfer, kernel time, precision mode, batch adaptation, OOM recovery, and fallback.

## LLM tests

Deterministic schema/security tests are primary. Bounded live tests use `NVIDIA_NIM_API_KEY` only when model conformance is material. Treat documents as prompt-injection data, require evidence-span grounding, test unsupported-claim rejection, record provider/model/prompt/reasoning hashes, and compare model/human agreement where the LLM acts as a rater.

## Monte Carlo acceptance

Simulation thresholds account for Monte Carlo standard error and interval uncertainty. Do not require an observed replication proportion to exceed the nominal target exactly when sampling variability makes that scientifically invalid.

## Release acceptance

A release requires one integrated protected head with all relevant scientific, numerical, security, migration, packaging, SBOM/provenance, accessibility, operational, and independent-review evidence passing. Planning validation and local-only results are supporting evidence, not release proof.