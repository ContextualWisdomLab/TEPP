# Posterior-aware ESEM/DSEM input gates

## Scope

This slice delivers the first executable ADR 0005 contract in `psychometric_core`:

1. classify each higher-order construct as reflective, formative, network, or unresolved before any ESEM/SEM interpretation;
2. refuse raw topic-proportion Pearson correlations and OLS loadings as psychometric inputs;
3. admit ALR, ILR, or logistic-normal coordinates as unconstrained structural inputs while reserving orthonormal Aitchison-distance claims for ILR;
4. recover a reflective loading point estimate by ordinary least squares on a CPU `f64` path;
5. average recovered loading point estimates across posterior indicator draws without claiming Rubin within/between uncertainty pooling (the Rubin `T` helper is a separate API; see `docs/research/rubin-total-variance.md`);
6. refuse latent-mean comparison unless typed invariance evidence carries a strong or strict two-group OLS status (`LatentMeanComparisonEvidence`; metric/configural evidence cannot reduce to a passing flag), and recover a mean difference only under that strong or strict status (Putnick & Bornstein, 2016: scalar licenses means; residual invariance is not required; two-observation series cap at strong because residual variance is identically `0`);
7. refuse causal language that rests only on temporal precedence, document linkage, event tracking, or model prediction.

Cluster-mean CWC, the CWC contextual effect, Kish WLS, event-time log-rate, CWC-then-lag, irregular already-centered residual log-rate, occasion-mean residual log-rate (Hamaker et al., 2015, Eq. 1a; not within-person; not RI-CLPM), the Driver Eq. 5 of the Eq. 3 evolved mean, the Driver Eq. 3 contemporaneous `TDPREDEFFECT` impulse, the Driver Eq. 5 of that contemporaneous impulse, the Driver Eq. 1–2 within-interval impulse carry, the Driver Eq. 5 of that carried latent mean, the Driver Eq. 3 `TIPREDEFFECT` increment, and the Driver Eq. 5 of that increment live in the same crate and are documented in `docs/research/multilevel-event-time-recovery.md`. Full ESEM/set-ESEM, formative composites, DSEM, and matrix continuous-time dynamics remain accepted-target.

## Authoritative sources

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling. *Structural Equation Modeling: A Multidisciplinary Journal, 16*(3), 397–438. https://doi.org/10.1080/10705510903008204

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural equation models. *Structural Equation Modeling: A Multidisciplinary Journal, 25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Bollen, K., & Lennox, R. (1991). Conventional wisdom on measurement: A structural equation perspective. *Psychological Bulletin, 110*(2), 305–314. https://doi.org/10.1037/0033-2909.110.2.305

Driver, C. C., Oud, J. H. L., & Voelkle, M. C. (2017). Continuous time structural equation modeling with R package ctsem. *Journal of Statistical Software, 77*(5), 1–35. https://doi.org/10.18637/jss.v077.i05

Hamaker, E. L., Kuiper, R. M., & Grasman, R. P. P. P. (2015). A critique of the cross-lagged panel model. *Psychological Methods, 20*(1), 102–116. https://doi.org/10.1037/a0038889

Mislevy, R. J. (1991). Randomization-based inference about latent variables from complex samples. *Psychometrika, 56*(2), 177–196. https://doi.org/10.1007/BF02294457

Putnick, D. L., & Bornstein, M. H. (2016). Measurement invariance conventions and reporting: The state of the art and future directions for psychological research. *Developmental Review, 41*, 71–90. https://doi.org/10.1016/j.dr.2016.06.003

Holland, P. W. (1986). Statistics and causal inference. *Journal of the American Statistical Association, 81*(396), 945–960. https://doi.org/10.1080/01621459.1986.10478354

## Formula notes

- **OLS loading** \(\hat\lambda = \sum_i (f_i-\bar f)(y_i-\bar y) / \sum_i (f_i-\bar f)^2\) on already-mapped coordinates.
- **Posterior-draw loading point estimate** is the arithmetic mean of \(\hat\lambda_d\) across draws. This narrow slice does not compute within-draw variance, between-draw variance, total variance, degrees of freedom, or Rubin-style pooled uncertainty; Mislevy (1991) motivates the future full posterior-propagation contract rather than validating this point-estimate shortcut.
- **RMSE** is computed from recovered versus known true loadings; tests do not hard-code expected recovery numbers.
- A good global fit statistic is not authority to reinterpret a formative or network construct as reflective (Bollen & Lennox, 1991; Asparouhov & Muthén, 2009).

## Verification

- noiseless OLS recovers a known loading with machine-scale computed RMSE;
- symmetric posterior-draw point-estimate noise cancels in the arithmetic mean and has smaller computed RMSE than a single draw;
- raw-proportion, empty, non-finite, singular, invariance-missing, formative-reinterpretation, and causal-heuristic paths fail closed.
