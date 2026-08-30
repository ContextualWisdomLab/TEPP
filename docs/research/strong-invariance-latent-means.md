# Strong-invariance gate for two-group OLS latent means

## Scope

Adds a two-group OLS classification of configural / metric / strong / strict status and recovers \((\bar y_c-\bar y_r)/\lambda\) only when strong or strict holds. The boolean `compare_latent_means` helper is unchanged.

This slice does **not** import the unpublished `measurement_invariance` crate on `#84`. Claim-boundary tests use that crate's wire names (`configural`, `metric`, `scalar`) as documented strings only. Local `strict` is Meredith-style residual invariance; `#84` has no such wire name, so `as_measurement_invariance_wire_name` returns `None` for `Strict` and `as_str` keeps `"strict"`.

## Claim boundary

- `#84` `metric` licenses shared **metric** meaning. It does **not** license latent means.
- `#84` `scalar` is the strong/scalar status (equal loading and intercept). That status licenses latent means.
- Strict (also equal residual variance) also licenses latent means. Residual invariance is **not** required for those means.
- Two-observation series have no residual degrees of freedom. OLS residual variance is then identically `0` and is not an estimated residual. Those series cap at strong/scalar and still license means.
- This is two-group OLS, not MGCFA, not partial invariance, and not alignment optimization.
- The weak/strong/strict labels remain conventional labels here. Meredith (1993) is the primary source for the hierarchy; its Cambridge Core original-paper page and abstract were opened, but its full-text PDF was not. Unpaywall, OpenAlex, Semantic Scholar, and Springer `content/pdf` were re-tried 2026-08-30T20:25Z and remain closed (`is_oa: false`, 0 locations, `best_oa_location: null`; Springer returns a 3038-byte HTML stub). Putnick and Bornstein (2016) cite Meredith for residual invariance as part of *full factorial invariance*; that specific claim is not a reading of Meredith's full text.

## Authoritative sources used for the mean gate

Meredith, W. (1993). Measurement invariance, factor analysis and factorial invariance. *Psychometrika, 58*(4), 525–543. https://doi.org/10.1007/BF02294825

The original-paper record and abstract were opened on the Cambridge Core page on 2026-08-21. Meredith defines weak measurement invariance, strong factorial invariance, and strict factorial invariance and relates factorial invariance to group differences. This is the primary source for the hierarchy used by this gate; the implementation deliberately reports the narrower local `strong`/`strict` labels rather than claiming a full multiple-group CFA.

Sörbom, D. (1974). A general method for studying differences in factor means and factor structure between groups. *British Journal of Mathematical and Statistical Psychology, 27*(2), 229–239. https://doi.org/10.1111/j.2044-8317.1974.tb00543.x

The original article record and abstract were opened on the Wiley Online Library page on 2026-08-21. Sörbom's primary model estimates factor means, loadings, and unique variances jointly from group observed means, variances, and covariances while allowing factorial-invariance constraints. It is the direct source for the factor-mean comparison target. The formula implemented here is the scalar two-group OLS reduction obtained by subtracting the observed-mean equation under equal loading and intercept; the source is not being presented as stating this crate-specific OLS formula.

Putnick, D. L., & Bornstein, M. H. (2016). Measurement invariance conventions and reporting: The state of the art and future directions for psychological research. *Developmental Review, 41*, 71–90. https://doi.org/10.1016/j.dr.2016.06.004

PMC author manuscript (PMC5145197) opened 2026-08-19T22:15Z from https://pmc.ncbi.nlm.nih.gov/articles/PMC5145197/. The NIHMS PDF endpoints returned HTML/500 on this cycle; the PMC HTML full text is the opened copy.

Putnick and Bornstein write that measurement invariance is a prerequisite to comparing group means. Metric invariance is equivalence of item loadings: each item contributes to the latent construct to a similar degree across groups. Scalar invariance is equivalence of item intercepts after metric: “mean differences in the latent construct capture all mean differences in the shared variance of the items.” After those steps, “the researcher is free to compare group means on the latent factors.” Residual invariance “is not a prerequisite for testing mean differences because the residuals are not part of the latent factor” (they cite Vandenberg & Lance, 2000, unread). Configural, metric, and scalar “are required prior to group mean comparisons.” This crate’s `#84` `metric` / `scalar` split follows that terminology. The executable map remains two-group OLS, not their multiple-group CFA.

Steenkamp, J.-B. E. M., & Baumgartner, H. (1998). Assessing measurement invariance in cross-national consumer research. *Journal of Consumer Research, 25*(1), 78–90. https://doi.org/10.1086/209528

The Oxford Academic article page and abstract were opened 2026-08-20. Steenkamp and Baumgartner connect sequential measurement-invariance requirements to when comparisons of construct means are meaningful and illustrate the procedure with multisample factor models. This is the primary source for the gate's comparison-purpose boundary; the implementation remains a narrower two-group OLS contract.

Baumgartner, H., & Steenkamp, J.-B. E. M. (1998). Multi-group latent variable models for varying numbers of items and factors with cross-national and longitudinal applications. *Marketing Letters, 9*, 21–35. https://doi.org/10.1023/A:1007911903032

The Springer Nature article page and abstract were opened 2026-08-20. Its simulation and empirical study concerns estimates of differences between latent means. In this repository, subtracting the two-group model \(y=ν+λ f+e\) under equal loading and intercept gives \((\bar y_c-\bar y_r)/\lambda\); that algebra is an explicit derivation of this OLS slice, not a claim that the source states the same implementation formula.

Opened sources that constrain the surrounding longitudinal/invariance stance:

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling. *Structural Equation Modeling: A Multidisciplinary Journal, 16*(3), 397–438. https://doi.org/10.1080/10705510903008204

Hamaker, E. L., Kuiper, R. M., & Grasman, R. P. P. P. (2015). A critique of the cross-lagged panel model. *Psychological Methods, 20*(1), 102–116. https://doi.org/10.1037/a0038889

Vandenberg, R. J., & Lance, C. E. (2000). A review and synthesis of the measurement invariance literature: Suggestions, practices, and recommendations for organizational research. *Organizational Research Methods, 3*(1), 4–70. https://doi.org/10.1177/109442810031002 (cited by Putnick & Bornstein, 2016, for residual invariance not being required for latent means; PDF not opened).

## Formula notes

Per group, \(y=\nu+\lambda f+e\) is fit by OLS. Status is:

- configural when \(|\lambda_r-\lambda_c|\) exceeds tolerance;
- metric when loadings match and intercepts differ;
- strong when loadings and intercepts match and residual variances differ, or when residual degrees of freedom are absent;
- strict when both groups have residual degrees of freedom and loadings, intercepts, and residual variances match.

The latent-mean difference is \((\bar y_c-\bar y_r)/\lambda\) with \(\lambda\) the midpoint of the two loadings, and only after strong or strict. Meredith (1993) supplies the invariance hierarchy and Sörbom (1974) supplies the factor-mean comparison model; the displayed expression is the explicitly stated scalar OLS derivation for this implementation.

The formula follows by subtracting the group means of \(y=\nu+\lambda f+e\) after the equal-loading/equal-intercept restrictions have been accepted; the cited multi-group latent-mean study supplies the comparison target, while this document records the narrower OLS derivation.

## Verification

- strong/strict series recover a known mean difference with computed RMSE;
- metric-only (equal loading, shifted intercept) and configural series return `StrongInvarianceRequired`;
- two-observation series with matching loading and intercept classify as strong, not strict, and still recover the known mean difference;
- `#84` wire-name tests: `metric` licenses shared metric meaning and refuses means; `scalar` is strong and licenses means; local `strict` is not a `#84` wire name.
