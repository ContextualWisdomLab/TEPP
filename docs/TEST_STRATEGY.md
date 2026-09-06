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
- compare the buffered O(n²) pair-record path, an allocation-free two-pass O(n²) reference, an algebraically equivalent narrow O(n) exact accumulator under a proved sufficient admission condition, a dependency-free two-limb wider-product O(n) characterization reference, the predecessor narrow-O(n)-fast-path/buffered-pair-fallback hybrid, and the implemented `narrow O(n) -> Wide256 O(n) -> pairwise fail-closed fallback` candidate;
- normalize the largest common power-of-two dyadic unit from exact anchor-relative coefficients before checked O(n) intermediates are judged. Raw-scale overflow is not a scientific or resource refusal when exact dyadic rescaling removes it;
- require an admitted/refused-set contract for any O(n) candidate. The predecessor `D=2^58, n=65` refusal was invalid because canonical normalization reduces the coefficients to zero/one and preserves the restored pair numerator `2^122`. The corrected narrow-path strict-subset boundary uses odd `D=2^58+1`: both pair and normalized narrow linear kernels fit at `n=64`, while at `n=65` the exact pair numerator still fits `u128` but the normalized narrow O(n) products require 129 bits before cancellation;
- retain the wider-product characterization `081000289f5a52e94863026d55696ee2a4daf923`: on odd `D=2^58+1, n=65`, exact two-limb products and subtraction recover the same 123-bit pair numerator after the two 129-bit intermediates cancel. This proves that narrow-intermediate refusal is not scientific refusal;
- retain accumulator-bound characterization `b7e4da353ac58069afd73ee7c0e8427d49993fdb`: after canonical minimum anchoring, all coefficients are nonnegative integers and at least one is zero, so `Σc_i <= Σc_i² <= Σ(i<j)(c_i-c_j)²`. Therefore any canonical pair numerator that fits `u128` necessarily bounds the coefficient and square accumulators. A pair-admitted `Σc_i` or `Σc_i²` overflow fixture is not a valid acceptance target;
- the remaining wider-path equivalence work is at the production represented-input boundary: verify full-width multiplication/subtraction, exact common-unit restoration, reduced denominator/midpoint comparison, and upstream exact-residual conversion semantics. Keep the pairwise path as a fail-closed reference/fallback until those contracts are executable and current-head GREEN;
- characterize checked-`u128` and wider-reference behavior as functions of sample count, canonical aligned dyadic diameter/exponent spread, and coefficient distribution rather than treating an integer cutoff or raw represented scale as a scientific boundary;
- keep the normalized O(n) distribution-independent intermediate envelope distinct from the exact pair-square numerator envelope: at aligned coefficient diameter `2^53`, the characterized sufficient bounds are `n<=2_047` and `n<=4_095` respectively, and neither is a production budget;
- record exact pair counts, target `size_of::<Option<(u128, i32)>>()`, actual scratch `Vec` capacity, scratch payload bytes, and allocator/RSS evidence separately; field-width estimates alone are not allocation evidence;
- use `crates/validation_core/examples/bias_se_exact_proof_budget.rs` to prove exact restored-numerator equality before timing. RED `3136739460ef0c8e13c044a7e5b04891e4f4e23d` required the missing narrow→Wide256→pair route and repair `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5` implements it. On odd `D=2^58+1, n=65`, the predecessor hybrid remains a pair-allocation comparison baseline while the new candidate must recover through `Wide256` with `used_wide_product=true` and `used_pairwise_fallback=false`; the harness records both route flags explicitly;
- keep RED `4f1bd2c343cf2d54905a07c257a570a89dc575d3`, normalization repair `d423b57797b6f7f127e61e0679f9ee9841525c77`, restoration hardening `96f17c02edba0792f61e0e92167703a6ae4e40d0`, wider-product characterization `081000289f5a52e94863026d55696ee2a4daf923`, wider-reference harness `0bd805d4b0304cf1f76344ae14b7f079b3dade17`, accumulator-bound characterization `b7e4da353ac58069afd73ee7c0e8427d49993fdb`, narrow-wide-pair RED `3136739460ef0c8e13c044a7e5b04891e4f4e23d`, and narrow-wide-pair repair `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5` in the exact evidence lineage;
- run the characterization harness in release mode on a recorded CPU/OS/Rust 1.98.0 toolchain and retain raw CSV plus p95; unexecuted harness code is not performance evidence;
- if a service/API buyer path is affected, measure the full applicable path and retain the `p95 <= 20 ms` target without shrinking samples, omitting proof work, or relying on unrealistic warm-cache setup;
- arithmetic representability alone does not authorize a production sample-count budget.

Until those measurements and exact-head gates exist, the `n<=16` production bias-SE exact pair-distance admission remains unchanged even when a larger represented-input counterexample is known.

## Release acceptance

A release requires one integrated protected head with all relevant scientific, numerical, security, migration, packaging, SBOM/provenance, accessibility, operational, and independent-review evidence passing. Planning validation, superseded-branch results, local-only results, and unexecuted benchmark tooling are supporting evidence, not release proof.

## References

The full APA 7th register is [`docs/research/standards-and-literature.md`](research/standards-and-literature.md). Method names used above cite Allen (1983) for interval algebra and Asparouhov & Muthén (2009), Asparouhov et al. (2018), and Marsh et al. (2014) for ESEM/DSEM.
