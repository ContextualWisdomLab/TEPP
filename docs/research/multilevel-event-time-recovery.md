# Multilevel cluster-mean OLS and event-time log-rate

## Scope

This slice stays inside `psychometric_core`. It does not add a second invariance crate and does not recreate `#78` `longitudinal_core` or `#80` `irregular_time`.

1. recover within-cluster and between-cluster OLS slopes after centering within cluster (CWC);
2. recover the CWC contextual effect as `between − within` (Enders & Tofighi, 2007, Table 2);
3. recover a Kish-weighted least-squares slope and report Kish ESS as the information diagnostic;
4. map a discrete lag-1 coefficient through the exact scalar exponential on **event time only**;
5. refuse the difference quotient as a continuous-time rate;
6. apply the same event-time map to CWC residuals (still not DSEM);
7. map already-centered lagged residuals with irregular event intervals without re-centering (Curran & Bauer, 2011, pp. 607–608).

## Claim boundary

This is two-level OLS and a noiseless scalar continuous-time map. It is not DSEM, not RI-CLPM, not a random-effects sampler, and not a matrix `expm` implementation. The CWC cluster-mean coefficient is the **contextual** effect, not the between-cluster effect.

## Authoritative sources

Enders, C. K., & Tofighi, D. (2007). Centering predictor variables in cross-sectional multilevel models: A new look at an old issue. *Psychological Methods, 12*(2), 121–138. https://doi.org/10.1037/1082-989X.12.2.121

Curran, P. J., & Bauer, D. J. (2011). The disaggregation of within-person and between-person effects in longitudinal models of change. *Annual Review of Psychology, 62*, 583–619. https://doi.org/10.1146/annurev.psych.093008.100356

Hamaker, E. L., Kuiper, R. M., & Grasman, R. P. P. P. (2015). A critique of the cross-lagged panel model. *Psychological Methods, 20*(1), 102–116. https://doi.org/10.1037/a0038889

Voelkle, M. C., Oud, J. H. L., Davidov, E., & Schmidt, P. (2012). An SEM approach to continuous time modeling of panel data: Relating authoritarianism and anomia. *Psychological Methods, 17*(2), 176–192. https://doi.org/10.1037/a0027543

Driver, C. C., Oud, J. H. L., & Voelkle, M. C. (2017). Continuous time structural equation modeling with R package ctsem. *Journal of Statistical Software, 77*(5), 1–35. https://doi.org/10.18637/jss.v077.i05

Kish, L. (1965). *Survey sampling*. John Wiley & Sons.

## Formula notes

- **CWC.** For cluster \(i\) and occasion \(t\), \(x_{it}^{w} = x_{it}-\bar x_{i}\) and \(y_{it}^{w} = y_{it}-\bar y_{i}\). The within slope is OLS of \(y^{w}\) on \(x^{w}\). The between slope is OLS of the cluster means. A grand-mean pooled slope confounds the two.
- **Contextual effect.** Enders and Tofighi (2007, Table 2, pp. 124–127): under CWC, the cluster-mean coefficient \(\gamma_{01}\) is the contextual effect (expected difference between two people with the same individual \(X\) from groups one unit apart on \(\bar X\)). Under CGM the same symbol is the between-cluster effect. The OLS identity is \(\gamma_{01}^{\mathrm{CWC}}=\beta_{\mathrm{between}}-\beta_{\mathrm{within}}\). Adding the CWC contextual coefficient to the within slope recovers the between-cluster slope. This crate reports the OLS analogue; it does not estimate their multilevel maximum-likelihood model.
- **Kish ESS.** \(\mathrm{ESS}=(\sum w)^{2}/\sum w^{2}\) on non-negative finite weights. WLS uses the weights in the slope; ESS is not a second slope.
- **Exact scalar map.** Voelkle et al. (2012, Eq. 7) and Driver et al. (2017, Eq. 3): \(\varphi = A^{*}(\Delta t)=\exp(a\,\Delta t)\). The inverse is \(a=\ln\varphi/\Delta t\). The difference quotient \((x(t+\Delta t)-x(t))/\Delta t\) is refused.
- **CWC-then-lag.** Sample cluster means are removed first. Consecutive within residuals are then fitted by least squares to \(r_{t+\Delta t}\approx\exp(a\Delta t)\,r_{t}\) on event time. Same-sign pair-wise logs initialize the scalar Newton step. Sign-flipping \(T=2\) CWC pairs have no real logarithm and fail closed. Curran and Bauer (2011, pp. 607–608) show that this person-mean subtraction on a raw autoregressive series does **not** isolate the lagged within-person effect; the helper therefore does not claim to recover the raw-process drift.
- **Already-centered irregular residual.** The caller supplies lagged within residuals. The mean of \(a=\ln(r_{t+\Delta t}/r_t)/\Delta t\) is the exact scalar map. Intervals may be irregular. The helper does not center again. This is not DSEM.

## Verification

- noiseless CWC recovers known within, between, and contextual slopes with smaller computed RMSE than a pooled OLS collapse;
- the CWC contextual effect is not equal to the between-cluster slope when the within slope is nonzero, and contextual + within recovers between;
- Kish WLS recovers a known slope;
- the exact scalar map recovers a known drift on event time and refuses every other clock plus the difference quotient;
- CWC-then-lag on a two-cluster decaying series has smaller computed RMSE than a level-pooled series when the latter is identified;
- already-centered irregular residuals recover a known drift at machine-scale RMSE, and that RMSE is smaller than CWC of the corresponding raw autoregressive series (Curran & Bauer, 2011, pp. 607–608);
- a singleton cluster is skipped; two singleton clusters yield an empty pair list and fail closed;
- overflowing CWC residuals, overflowing contextual subtraction, and Newton overflow / flat-derivative steps fail closed.
