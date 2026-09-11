# Six-observation mean-bias standard error: exact pair-distance rounding

## Problem

After GAP-114, the bounded pair-distance/midpoint proof covered four and five observations. A six-observation sample with exact represented residuals still fell through to the translated floating second-moment path:

```text
truth     = [0, 0, 0, 0, 0, 0]
recovered = [1120315269, 1513609015, 1569037659, 1789057504, 1807936669, 1914796738]
```

All values are exactly represented binary64 integers and all pairwise differences are exact. The fifteen squared pair distances sum to

```text
N = 2486413148856729212.
```

For `n=6`,

```text
SE(mean)^2 = sum_{i<j}(r_i-r_j)^2 / (n^2(n-1)) = N / 180.
```

`gcd(N,180)=4`, giving the exact reduced radicand `621603287214182303/45`. Its correctly rounded binary64 square root is bits `0x419c057d42fc5857`; the predecessor translated floating moment/`sqrt` path returns adjacent upper `0x419c057d42fc5858`. This is deterministic represented-input arithmetic error, not Monte Carlo uncertainty.

## Decision

Public RED `cb07b89bac926bbe0b0c69318310fe78642e62cb` adds permutations and sign mirrors for the six-observation sample. Causal repair `8e194dc1b0fa66cc923c5cb939bf3319ed0b4554` extends only the already bounded exact pair-distance path from `n=4..=5` to `n=4..=6`.

The proof requirements are unchanged: every signed residual and every pairwise residual difference must be finite and error-free in represented binary64; dyadic coefficients and squared pair distances use checked `u128`; the scientific denominator is checked `n^2(n-1)`; the exact ratio is reduced by GCD; the floating ratio and `sqrt` only seed a candidate; exact dyadic candidate-square and adjacent-midpoint comparisons alone authorize the returned binary64 value. Any proof or bounded integer failure returns to the established `bias.rs` fallback.

The O(n^2) proof remains deliberately bounded. This repair does not generalize pair-distance work to arbitrary sample counts, does not add arbitrary-precision production arithmetic, and does not claim globally correctly rounded `n>2` standard errors. A broader admission requires separate numerical and performance evidence.

CHANGELOG `d709199bfccca6d70f2cd91d7115474a0f8e04cd` records the contract. GAP-114 and all preceding Validation Evidence lineages remain inherited.

## Method and standards trace

Morris, White, and Crowther (2019) distinguish deterministic performance-measure calculations from Monte Carlo uncertainty; a one-ULP deterministic bias-SE error is therefore repaired in the estimator rather than treated as simulation noise. Published floating-point authority remains IEEE 754-2019 / ISO/IEC 60559:2020. IEEE P754 is active revision work, not a published replacement. AERA/APA/NCME public Testing Standards authority remains the 2014 edition while its revision proceeds.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

ISO/IEC. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
