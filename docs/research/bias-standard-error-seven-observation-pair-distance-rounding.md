# Seven-observation mean-bias standard error: exact pair-distance rounding

## Problem

After GAP-115, the bounded pair-distance/midpoint proof covered four through six observations. A seven-observation sample with exact represented residuals still fell through to the translated floating second-moment path:

```text
truth     = [0, 0, 0, 0, 0, 0, 0]
recovered = [912628433, 991631865, 1109185293, 1253685899, 1354619842, 1368206500, 1611374925]
```

All values are exactly represented binary64 integers and all pairwise differences are exact. The twenty-one squared pair distances sum to

```text
N = 2469379766402987422.
```

For `n=7`,

```text
SE(mean)^2 = sum_{i<j}(r_i-r_j)^2 / (n^2(n-1)) = N / 294.
```

`gcd(N,294)=2`, giving the exact reduced radicand `1234689883201493711/147`. Its correctly rounded binary64 square root is bits `0x4195d9b70ca9e6ee`; the predecessor translated floating moment/`sqrt` path returns adjacent upper `0x4195d9b70ca9e6ef`. Reordering the same represented multiset and mirroring its sign preserve the same mismatch. This is deterministic represented-input arithmetic error, not Monte Carlo uncertainty.

## Decision

Public RED `a1de7ae90ae57e0eb55d7efe0b8bf6d9f5a6f3f3` adds multiple permutations and sign mirrors for the seven-observation sample. Causal repair `2907e468363040a435fde4dd14e74ca32cc3b082` extends only the already bounded exact pair-distance path from `n=4..=6` to `n=4..=7`.

The proof requirements are unchanged: every signed residual and every pairwise residual difference must be finite and error-free in represented binary64; dyadic coefficients and squared pair distances use checked `u128`; the scientific denominator is checked `n^2(n-1)`; the exact ratio is reduced by GCD; the floating ratio and `sqrt` only seed a candidate; exact dyadic candidate-square and adjacent-midpoint comparisons alone authorize the returned binary64 value. Any proof or bounded integer failure returns to the established `bias.rs` fallback.

The O(n^2) proof remains deliberately bounded. This repair does not generalize pair-distance work to arbitrary sample counts, does not add arbitrary-precision production arithmetic, and does not claim globally correctly rounded `n>2` standard errors. Extending the bound again requires a demonstrated scientific counterexample plus numerical and performance evidence rather than a speculative increase.

CHANGELOG `78d5ed3fb81765edbc4ee662dacabe6ec631265f` records the contract. GAP-115 and all preceding Validation Evidence lineages remain inherited.

## Alternatives rejected

A payload-specific seven-value branch would encode a fixture rather than the estimator invariant. Replacing the translated fallback with an unconditional pair-distance formula would impose O(n^2) work on ordinary larger samples without evidence that the commercial path needs it. Arbitrary-precision production arithmetic would also broaden ownership and dependency surface far beyond the demonstrated defect. The bounded admission instead reuses the existing checked dyadic proof and changes only its scientifically evidenced upper sample count.

## Method and standards trace

Morris, White, and Crowther (2019) distinguish deterministic performance-measure calculations from Monte Carlo uncertainty; a one-ULP deterministic bias-SE error is therefore repaired in the estimator rather than treated as simulation noise. Published floating-point authority remains IEEE 754-2019 / ISO/IEC 60559:2020. IEEE P754 is active revision work, not a published replacement. AERA/APA/NCME public Testing Standards authority remains the 2014 edition while its revision proceeds.

## Traceability

- Bounded context: Validation Evidence.
- Production module/API: `crates/validation_core/src/bias_se.rs` → `bias_standard_error`.
- Public executable contract: `crates/validation_core/tests/bias_standard_error_seven_observation_pair_distance_contract.rs`.
- Scientific invariant: for exact represented residuals admitted by the bounded proof, `SE(mean)^2 = sum_{i<j}(r_i-r_j)^2 / [n^2(n-1)]`; returned binary64 is authorized by exact candidate-square and adjacent-midpoint comparison, not by the rounded floating seed.
- Failure policy: if residual exactness, pairwise-difference exactness, checked `u128`, reduced-ratio, or midpoint proof fails, retain the established `bias.rs` fallback.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

ISO/IEC. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
