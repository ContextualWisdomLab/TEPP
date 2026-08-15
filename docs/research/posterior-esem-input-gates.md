# Posterior-aware ESEM/DSEM input gates

## Scope

This slice delivers the first executable ADR 0005 contract in `psychometric_core`:

1. classify each higher-order construct as reflective, formative, network, or unresolved before any ESEM/SEM interpretation;
2. refuse raw topic-proportion Pearson correlations and OLS loadings as psychometric inputs;
3. admit ALR, ILR, or logistic-normal coordinates as unconstrained structural inputs while reserving orthonormal Aitchison-distance claims for ILR;
4. recover a reflective loading point estimate by ordinary least squares on a CPU `f64` path;
5. average recovered loading point estimates across posterior indicator draws without claiming Rubin within/between uncertainty pooling;
6. refuse latent-mean comparison without invariance evidence;
7. refuse causal language that rests only on temporal precedence, document linkage, event tracking, or model prediction.

Full ESEM/set-ESEM, formative composites, DSEM, and continuous-time dynamics remain accepted-target.

## Authoritative sources

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling. *Structural Equation Modeling: A Multidisciplinary Journal, 16*(3), 397–438. https://doi.org/10.1080/10705510903008204

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural equation models. *Structural Equation Modeling: A Multidisciplinary Journal, 25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Bollen, K., & Lennox, R. (1991). Conventional wisdom on measurement: A structural equation perspective. *Psychological Bulletin, 110*(2), 305–314. https://doi.org/10.1037/0033-2909.110.2.305

Mislevy, R. J. (1991). Randomization-based inference about latent variables from complex samples. *Psychometrika, 56*(2), 177–196. https://doi.org/10.1007/BF02294457

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
