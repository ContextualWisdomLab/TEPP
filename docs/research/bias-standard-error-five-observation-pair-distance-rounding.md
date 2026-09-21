# Five-observation mean-bias standard error: exact pair-distance rounding

## Problem

The Validation Evidence `bias_standard_error` contract evaluates the standard error of represented signed recovery bias. After GAP-113, the bounded exact pair-distance/midpoint proof applied only to four observations. A five-observation sample with exact represented residuals could therefore fall through to the translated floating second-moment path even when every residual and every pairwise difference was exact.

For

```text
truth     = [0, 0, 0, 0, 0]
recovered = [1342748146, 1434848064, 1525257611, 1685877224, 1771341094]
```

all inputs are exactly represented binary64 integers. The ten exact squared pair distances sum to

```text
N = 621298477313343404
```

For `n = 5`, the pair-distance identity is

```text
SE(mean)^2 = sum_{i<j}(r_i-r_j)^2 / (n^2 (n-1)) = N / 100.
```

Reducing by `gcd(N,100)=4` gives the exact radicand

```text
155324619328335851 / 25.
```

The correctly rounded represented-input standard error is binary64 bits
`0x4192caf164065ad0`. The predecessor translated floating moment/`sqrt` path returns the adjacent upper value `0x4192caf164065ad1`. This is deterministic arithmetic error, not Monte Carlo uncertainty.

## Constraints

The repair must preserve TEPP's Validation Evidence boundary. It must not move reusable static psychometric estimation into TEPP, replace numerical acceptance with an LLM, add arbitrary-precision production arithmetic, weaken residual or pairwise-difference exactness, or make the normal large-sample path quadratic. The repository's existing two- and three-observation identities remain owned by `bias.rs`.

## Decision

`crates/validation_core/src/bias_se.rs` now treats the pair-distance proof as a deliberately bounded small-sample reference path for four and five observations. The implementation:

- proves every signed residual exact in binary64;
- proves every pairwise residual difference exact;
- converts each nonzero difference to an exact dyadic coefficient and common power-of-two unit;
- accumulates the squared coefficients with checked `u128` arithmetic;
- forms the scientific denominator `n^2(n-1)` with checked integer arithmetic and reduces the exact ratio by GCD;
- uses the binary64 ratio/`sqrt` only to seed a candidate;
- admits a result only after exact dyadic candidate-square and adjacent-midpoint comparisons settle the correctly rounded binary64 value;
- falls back to the established `bias.rs` path whenever any proof or bounded integer operation fails.

The bounded observation-count admission is intentional. Generalizing the exact pair-distance proof to arbitrary `n` would introduce `O(n^2)` work on the ordinary estimator path and is not required to repair this demonstrated five-observation defect. A later extension requires its own performance and numerical evidence.

## Alternatives rejected

A payload-specific branch was rejected because the defect is the omitted five-observation pair-distance identity, not these particular integers. Replacing the translated path with an unconditional floating pair-distance formula was rejected because it would preserve the same rounded-ratio/`sqrt` ambiguity without exact midpoint authority. Extending the exact path to arbitrary sample counts in this repair was rejected because it changes asymptotic cost and buyer-path performance without evidence. Arbitrary-precision production arithmetic was rejected because the existing checked `u128` proof is sufficient for this bounded case and preserves the Rust-first deterministic reference contract.

## Validation and traceability

Public RED commit `5878ec10f458efb5f070446dcb3ead30900ef707` adds `crates/validation_core/tests/bias_standard_error_five_observation_pair_distance_contract.rs` with multiple permutations and sign mirrors. The predecessor exact head is GAP-113 research head `37e3defb3bde29a8d1ac852456ba05f787aee1f3`. Causal source repair commit `41298270e8e3d4476ba1bbad9f22ea94752a9e6a` generalizes only the bounded pair-distance helper from four to four-or-five observations and retains the existing fail-closed proof. CHANGELOG commit `4f6f61d398667936ecfa1d9209755e4c0d775e1f` records the contract.

The test target is a deterministic performance-measure calculation over known represented inputs. Morris, White, and Crowther distinguish the performance measure itself from Monte Carlo uncertainty and recommend explicit performance-measure definitions and Monte Carlo standard errors for simulation uncertainty. That distinction is why a one-ULP deterministic bias-SE error is repaired in the estimator rather than absorbed into simulation uncertainty.

IEEE/ISO floating-point authority remains the published 2019/2020 standard family. IEEE P754 is an active revision PAR approved June 6, 2024 and is not treated as a published replacement. The AERA/APA/NCME public Testing Standards authority remains the 2014 edition while a Joint Committee is revising that edition.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

ISO/IEC. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
