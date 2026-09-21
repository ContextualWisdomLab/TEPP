# Two-replication nearest-rank sample exhaustion

## Scientific contract

`MonteCarloSummary` stores a represented sample mean, sample standard deviation (`n - 1` denominator), standard error of the mean, and inclusive nearest-rank percentile endpoints selected from retained replications. The carrier remains sign-neutral because it can summarize signed metrics such as bias as well as nonnegative metrics such as the RMSE summaries admitted by `ValidationReport`.

For more than two replications, lower and upper percentile endpoints do not identify the unreported retained values, so TEPP only enforces support that follows from the stored moments. The `n = 2` case is different. When the two nearest-rank endpoint values are numerically distinct, they must designate the two distinct retained observations. There are no remaining replications whose values could alter the represented mean or sample spread.

Therefore, for `replication_count = 2` and `percentile_lower != percentile_upper`, the retained sample is exactly

`[percentile_lower, percentile_upper]`.

The recorded mean and sample SD must be coherent with that exhausted sample. This is stronger than the generic individual endpoint-radius and joint squared-deviation budget, but only in the finite-sample case where the artifact itself identifies every retained value.

The public RED uses endpoints `[-0.5, 0.5]`. The canonical producer returns represented mean `0.0` and sample SD `sqrt(0.5)`. A payload that records the same two endpoint values with `SD = 1.0` and `SE = 1/sqrt(2)` passes the predecessor's standard-error coherence, individual endpoint support, and joint deviation-budget checks, yet no two-observation retained sample can produce it. A second fixture records mean `0.25`; it likewise satisfies the looser moment budget but cannot be the mean of the only two retained values.

Equal numeric endpoints are not treated as sample exhaustion because nearest-rank lower and upper requests may select the same retained rank, or two retained observations may share the same represented value. Without percentile probabilities or rank multiplicity in the artifact, inferring two distinct observations from one numeric endpoint would overconstrain valid summaries.

## Numerical implementation

For the exhausted two-value case, admission reuses the same deterministic represented-mean and scaled sample-SD references as `summarize_replications`. The recorded and reconstructed values are compared after normalization by their own maximum magnitude, with the existing empirical-support relative tolerance. This avoids raw full-range subtraction and does not scale a near-zero mean comparison by the much larger endpoint magnitudes, which would otherwise admit a materially wrong near-zero mean.

The normalized operands are finite and bounded by one, so the final relative-distance calculation cannot create a non-finite validation-only intermediate. The implementation therefore does not add an unreachable finite-check branch that would weaken owned branch-coverage evidence.

## RED, causal repair, and traceability

- Public RED `81bf0d9e2f1a28947b1343244002d6762b703f8a` adds `crates/validation_core/tests/monte_carlo_two_replication_endpoint_exhaustion_contract.rs` and demonstrates a two-replication summary that passes the predecessor's looser support checks but is not realizable from the exposed endpoint values.
- Causal implementation was corrected and stabilized at `d48f8fef08e77b8fa654f2852814c25c5d1baa79` after immediate source review of the first contents update; `fb314d8afccdf19f0074c64f6277d2edb290e907` adds explicit represented-mean coverage, and `440b78d86908fe1464e65bdbc4ceb5f9f6606c9f` removes an unreachable non-finite comparison branch while retaining the same scientific contract.
- Changelog trace `45116498f29c0d3421192d452e26182975b114ae` records the durable-evidence repair.

This remains TEPP Validation Evidence artifact admission. It does not add a psychometric estimator, change Longitudinal Modeling composition, or copy mutable arithmetic from fast-mlsirm.

## Methodological reference

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

The ADEMP framing separates estimands, methods, data-generating mechanisms, and performance measures. TEPP applies that discipline at the evidence boundary: a reported performance summary is admitted only when the stored finite-sample fields can coexist under the producer contract actually used to generate them.
