# Eleven-observation mean-bias standard error: exact pair-distance rounding

## Problem

After GAP-119, the bounded pair-distance/midpoint proof covered four through ten observations. An eleven-observation sample with exact represented residuals still fell through to the translated floating second-moment path:

```text
truth     = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
recovered = [50511426, 167164486, 318141475, 357712576, 407960427,
             441767841, 691573103, 733495428, 1082192974, 1543970183,
             1797594737]
```

All values are exactly represented binary64 integers and all pairwise differences are exact. The fifty-five squared pair distances sum to

```text
N = 35022924934975401574.
```

For `n=11`,

```text
SE(mean)^2 = sum_{i<j}(r_i-r_j)^2 / (n^2(n-1)) = N / 1210.
```

`gcd(N,1210)=2`, giving the exact reduced radicand `17511462467487700787/605`. Exact rational midpoint comparison gives the correctly rounded binary64 square-root bits `0x41a447fca4517b3f`; the predecessor translated floating moment/`sqrt` path returns adjacent lower `0x41a447fca4517b3e`. Reordering the same represented multiset and mirroring its sign preserve the mismatch. This is deterministic represented-input arithmetic error, not Monte Carlo uncertainty.

## Decision

Public RED `967e1e093603cbefc01889adce0087d744bfcd90` adds multiple permutations and sign mirrors for the eleven-observation sample. Causal repair `59c5a4ef3bb9693329bfb359cecf3bbd93eecb3b` extends only the already bounded exact pair-distance path from `n=4..=10` to `n=4..=11`.

The proof requirements are unchanged: every signed residual and every pairwise residual difference must be finite and error-free in represented binary64; dyadic coefficients and squared pair distances use checked `u128`; the scientific denominator is checked `n^2(n-1)`; the exact ratio is reduced by GCD; the floating ratio and `sqrt` only seed a candidate; exact dyadic candidate-square and adjacent-midpoint comparisons alone authorize the returned binary64 value. Any proof or bounded-integer failure returns to the established `bias.rs` fallback.

The O(n^2) proof remains deliberately bounded. This repair does not generalize pair-distance work to arbitrary sample counts, does not add arbitrary-precision production arithmetic, and does not claim globally correctly rounded `n>2` standard errors. `n=12` remains on the established path until a separate represented-input counterexample and performance evidence justify another extension.

CHANGELOG `2e9c6a54c417ccbe4e567b8f70e50175f1c47553` records the contract. GAP-119 and all preceding Validation Evidence lineages remain inherited.

## Alternatives rejected

A payload-specific eleven-value branch would encode a fixture rather than the estimator invariant. Replacing the translated fallback with an unconditional pair-distance formula would impose O(n^2) work on ordinary larger samples without evidence that the commercial path needs it. Raising the bound speculatively beyond the demonstrated case would likewise add quadratic work without a scientific defect to justify it. Arbitrary-precision production arithmetic would broaden ownership and dependency surface beyond the demonstrated defect. The bounded admission reuses the existing checked dyadic proof and changes only its evidenced upper sample count.

## Method and standards trace

Morris, White, and Crowther (2019) distinguish deterministic performance-measure calculations from Monte Carlo uncertainty; a one-ULP deterministic bias-SE error is therefore repaired in the estimator rather than treated as simulation noise. Published floating-point authority remains IEEE 754-2019 / ISO/IEC 60559:2020. IEEE P754 remains an active PAR approved June 6, 2024 to supersede 754-2019; it is revision work, not a published replacement. ISO continues to list ISO/IEC 60559:2020 as a published International Standard at stage 60.60. AERA, APA, and NCME continue revising the 2014 *Standards for Educational and Psychological Testing*; the unpublished revision is not treated as normative authority.

## Traceability

- Bounded context: Validation Evidence.
- Production module/API: `crates/validation_core/src/bias_se.rs` → `bias_standard_error`.
- Public executable contract: `crates/validation_core/tests/bias_standard_error_eleven_observation_pair_distance_contract.rs`.
- RED: `967e1e093603cbefc01889adce0087d744bfcd90`.
- Causal source repair: `59c5a4ef3bb9693329bfb359cecf3bbd93eecb3b`.
- CHANGELOG: `2e9c6a54c417ccbe4e567b8f70e50175f1c47553`.
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
