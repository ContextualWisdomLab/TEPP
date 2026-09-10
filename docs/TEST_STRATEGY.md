# TEPP Test and Scientific Validation Strategy

**Status:** Accepted quality baseline aligned to PRD v0.4  
**Last reviewed:** 2026-09-11

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
- keep the exact represented-residual gate, GCD reduction, exact candidate/midpoint authorization, and fail-closed fallback;
- treat exact pairwise-f64 subtraction as a comparison/reference authority, not a scientific prerequisite or an unconditional first production route. Characterization `2bc1d2284d75154e020640adb573c1cfadf005fb` proves `[0,1,2^-54,2]` has exact neutral-zero coordinates even though a non-anchor pair subtraction rounds;
- do not require the represented minimum to be the exact anchor. Source RED `fd9f9ff2c5c395e4cc13042232f4deef018adb48` uses `[0,1,2,-2^53]`: minimum `-2^53` cannot exactly translate `1`, while neutral zero preserves all coordinates. Exact `P=243388915243820099130562543878155`, denominator `48`, and public result `0x4320000000000001` differ by one ULP from the predecessor fallback;
- do not restrict exact translation origins to observed residuals. Follow-up RED `9f403194a2ec1636531c2dfe9229cfb34b73d747` uses `[1,2^-54,2,3]`, where no observed residual exactly translates every other residual, but neutral dyadic origin `0` preserves all. On unit `2^-54`, exact `P=6490371073168534319490338297741315`; exact result `0x3fe4a7e9cb8a3491` differs by one ULP from predecessor fallback `0x3fe4a7e9cb8a3492`;
- retain the correctness lineage through repair `6cf30eeb549c0df0377bda1111cf46396e8282a3`, but do not retain its quadratic route order as production policy. Source-level route-order RED `e0b324864e48a503e2aba0d2a487a0b95f5276ed` required a linear neutral-zero proof before quadratic work; repair `2b62bd46eb0c391327d2285c2244a76f5a1e0449` implemented it. A later retention criterion required conditioned observed-anchor search to prove unique bounded admission after neutral-zero refusal. The checked-in corpus supplied no such fixture, so RED `d40bfbf98b36562164d15d505f8f8825fd1c1349` → repair `14e7862f4ddccce54f3b93d4dac89adbf047ba77` removed that unsupported O(n²) route. Production order is now `neutral_zero_linear -> pairwise_reference (n <= 16 only) -> generic_fallback`; for `n > 16`, O(n) proof refusal skips pairwise enumeration and delegates directly to the established generic implementation. Neither superseded RED finished a hosted failing run and neither is cited as hosted RED evidence;
- test the neutral-zero kernel as O(n) in loop structure and O(1) in *proof* storage after the already-required residual vector. Do not describe the whole public call as O(1) space while it still materializes residuals;
- preserve pair-versus-neutral-zero equality wherever both admit and explicitly test neutral-zero-only admissions. Commit `da703fc50e136ed79e12262909c9f400a3945621` mirrors the production signed neutral-zero arithmetic, keeps exact pair/linear numerator and reduced-ratio equality for represented `{0,1,2^53}` at `n=4,16,17,65,257,2050`, preserves order invariance, and adds mixed-sign `[-2^53,0,1,2^53]`, where pairwise refuses but neutral-zero admits `P=2^109+3` and public bits `0x432a20bd700c2c3e`. Broaden deterministic represented-input equality beyond these fixtures before promotion;
- do not reintroduce conditioned observed-anchor scanning without a represented fixture that proves unique scientific admission value after neutral-zero bounded refusal; O(n²) search is not retained by assumption;
- neutral zero is a translation origin, not synthetic evidence: the observed residual values are unchanged. Do not claim it is globally resource-optimal without a separate proof;
- compare buffered O(n²), allocation-free two-pass O(n²), normalized narrow O(n), dependency-free Wide256 O(n), predecessor narrow→pair hybrid, and narrow→Wide256→pair candidate resource shapes when characterizing proof/resource trade-offs; do not turn a characterization threshold back into a production sample-count staircase;
- normalize the common power-of-two dyadic unit before narrow checked O(n) intermediates are judged. Raw `D=2^58,n=65` refusal is invalid after normalization; odd `D=2^58+1,n=65` remains the narrow 129-bit product witness;
- retain the earlier nonnegative minimum-anchor accumulator theorem `b7e4da353ac58069afd73ee7c0e8427d49993fdb`, `Σc_i <= Σc_i² <= P`, only within that characterized representation. Production neutral-zero coordinates may be signed, so positive and negative coefficient mass and resulting signed sum must be exercised separately;
- retain RED `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993` → repair `e9a7dee29afb97542bfe2965f850c8ab5a34368e`: canonical `u128` cancellation operands require no more than 256-bit products;
- retain represented reachability `5a19b6334487b43fb630abba7e487d7cf4c49960`: at `n=4096`, `{0,1,2^53}` reaches narrow-product overflow while Wide256 recovers exact 119-bit `P=664_289_479_338_799_435_974_172_876_300_357_631`;
- retain exact-rounding-width characterization `a8423173188fa53a26a16d3afdafeb76e114cc1d`: at represented `n=2050`, `P=332_306_998_946_228_931_332_463_617_650_984_961`, denominator `8_610_922_500`, candidate-square comparison requires 136 bits and the upward midpoint 140 bits;
- retain exponent-safe comparison characterization `aab9fe9115cee97225f2aa81e54a55ceafb23336`, production comparison RED `f7717361ad8c5f0592688c1514c104cc1b4adabe` → repair `e4a85f53a611922be7492fe906d62ce65787c18e`, and tie-to-even edge contract `1240ace8eb41a01fa72a4bb99df842fd550a1288`;
- retain historical minimum-anchor pair/Wide256 exact-ratio equivalence `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943` only within its characterized input family; use `da703fc50e136ed79e12262909c9f400a3945621` as signed neutral-zero equivalence evidence rather than copying production arithmetic as an independent oracle;
- keep normalized O(n) intermediate, exact pair-numerator, and denominator envelopes separate. At aligned diameter `2^53`, `n<=2_047`, `n<=4_095`, and `n<=208_064` are arithmetic evidence points, not production budgets;
- record exact pair counts, target `size_of::<Option<(u128, i32)>>()`, scratch `Vec` capacity/payload, and allocator/RSS evidence separately; field-width estimates are not allocation evidence;
- use `crates/validation_core/examples/bias_se_exact_proof_budget.rs` to prove exact restored-numerator equality before timing, but do not mislabel that characterization harness as production route telemetry. Production evidence must distinguish at least `neutral_zero_linear`, `pairwise_reference`, and `generic_fallback`;
- run the characterization and surviving production-route harness in release mode on recorded CPU/OS/Rust 1.98.0 and retain raw CSV plus p95. Unexecuted harness code is not performance evidence;
- if a service/API buyer path is affected, measure the complete applicable path against `p95 <= 20 ms` without shrinking samples, omitting proof work, or using unrealistic warm-cache-only setup;
- arithmetic representability, one successful exact origin, or one common-domain equality fixture does not authorize a production sample-count budget.

Current production behavior is proof-driven: the checked O(n) neutral-zero exact path is eligible for every `n >= 3`; only the O(n²) pairwise reference remains bounded to `n <= 16` after O(n) refusal, and refusal above sixteen delegates directly to the established general implementation. Release acceptance still requires broad same-head correctness, exact-head gates, independent review, bounded resource evidence, and applicable buyer-path measurements.

## Release acceptance

A release requires one integrated protected head with all relevant scientific, numerical, security, migration, packaging, SBOM/provenance, accessibility, operational, and independent-review evidence passing. Planning validation, superseded-branch results, local-only results, and unexecuted benchmark tooling are supporting evidence, not release proof.

## References

The full APA 7th register is [`docs/research/standards-and-literature.md`](research/standards-and-literature.md). Method names used above cite Allen (1983) for interval algebra and Asparouhov & Muthén (2009), Asparouhov et al. (2018), and Marsh et al. (2014) for ESEM/DSEM.