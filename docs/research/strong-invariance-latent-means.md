# Strong-invariance gate for two-group OLS latent means

## Scope

Adds a two-group OLS classification of configural / metric / strong / strict status and recovers \((\bar y_c-\bar y_r)/\lambda\) only when strong or strict holds. The boolean `compare_latent_means` helper is unchanged.

This slice does **not** import the unpublished `measurement_invariance` crate on `#84`. Claim-boundary tests use that crate's wire names (`configural`, `metric`, `scalar`) as documented strings only.

## Claim boundary

- `#84` `metric` licenses shared **metric** meaning. It does **not** license latent means.
- `#84` `scalar` is the strong/scalar status (equal loading and intercept). That status licenses latent means.
- Strict (also equal residual variance) also licenses latent means.
- This is two-group OLS, not MGCFA, not partial invariance, and not alignment optimization.
- Meredith (1993) names weak/strong/strict are used only as conventional labels. That PDF was not opened (Springer remains closed; Unpaywall reported no OA location). Do not cite Meredith equations as having been read.

## Authoritative sources used for the mean gate

The executable gate follows the ADR 0005 rule that mean comparison requires the invariance level needed for that claim, and the `#84` terminology split between metric meaning and scalar/strong means. Opened sources that constrain the surrounding longitudinal/invariance stance:

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling. *Structural Equation Modeling: A Multidisciplinary Journal, 16*(3), 397–438. https://doi.org/10.1080/10705510903008204

Hamaker, E. L., Kuiper, R. M., & Grasman, R. P. P. P. (2015). A critique of the cross-lagged panel model. *Psychological Methods, 20*(1), 102–116. https://doi.org/10.1037/a0038889

## Formula notes

Per group, \(y=\nu+\lambda f+e\) is fit by OLS. Status is:

- configural when \(|\lambda_r-\lambda_c|\) exceeds tolerance;
- metric when loadings match and intercepts differ;
- strong when loadings and intercepts match and residual variances differ;
- strict when loadings, intercepts, and residual variances match.

The latent-mean difference is \((\bar y_c-\bar y_r)/\lambda\) with \(\lambda\) the midpoint of the two loadings, and only after strong or strict.

## Verification

- strong/strict series recover a known mean difference with computed RMSE;
- metric-only (equal loading, shifted intercept) and configural series return `StrongInvarianceRequired`;
- `#84` wire-name tests: `metric` licenses shared metric meaning and refuses means; `scalar` is strong and licenses means.
