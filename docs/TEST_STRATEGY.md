# TEPP Test and Scientific Validation Strategy

**Status:** Accepted quality baseline aligned to PRD v0.4  
**Last reviewed:** 2026-09-06

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

Queued, cancelled, skipped-required, absent, stale, predecessor-head, superseded-lineage, or synthetic-only evidence is not passing. A replacement branch may preserve source/test lineage for auditability, but it must reacquire every exact-head merge gate.

## Evidence-domain tests

Verify immutable source bytes/text, canonical SHA-256, UUIDv7 identifiers, size bounds, strict versioned JSON, unknown-field rejection, content-redacting errors, exact byte/scalar spans, UTF-8 boundary handling, cross-document rejection, and optional geometry validity.

## Temporal tests

Protected-main `temporal_core` (merged PR #8) must prove six nominal clock types cannot be accidentally interchanged, strict known-offset RFC 3339/UTC normalization, precision retention, interval boundary semantics, unknown/open intervals, reversed/empty rejection, strict wire schemas, schema/runtime parity, and non-reflecting errors. Superseded PR #5 is historical TDD lineage only and its old checks/reviews are not current evidence.

Task 4 on protected main (merged PR #9) must prove all 13 Allen relations (Allen, 1983), inverse/composition laws, independent composition verification, proper-interval classification, bounded path-consistency, contradiction evidence, provenance, resource limits, and atomic rollback. Superseded PR #6 is historical lineage on the discarded #5 stack, not a current-product claim. Path consistency must not be documented as unrestricted global satisfiability.

## Leakage tests

Construct retrospective and revised documents with event time earlier than availability time and assert rolling-origin/historical snapshots exclude future-available evidence. Related revision/translation/copied-template/event-episode variants must stay on one side of train/validation/test when leakage would inflate results.

## Event/graph recovery

Synthetic truth should define event instances, mentions, typed relations, roles, partial orders, and membership. Evaluate mention/relation precision/recall, relation sign/type recovery, contradiction detection, temporal ordering, and graph structure rather than only parser accuracy.

## Multilingual measurement validation

Use parallel/equivalent content and human-reviewed span/concept evidence to evaluate semantic-unit span F1, concept precision/recall, calibration/Brier score, language-specific error, and shared latent alignment. Topic/factor comparisons across language/time/template require appropriate invariance evidence.

## Topic true-parameter recovery

Generate corpora with known topic prevalence/content parameters, covariance, covariate effects, document coordinates, temporal drift, and method/background factors. Match recovered topics to truth before computing RMSE/bias/coverage. Evaluate seed/bootstrap stability and known-K recovery/acceptable-set behavior.

## Psychometric validation

For ESEM/DSEM simulations (Asparouhov & Muthén, 2009; Asparouhov et al., 2018; Marsh et al., 2014) evaluate loading/factor/path recovery, bias, RMSE, confidence/credible interval coverage, convergence, configural/metric/scalar or partial invariance as required, multilevel/multiple-membership effects, within/between decomposition, irregular-time dynamics, and posterior plausible-value propagation. These psychometric targets remain accepted-target.

## Network/cluster validation

Use known covariance/community structures. Evaluate CLR/log-ratio correlation recovery, edge sign/precision/recall/interval coverage/selection stability and cluster ARI/NMI/bootstrap stability. Raw compositional topic proportions are not validated through naïve Pearson correlation.

## CPU/GPU parity

CPU `f64` is reference. Required accelerator lanes execute real kernels; skipped GPU tests are failures. Compare objective values, parameters/posteriors, convergence, validation metrics, and deterministic artifacts under stated tolerances. Record peak VRAM, transfer, kernel time, precision mode, batch adaptation, OOM recovery, and fallback.

## LLM tests

Deterministic schema/security tests are primary. Bounded live tests use released contextual-orchestrator contracts when model conformance is material. Treat documents as prompt-injection data, require evidence-span grounding, test unsupported-claim rejection, record provider/model/prompt/reasoning hashes, and compare model/human agreement where the LLM acts as a rater. Model-backed Actions must use the approved `orchestrator/free` route and must not make an LLM authoritative for numerical or scientific acceptance.

## Monte Carlo acceptance

Simulation thresholds account for Monte Carlo standard error and interval uncertainty. Do not require an observed replication proportion to exceed the nominal target exactly when sampling variability makes that scientifically invalid.

## Exact-proof resource budgeting

Validation Evidence numerical proofs that add asymptotic work or material allocation require a measured resource contract before a production admission boundary is widened. For the bias-standard-error exact pair-distance path tracked by issue #491:

- retain realistic represented-input counterexamples and permutation/sign-mirror contracts for scientific correctness;
- keep the existing exact residual/pairwise-subtraction, checked-integer, GCD-reduction, and exact midpoint authority fail-closed;
- compare the current O(n²) pair-distance proof with an algebraically equivalent O(n) exact accumulator only under a proved sufficient admission condition, and compare a wider-integer/reference alternative separately;
- characterize checked-`u128` refusal as a function of sample count and aligned dyadic diameter/exponent spread rather than treating an integer cutoff as a scientific boundary;
- record allocation count and compiled layout measurements separately; field-width estimates are not allocation evidence;
- run `crates/validation_core/examples/bias_se_exact_proof_budget.rs` in release mode on a recorded CPU/OS/Rust toolchain and retain raw samples plus p95; the harness is integer-kernel characterization, not an HTTP buyer-path result;
- if a service/API buyer path is affected, measure the full applicable path and retain the `p95 <= 20 ms` target without shrinking samples, omitting proof work, or relying on unrealistic warm-cache setup;
- arithmetic representability alone does not authorize a production sample-count budget.

Until those measurements and exact-head gates exist, the `n<=16` production bias-SE exact pair-distance admission remains unchanged even when a larger represented-input counterexample is known.

## Release acceptance

A release requires one integrated protected head with all relevant scientific, numerical, security, migration, packaging, SBOM/provenance, accessibility, operational, and independent-review evidence passing. Planning validation, superseded-branch results, local-only results, and unexecuted benchmark tooling are supporting evidence, not release proof.

## References

The full APA 7th register is [`docs/research/standards-and-literature.md`](research/standards-and-literature.md). Method names used above cite Allen (1983) for interval algebra and Asparouhov & Muthén (2009), Asparouhov et al. (2018), and Marsh et al. (2014) for ESEM/DSEM.