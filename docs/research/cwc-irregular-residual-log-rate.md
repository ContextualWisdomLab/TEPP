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
tiny intervals with huge log-ratios, underflowed nonzero final rates, and the
Curran refusal.

The current public numerical regressions include same-sign raw-sum overflow,
full-exponent mixed-sign cancellation, minimum-subnormal cancellation,
halfway ties-to-even for same-sign means, and the mixed-sign `-31u/3` case
above. Hosted exact-head CI and independent review remain delivery gates; these
source contracts do not by themselves establish release readiness.

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
