# Mean-bias pairwise subtraction roundoff

## Problem

`mean_bias` is the Validation Evidence performance measure `mean(recovered - truth)` over represented binary64 recovery values. The predecessor treated each finite pairwise subtraction as authoritative before aggregation. That is not algebraically neutral when a finite subtraction rounds away a low-order dyadic term and several subsequent rounding steps place the mean on a binary64 midpoint.

Public RED `a9d9bda3df1533eb91ece8e3bc6ce597b9c1400f` uses

- `truth = [2^-108, 2^-53]`
- `recovered = [2^-54, 1]`.

The first exact represented-input residual is `2^-54 - 2^-108`, but binary64 subtraction rounds it to `2^-54`. The second residual is the exactly representable `1 - 2^-53`. The predecessor therefore averaged the rounded residuals to the ties-to-even result `0x3fe0000000000000` (`0.5`). The represented-input numerator still contains `-2^-108`; after division by two it lies strictly below that midpoint, so the correct represented mean is `0x3fdfffffffffffff`, the float immediately below `0.5`. The sign mirror has the symmetric expected bits `0xbfdfffffffffffff`.

## Constraints

The repair must preserve the existing exact-subtraction fast path, the overflow-safe recovered-plus-negated-truth fallback, canonical order stability, original recovery-unit denominator, fail-closed underflow/overflow semantics, and the earlier subnormal one-rounding contract. It must not create a second public estimator, copy reusable psychometric arithmetic from fast-mlsirm, or add an arbitrary-precision production dependency.

## Repair lineage

The first source repair `96bff8e55083fc791650b185f819cd3aa90dac1b` correctly detected nonzero error-free subtraction roundoff and redirected such finite cases to the existing expanded represented-input numerator. The RED then exposed a second rounding loss inside that path: mixed-remainder Neumaier compensation stored its correction in one binary64 value, so the `-2^-108` contribution could disappear when the leading correction already sat at the final midpoint.

Corrected causal repair `676ccda02da0186361555879d5637c4f02178a71` retains a second-order correction tail while accumulating the mixed remainder. It carries the leading division residual, correction numerator residual, correction-division residual, and retained compensation tail through the original scientific denominator. The final candidate is moved to an adjacent float only when the exact two-term tail is beyond the binary64 midpoint, or is exactly at the midpoint and ties-to-even selects the adjacent value. This keeps ordinary exact-subtraction inputs on the predecessor path and does not claim globally correctly rounded summation/division for every binary64 sequence.

## Alternatives rejected

Always accepting pairwise-rounded residuals was rejected because the RED proves that a finite intermediate subtraction can change the represented scientific result. Always expanding every bias numerator was rejected because it would unnecessarily change the established exact-residual and all-subnormal paths, including the bounded exact-unit rule added for GAP-090. Adding an arbitrary-precision runtime dependency was rejected because the demonstrated defect is resolved inside the existing deterministic binary64 reference without creating a new numerical owner or deployment dependency. Treating the first expanded-numerator patch as complete was rejected because the original RED remained failing until second-order mixed-remainder compensation was retained through final rounding.

## Scientific and standards trace

IEEE 754-2019 remains the active published floating-point standard; IEEE P754 is an active revision PAR rather than a published replacement. ISO/IEC 60559:2020 remains the published international standard adopting the same floating-point arithmetic model. AERA, APA, and NCME continue to publish the 2014 *Standards for Educational and Psychological Testing* while the Joint Committee conducts its announced revision. Morris, White, and Crowther (2019) provide the methodological basis for treating bias as a known-truth simulation performance measure and for reporting simulation design and Monte Carlo uncertainty explicitly.

References:

- IEEE. (2019). *IEEE Standard for Floating-Point Arithmetic* (IEEE Std 754-2019).
- ISO/IEC. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).
- American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. AERA.
- Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

## Scope and remaining risk

This repair establishes the public represented-input counterexample and its sign mirror. It does not claim that every possible binary64 mean is globally correctly rounded, nor does it alter `bias_standard_error`, RMSE, Monte Carlo, Wilson, Longitudinal Modeling, or fast-mlsirm ownership. A future GAP requires a separate public counterexample where the current exact-head return value disagrees with the represented-input estimand or admission decision; algebraic suspicion alone is insufficient.
