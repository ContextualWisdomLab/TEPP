# Eight-observation mean-bias standard error: exact pair-distance rounding

## Problem

After GAP-116, the bounded pair-distance/midpoint proof covered four through seven observations. An eight-observation sample with exact represented residuals still fell through to the translated floating second-moment path:

```text
truth     = [0, 0, 0, 0, 0, 0, 0, 0]
recovered = [121838114, 741324193, 994684355, 1673429661,
             1824103795, 1861311798, 1872117478, 1936395613]
```

All values are exactly represented binary64 integers and all pairwise differences are exact. The twenty-eight squared pair distances sum to

```text
N = 25648518292283252135.
```

For `n=8`,

```text
SE(mean)^2 = sum_{i<j}(r_i-r_j)^2 / (n^2(n-1)) = N / 448.
```

`gcd(N,448)=7`, giving the exact reduced radicand `3664074041754750305/64`. Exact rational midpoint comparison gives the correctly rounded binary64 square root bits `0x41ac860197accd4c`; the predecessor translated floating moment/`sqrt` path returns adjacent upper `0x41ac860197accd4d`. Reordering the same represented multiset and mirroring its sign preserve the same mismatch. This is deterministic represented-input arithmetic error, not Monte Carlo uncertainty.

## Decision

Public RED `40413dc9e3279f06c3d02ed2180ac9c170a51001` adds multiple permutations and sign mirrors for the eight-observation sample. Causal repair `26ca2d68bf057b93b9fef1e14953bedbdcf754c0` extends only the already bounded exact pair-distance path from `n=4..=7` to `n=4..=8`.

The proof requirements are unchanged: every signed residual and every pairwise residual difference must be finite and error-free in represented binary64; dyadic coefficients and squared pair distances use checked `u128`; the scientific denominator is checked `n^2(n-1)`; the exact ratio is reduced by GCD; the floating ratio and `sqrt` only seed a candidate; exact dyadic candidate-square and adjacent-midpoint comparisons alone authorize the returned binary64 value. Any proof or bounded integer failure returns to the established `bias.rs` fallback.

The O(n^2) proof remains deliberately bounded. This repair does not generalize pair-distance work to arbitrary sample counts, does not add arbitrary-precision production arithmetic, and does not claim globally correctly rounded `n>2` standard errors. Extending the bound again requires a demonstrated represented-input counterexample plus numerical and performance evidence rather than a speculative increase.

CHANGELOG `2641b2613a7c37e4153d9b8baa47487a1b80718e` records the contract. GAP-116 and all preceding Validation Evidence lineages remain inherited.

## Alternatives rejected

A payload-specific eight-value branch would encode a fixture rather than the estimator invariant. Replacing the translated fallback with an unconditional pair-distance formula would impose O(n^2) work on ordinary larger samples without evidence that the commercial path needs it. Raising the bound speculatively beyond the demonstrated case would likewise add quadratic work without a scientific defect to justify it. Arbitrary-precision production arithmetic would broaden ownership and dependency surface far beyond the demonstrated defect. The bounded admission instead reuses the existing checked dyadic proof and changes only its evidenced upper sample count.

## Method and standards trace

Morris, White, and Crowther (2019) distinguish deterministic performance-measure calculations from Monte Carlo uncertainty; a one-ULP deterministic bias-SE error is therefore repaired in the estimator rather than treated as simulation noise. Published floating-point authority remains IEEE 754-2019 / ISO/IEC 60559:2020. IEEE P754 is an active PAR approved June 6, 2024 to supersede 754-2019; it is revision work, not a published replacement. AERA, APA, and NCME continue to publish the 2014 *Standards for Educational and Psychological Testing* while their Joint Committee carries out the next revision.

## Traceability

- Bounded context: Validation Evidence.
- Production module/API: `crates/validation_core/src/bias_se.rs` → `bias_standard_error`.
- Public executable contract: `crates/validation_core/tests/bias_standard_error_eight_observation_pair_distance_contract.rs`.
- RED: `40413dc9e3279f06c3d02ed2180ac9c170a51001`.
- Causal source repair: `26ca2d68bf057b93b9fef1e14953bedbdcf754c0`.
- CHANGELOG: `2641b2613a7c37e4153d9b8baa47487a1b80718e`.
- Scientific invariant: for exact represented residuals admitted by the bounded proof, `SE(mean)^2 = sum_{i<j}(r_i-r_j)^2 / [n^2(n-1)]`; returned binary64 is authorized by exact candidate-square and adjacent-midpoint comparison, not by the rounded floating seed.
- Failure policy: if residual exactness, pairwise-difference exactness, checked `u128`, reduced-ratio, or midpoint proof fails, retain the established `bias.rs` fallback.
- Ownership: this remains TEPP Validation Evidence performance-measure arithmetic. It does not create reusable static psychometric estimation authority and does not copy fast-mlsirm source.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

American Educational Research Association. (2024, June 12). *Members of the Joint Committee for the revision of the Standards for Educational and Psychological Testing named*. https://www.aera.net/Newsroom/Members-of-the-Joint-Committee-for-the-Revision-of-the-Standards-for-Educational-and-Psychological-Testing-Named

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

IEEE Standards Association. (2024). *P754: Standard for floating-point arithmetic*. https://standards.ieee.org/ieee/754/11684/

ISO/IEC. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). https://www.iso.org/standard/80985.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
