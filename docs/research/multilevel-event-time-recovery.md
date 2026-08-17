# Multilevel cluster-mean OLS and event-time log-rate

## Scope

This slice stays inside `psychometric_core`. It does not add a second invariance crate and does not recreate `#78` `longitudinal_core` or `#80` `irregular_time`.

1. recover within-cluster and between-cluster OLS slopes after centering within cluster (CWC);
2. recover the CWC contextual effect as `between − within` (Enders & Tofighi, 2007, Table 2);
3. recover a Kish-weighted least-squares slope and report Kish ESS as the information diagnostic;
4. map a discrete lag-1 coefficient through the exact scalar exponential on **event time only**;
5. recover the exact scalar forward map `φ(Δt) = exp(a Δt)` and remap a discrete lag onto another event interval through that log-rate;
6. refuse a binary64 underflow of that forward map to `+0` (not a discrete lag);
7. recover the exact scalar discrete effect of a constant predictor (Voelkle et al., 2012, Eq. 12);
8. refuse pooling discrete lags from unequal event intervals as one coefficient;
9. refuse the difference quotient as a continuous-time rate;
10. apply the same event-time map to CWC residuals (still not DSEM);
11. map already-centered lagged residuals with irregular event intervals without re-centering (Curran & Bauer, 2011, pp. 607–608).

## Claim boundary

This is two-level OLS and a noiseless scalar continuous-time map. It is not DSEM, not RI-CLPM, not a random-effects sampler, and not a matrix `expm` implementation. The CWC cluster-mean coefficient is the **contextual** effect, not the between-cluster effect. Discrete lags from different event intervals are not one coefficient.

## Authoritative sources

Enders, C. K., & Tofighi, D. (2007). Centering predictor variables in cross-sectional multilevel models: A new look at an old issue. *Psychological Methods, 12*(2), 121–138. https://doi.org/10.1037/1082-989X.12.2.121

Curran, P. J., & Bauer, D. J. (2011). The disaggregation of within-person and between-person effects in longitudinal models of change. *Annual Review of Psychology, 62*, 583–619. https://doi.org/10.1146/annurev.psych.093008.100356

Hamaker, E. L., Kuiper, R. M., & Grasman, R. P. P. P. (2015). A critique of the cross-lagged panel model. *Psychological Methods, 20*(1), 102–116. https://doi.org/10.1037/a0038889

Voelkle, M. C., Oud, J. H. L., Davidov, E., & Schmidt, P. (2012). An SEM approach to continuous time modeling of panel data: Relating authoritarianism and anomia. *Psychological Methods, 17*(2), 176–192. https://doi.org/10.1037/a0027543

Driver, C. C., Oud, J. H. L., & Voelkle, M. C. (2017). Continuous time structural equation modeling with R package ctsem. *Journal of Statistical Software, 77*(5), 1–35. https://doi.org/10.18637/jss.v077.i05

Kish, L. (1965). *Survey sampling*. John Wiley & Sons.

The Voelkle et al. (2012) ZORA accepted manuscript was opened 2026-08-17 (https://www.zora.uzh.ch/handle/20.500.14742/72792). This cycle the ZORA bitstream was Anubis-blocked (2026-08-17T12:03Z). Driver, Oud, and Voelkle (2017, Eq. 3 and p. 4; JSS PDF re-opened 2026-08-17T12:04Z) write \(A_{\Delta t}=\operatorname{expm}(A\Delta t)\) and restate the discrete intercept as a function of \(A\) and \(\Delta t\). Discrete auto-effects are strictly positive for finite real drift and interval; the inverse \(a=\ln\varphi/\Delta t\) therefore requires \(\varphi>0\). Binary64 `exp` of a large negative argument is `+0` and is refused. Equations 3–4 of Voelkle et al. (2012) are the discouraged difference-quotient approximation. Meredith (1993) remains unread (Unpaywall/OpenAlex/Semantic Scholar/Cambridge Core/Springer 2026-08-17T12:04Z: closed). Mislevy (1991) remains unread. The 1988 ETS RR-88-45 / DTIC ADA200179 technical report was opened 2026-08-17T12:04Z and is not the 1991 journal article.

## Formula notes

- **CWC.** For cluster \(i\) and occasion \(t\), \(x_{it}^{w} = x_{it}-\bar x_{i}\) and \(y_{it}^{w} = y_{it}-\bar y_{i}\). The within slope is OLS of \(y^{w}\) on \(x^{w}\). The between slope is OLS of the cluster means. A grand-mean pooled slope confounds the two.
- **Contextual effect.** Enders and Tofighi (2007, Table 2, pp. 124–127): under CWC, the cluster-mean coefficient \(\gamma_{01}\) is the contextual effect (expected difference between two people with the same individual \(X\) from groups one unit apart on \(\bar X\)). Under CGM the same symbol is the between-cluster effect. The OLS identity is \(\gamma_{01}^{\mathrm{CWC}}=\beta_{\mathrm{between}}-\beta_{\mathrm{within}}\). Adding the CWC contextual coefficient to the within slope recovers the between-cluster slope. This crate reports the OLS analogue; it does not estimate their multilevel maximum-likelihood model.
- **Kish ESS.** \(\mathrm{ESS}=(\sum w)^{2}/\sum w^{2}\) on non-negative finite weights. WLS uses the weights in the slope; ESS is not a second slope.
- **Exact scalar map.** Voelkle et al. (2012, Eq. 7) and Driver et al. (2017, Eq. 3): \(\varphi = A^{*}(\Delta t)=\exp(a\,\Delta t)\). The inverse is \(a=\ln\varphi/\Delta t\). The forward map is the same equation. The real exponential is strictly positive; a binary64 underflow to `+0` is refused because the inverse logarithm does not exist at zero. The difference quotient \((x(t+\Delta t)-x(t))/\Delta t\) is refused.
- **Unequal-interval remap.** Discrete \(\varphi(\Delta t_1)\) and \(\varphi(\Delta t_2)\) are not comparable when \(\Delta t_1\neq\Delta t_2\) (Voelkle et al., 2012, ZORA manuscript pp. 2, 16, 33). The licensed path is \(a=\ln\varphi_{\mathrm{src}}/\Delta t_{\mathrm{src}}\) then \(\varphi_{\mathrm{ref}}=\exp(a\,\Delta t_{\mathrm{ref}})\). Pooling those discrete lags fails closed.
- **Constant-predictor discrete effect.** Voelkle et al. (2012, Eq. 12; ZORA accepted manuscript p. 16): for a constant predictor with \(a_{xx}\neq 0\), \(b^{*}_{y.x}(\Delta t)=(a_{yx}/a_{xx})(\exp(a_{xx}\Delta t)-1)\). Driver, Oud, and Voelkle (2017, p. 4, after Eq. 3; PDF re-opened 2026-08-17T12:04Z) restate the discrete intercept as a function of \(A\) and \(\Delta t\). The algebraically identical evaluation is \(a_{yx}\Delta t\,(\operatorname{expm1}(z)/z)\) with \(z=a_{xx}\Delta t\). When binary64 \(z\) underflows to `+0`, the mathematical limit of Eq. 12 is \(a_{yx}\Delta t\). That limit is IEEE-754 evaluation of Eq. 12, not a substitution of the first-order product as the general discrete effect. \(a_{xx}=0\) fails closed. This is not DSEM.
- **CWC-then-lag.** Sample cluster means are removed first. Consecutive within residuals are then fitted by least squares to \(r_{t+\Delta t}\approx\exp(a\Delta t)\,r_{t}\) on event time. Same-sign pair-wise logs initialize the scalar Newton step. Sign-flipping \(T=2\) CWC pairs have no real logarithm and fail closed. Curran and Bauer (2011, pp. 607–608) show that this person-mean subtraction on a raw autoregressive series does **not** isolate the lagged within-person effect; the helper therefore does not claim to recover the raw-process drift.
- **Already-centered irregular residual.** The caller supplies lagged within residuals. The mean of \(a=\ln(r_{t+\Delta t}/r_t)/\Delta t\) is the exact scalar map. Intervals may be irregular. The helper does not center again. This is not DSEM.

## Verification

- noiseless CWC recovers known within, between, and contextual slopes with smaller computed RMSE than a pooled OLS collapse;
- the CWC contextual effect is not equal to the between-cluster slope when the within slope is nonzero, and contextual + within recovers between;
- Kish WLS recovers a known slope;
- the exact scalar map recovers a known drift on event time and refuses every other clock plus the difference quotient;
- the forward map inverts the log-rate, remaps \(\varphi(1)\) onto \(\varphi(2)\) at machine-scale RMSE, and that RMSE is smaller than treating \(\varphi(1)\) as \(\varphi(2)\);
- a binary64 underflow of \(\exp(a\Delta t)\) to `+0` (direct forward map and large-interval remap) fails closed;
- Voelkle et al. (2012, Eq. 12) recovers a known discrete constant-predictor effect at machine-scale RMSE, and that RMSE is smaller than the first-order product \(a_{yx}\Delta t\); \(a_{xx}=0\) fails closed; binary64 underflow of \(a_{xx}\Delta t\) to `+0` recovers the Eq. 12 limit \(a_{yx}\Delta t\);
- pooling discrete lags from unequal intervals fails closed;
- CWC-then-lag on a two-cluster decaying series has smaller computed RMSE than a level-pooled series when the latter is identified;
- already-centered irregular residuals recover a known drift at machine-scale RMSE, and that RMSE is smaller than CWC of the corresponding raw autoregressive series (Curran & Bauer, 2011, pp. 607–608);
- a singleton cluster is skipped; two singleton clusters yield an empty pair list and fail closed;
- overflowing CWC residuals, overflowing contextual subtraction, later-only residual overflow, non-finite intervals, Newton overflow / start-skip / deriv-INF, and Pearson empty/mismatch paths fail closed.
