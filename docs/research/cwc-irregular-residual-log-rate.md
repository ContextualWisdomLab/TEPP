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
   Singleton units are skipped.
2. `recover_within_unit_irregular_residual_log_rate` is the pairwise mean of
   the Driver, Oud, and Voelkle (2017, Eq. 3) scalar inverse
   `a = ln(|later| / |earlier|) / Δt` on nonzero same-sign residuals. When
   the absolute residual ratio is finite and positive the finite-ratio
   logarithm is used; overflowed or underflowed ratios fall back to
   `ln|later| − ln|earlier|`. The pairwise mean is incremental so two finite
   rates whose raw sum overflows stay representable.
3. `recover_centered_irregular_residual_log_rate` is the already-centered
   path. It does **not** re-center. The signed residual ratio must be
   strictly positive. Known-truth pairs `(1, 0.5)` over unit event time
   recover `ln(0.5)`.
4. `refuse_cwc_residual_log_rate_as_raw_process_drift` always fails closed.

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

Already-centered irregular pairs recover known `a` at machine precision,
including `ln(0.5)`. CWC of a raw AR path with a stable between-unit
offset does **not** recover that `a`. Fail-closed cases cover empty and
singleton-only rows, one unit, non-positive intervals, non-finite scores,
overflowed unit means, overflowing CWC residuals after a finite mean,
tiny intervals with huge log-ratios, and the Curran refusal.

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
