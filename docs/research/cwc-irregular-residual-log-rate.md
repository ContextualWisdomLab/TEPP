# CWC-then-irregular residual log-rate

## Domain owner

This contract belongs to TEPP's **Longitudinal Modeling** bounded context.
Public operations whose meaning depends on substantive event time expose
`EventTimeInterval` rather than a generic clock label. Unique evidence is
folded from Draft #327 into `longitudinal_core` on landing vehicle #310.
Reusable static/generalized-mixed kernels remain fast-mlsirm-owned.

This is not DSEM, not Newton least-squares, not ctsem estimation, and not
raw-process autoregressive drift.

## Estimands

1. `center_within_unit_event_lags` subtracts the unit mean (CWC) and emits
   consecutive [`LaggedWithinResidual`] pairs on admitted event intervals.
   Singleton units are skipped and at least two units must contribute lags.
2. `recover_within_unit_irregular_residual_log_rate` is the mean of the
   Driver, Oud, and Voelkle (2017, Eq. 3) scalar inverse
   `a = ln(|later| / |earlier|) / Δt` on nonzero same-sign residuals. When
   the absolute residual ratio is finite and positive the finite-ratio
   logarithm is used; overflowed or underflowed ratios fall back to
   `ln|later| − ln|earlier|`.
3. `recover_centered_irregular_residual_log_rate` is the already-centered
   path. It does **not** re-center. Residuals must be nonzero and have equal
   sign. Known-truth pairs `(1, 0.5)` with `Δt = 1` recover `ln(0.5)`; for a
   general admitted interval the scalar result is `ln(0.5) / Δt`.
4. `refuse_cwc_residual_log_rate_as_raw_process_drift` always fails closed.

## Numerical mean contract

CWC means, occasion means, and admitted irregular-rate means share
`irregular_residual::scaled_compensated_mean`. Same-sign finite terms use an
exact power-of-two scale, deterministic ordering, and compensated summation so
an avoidable raw partial-sum overflow does not reject a representable mean and
non-power-of-two normalization does not add an earlier rounding step.

Mixed-sign inputs cancel opposite signs from the largest magnitudes first.
The surviving one-sign residuals are then normalized and compensated, but the
division uses the **original sample count directly** before scaling back. The
implementation must not first round a mean over only the surviving residuals
and then weight that rounded intermediate. For the minimum-subnormal ULP `u`,
`[-20u, -20u, 9u]` has exact mean `-31u/3`, which rounds once to `-10u` in
binary64; the retired retained-mean-then-weight path produced `-11u`.

RED `ae5e61f9a829adbfed2ea13c5705d4b85d80b0d6` pins this through the public
CWC API. Causal repair `39469067aca2fa93e2fa4c914848f7cec8031811`
applies the original denominator inside the shared normalized sum. Contract
`mixed_sign_mean_rounding_contract.rs` also exercises the public occasion-mean
path so the shared numerical authority cannot silently split again.

A second boundary applies after the normalized sum. If the retained residual
mass has one sign and is nonzero, its real mean is also nonzero. Binary64 may
still round that final mean to exact zero when the magnitude lies below half
the minimum subnormal. Reporting that zero would change a nonzero temporal
rate into exact no-change. `same_sign_mean_over_total` therefore accepts zero
only when its admitted mass was itself exact zero; otherwise final zero is
`InvalidTemporalTransformInput`.

Public RED `96f1c3342071173ba870e6ef5e11b826391c7621` adds
`irregular_rate_mean_underflow_contract.rs`. One already-centered pair has an
exact zero rate and one has a representable positive subnormal rate; averaging
that nonzero rate with fifteen exact-zero rates makes the real mean positive
but too small for binary64. Causal repair
`ae5081d8ae580c19e73aff7f03711e50c3c631dd` fails closed at the shared mean
boundary rather than returning exact zero. Exact cancellation of mixed-sign
rates still returns zero before the one-sign helper, and an all-zero input
still returns canonical `+0.0`.

This arithmetic remains a Longitudinal composition primitive in this stack.
It is not a new reusable static psychometric kernel. A fast-mlsirm handoff
requires semantic-equivalence evidence and an immutable released owner
contract rather than source copying.

## Identification and admissibility

Curran and Bauer (2011, pp. 583–619; PMC3059070 XML opened 2026-09-02)
show that person-mean centering of a time-varying covariate related to time
is biased for the within-person effect. The licensed alternative is the
person-specific OLS residual of the covariate on time (their Eq. 36). An
autoregressive series is related to time, so CWC of a raw AR path does not
recover process drift `a`. Traditional person-mean centering is the
horizontal line in their Figure 9; detrend uses the individual regression
line.

T=2 CWC is always `r, −r` (empty admissible). T=3 arithmetic progression
has a zero residual (empty admissible). Opposite-sign and zero residuals
are skipped. An empty admissible set fails closed.

Driver et al. (2017, Eq. 3, p. 5; JSS PDF opened 2026-09-02) write the
discrete solution `η(t) = e^{A(t−t0)} η(t0) + ⋯`. The noiseless scalar
inverse is `a = ln(later / earlier) / Δt`. Voelkle, Oud, Davidov, and
Schmidt (2012, Eq. 7) print the same exponential map; the ZORA PDF was
not re-opened this cycle (invalid cross-reference table) and is cited
only as previously opened lineage.

No claim is made for matrix `expm`, Kalman filtering, ESEM, DSEM, or
ctsem estimation.

## Recovery evidence

Already-centered irregular pairs recover known `a` at machine precision.
CWC of a raw AR path with a stable between-unit offset does **not** recover
that `a`. Fail-closed cases cover empty and singleton-only rows, fewer than
two lag-contributing units, non-positive intervals, non-finite scores,
non-representable means, overflowing CWC residuals after a finite mean,
tiny intervals with huge log-ratios, underflowed nonzero individual rates,
underflowed nonzero final mean rates, and the Curran refusal.

Exact zero CWC residuals also have one public identity. IEEE 754 binary64
has distinct `+0.0` and `-0.0` encodings, and subtraction from a canonical
`+0.0` unit mean can therefore leave `-0.0` when an observed score is
signed zero or otherwise equals that mean. That sign bit does not represent
positive versus negative within-person change: the person-mean deviation is
exactly zero. The public CWC lag boundary therefore canonicalizes only
validated exact-zero residuals to `+0.0`; private numerical intermediates
and caller-constructed already-centered pairs remain free to retain signed
zero. IEEE Std 754-2019 remains the active published floating-point
standard while IEEE P754 is the active revision project as of 2026-09-03.
This is the same public-identity contract as
`decompose_within_between`; it is not a license to treat CWC residuals as
raw-process drift (Curran & Bauer, 2011, Eq. 36).

Signed-zero traceability:

- RED `crates/longitudinal_core/tests/cwc_signed_zero_contract.rs` drives
  `center_within_unit_event_lags` with a `-0.0` score equal to the unit
  mean so the public later/earlier residual pair cannot keep `-0.0`.
- Causal repair `f98ee093f6f7fd318ad6623ab44313385195f956` canonicalizes an exact-zero CWC residual only
  after finite-result validation.
- Standard authority — IEEE. (2019). *IEEE standard for floating-point
  arithmetic* (IEEE Std 754-2019). IEEE. The canonical repository register
  is `docs/research/standards-and-literature.md`.

The current public numerical regressions include same-sign raw-sum overflow,
full-exponent mixed-sign cancellation, minimum-subnormal cancellation,
halfway ties-to-even for same-sign means, the mixed-sign `-31u/3` case, and
nonzero irregular-rate means that are not representable in binary64. Hosted
exact-head CI and independent review remain delivery gates; these source
contracts do not by themselves establish release readiness.

## Traceability

Curran, P. J., & Bauer, D. J. (2011). The disaggregation of within-person
and between-person effects in longitudinal models of change. *Annual Review
of Psychology, 62*, 583–619. https://doi.org/10.1146/annurev.psych.093008.100356

Driver, C. C., Oud, J. H. L., & Voelkle, M. C. (2017). Continuous time
structural equation modeling with R package ctsem. *Journal of Statistical
Software, 77*(5), 1–35. https://doi.org/10.18637/jss.v077.i05

Voelkle, M. C., Oud, J. H. L., Davidov, E., & Schmidt, P. (2012). An SEM
approach to continuous time modeling of panel data: Relating
authoritarianism and anomia. *Psychological Methods, 17*(2), 176–192.
https://doi.org/10.1037/a0027543

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std
754-2019). IEEE.
