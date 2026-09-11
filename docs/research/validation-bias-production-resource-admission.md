# Bias standard-error production resource admission

## Decision scope

This note separates two contracts that must not be collapsed to satisfy coverage.

- `validation_core::bias_standard_error` owns numerical truth and fail-closed behavior for admitted represented inputs.
- the Analysis Run application boundary owns how much evidence one in-memory product execution may admit.

A product resource bound is not, by itself, a new mathematical precondition on the public validation library. Conversely, a source-domain witness that is representable by Rust slices is not automatically a realistic product acceptance fixture.

This note changes neither production arithmetic nor scientific acceptance. It records the owner boundary needed to close the remaining bias-SE coverage obligations without an arbitrary sample-count cutoff or duplicated cross-context policy.

## Existing product admission evidence

`analysis_engine` already declares `MAX_EVIDENCE_UNITS = 100_000`. `AnalysisCorpus::new` returns `AnalysisEngineError::LimitExceeded` when an in-memory evidence corpus exceeds that limit.

That constant belongs to the Analysis Run application boundary. `validation_core` does not currently consume a typed/versioned admission object that proves every production call to `bias_standard_error` came through `AnalysisCorpus`. `analysis_engine` also does not currently depend on `validation_core`, and `AnalysisEvidenceUnit` carries evidence identity, event time, availability time, and membership count rather than a truth/recovered numerical pair.

Therefore an evidence-unit count is not presently a proven bound on the sample count `n` supplied to `bias_standard_error`. A future buyer path must state and enforce the cardinality mapping between admitted evidence and validation truth/recovered pairs before `MAX_EVIDENCE_UNITS` can participate in a bias-SE resource proof. The value `100_000` must not be copied into `validation_core`, treated as a public-library domain restriction, or used to delete numerical refusals.

The missing trace is architectural and semantic, not arithmetic: establish the supported buyer call path from an admitted Analysis Run population to the Validation Evidence computation, state its cardinality contract, then keep the resource policy at its owner and pass only the admitted population/contract required by the numerical boundary.

## Helper false-zero witness and product materialization

The remaining `standard_error_from_deviations` false-zero refusal is source-domain reachable. The known construction uses

- `n = 2^54 = 18,014,398,509,481,984`,
- the minimum positive binary64 subnormal `q = 2^-1074`,
- `M = 2^-1021 = 2^53 q`,
- one `M` and `n - 1` copies of `q`.

The normalized standard error is `2^-54`; restoring by `M` produces the exact real value `q/2`, which ties to represented zero under round-to-nearest, ties-to-even. The guard therefore protects a real positive estimand from a false represented zero.

For scale only, the witness count is about `1.8014398509481984e11` times `100_000`. That ratio is not an admission proof because evidence-unit cardinality is not yet metric-sample cardinality. Independently of the product path, materializing only the two public `f64` input slices at `n = 2^54` would require

`2 × 2^54 × 8 = 2^58 = 288,230,376,151,711,744 bytes = 256 PiB`,

before residual, subtraction-roundoff, translated, normalized, square, allocator, and runtime overhead.

The consequence is not that the branch is unreachable. The consequence is that product acceptance needs a bounded representation/resource contract rather than a giant allocation test. The fail-closed guard remains part of numerical safety unless the public library contract itself is deliberately changed with an owner-correct, versioned admission boundary.

## Translated-dispersion proof target

The repository-owned exact-real theorem already gives, for every admitted general translated path with `n >= 3`,

`D = n Σ y_i² - (Σ y_i)² >= n/2`.

The remaining production branch is about the implemented floating path, not exact-real variance. The implementation currently rounds normalized squares, sorted compensated first and second moments, `usize -> f64`, and `n * Q` separately, then evaluates the final subtraction with one FMA.

A buyer-bound proof may use an Analysis Run population limit only after the buyer call trace and the evidence-to-metric cardinality contract are explicit. If those contracts prove every supported product bias-SE execution has `n <= 100_000`, the implementation-matched forward-error obligation can be bounded on that finite admitted range. That would narrow product acceptance evidence; it would not silently redefine the source domain of the public validation library.

Until that trace exists, the numerical proof must continue to state its actual domain. No line or branch may be declared unreachable merely because current application data are smaller.

## Required ownership and test sequence

1. Analysis Run owns and versions the product resource-admission rule. Do not duplicate `MAX_EVIDENCE_UNITS` in Validation Evidence.
2. Add or identify the typed application contract that proves the population passed to the supported buyer Validation Evidence path was admitted under that rule, including the cardinality mapping from evidence units to truth/recovered metric pairs.
3. Test the application boundary at the maximum admitted metric population and at the first refused population without allocating an artificial source-domain witness beyond the product contract.
4. Derive the floating translated-dispersion error bound against the exact implementation and the supported product metric-population range. Include square rounding, deterministic sorted compensation, final correction addition, sample-count conversion, multiplication, and FMA.
5. Keep independent public-library numerical tests for compact caller-valid counterexamples and edge arithmetic. If a compact counterexample reaches `dispersion_numerator <= 0.0`, preserve it as a deterministic regression rather than replacing it with a product-size assumption.
6. Only remove a fail-closed branch when its impossibility follows from the explicit contract and repository-owned proof. Coverage suppression, `skip`/`xfail`, source rewriting, and denominator changes are not evidence.

## Acceptance consequence

The current two missing bias-SE branches have different closure routes.

- The helper false-zero branch is known reachable in the public source domain but presently outside realistic in-memory materialization. Close its product acceptance gap through an owner-correct bounded representation/resource contract while preserving numerical fail-closed behavior.
- The translated nonpositive-dispersion branch has an exact-real positive margin but still needs an implementation-matched floating proof or a compact caller-valid counterexample. A verified buyer-path population and cardinality contract can narrow the product proof domain, but cannot substitute for that proof.

This separation keeps resource policy, metric cardinality, numerical truth, and coverage evidence independently auditable.
