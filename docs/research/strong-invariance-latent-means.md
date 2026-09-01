# Strong-invariance gate for two-group OLS latent means

## Scope

Adds a two-group OLS classification of configural / metric / strong / strict status and recovers \((\bar y_c-\bar y_r)/\lambda\) only when strong or strict holds. The boolean `compare_latent_means` helper is unchanged.

This slice does **not** import the unpublished `measurement_invariance` crate on `#84`. Claim-boundary tests use that crate's wire names (`configural`, `metric`, `scalar`) as documented strings only. Local `strict` is Meredith-style residual invariance; `#84` has no such wire name, so `as_measurement_invariance_wire_name` returns `None` for `Strict` and `as_str` keeps `"strict"`.

## Claim boundary

- `#84` `metric` licenses shared **metric** meaning. It does **not** license latent means.
- `#84` `scalar` is the strong/scalar status (equal loading and intercept). This implementation permits latent-mean recovery at that status.
- Strict additionally requires equal residual variance and also permits latent-mean recovery.
- This document does **not** claim from primary evidence that residual/strict invariance is universally unnecessary for every latent-mean comparison. The local gate simply does not add residual equality as a further requirement once its strong/scalar loading-and-intercept contract is satisfied. That narrower implementation policy is grounded in the factor-mean model and strong/scalar boundary described below; the stronger “residual invariance is not required” wording is treated only as secondary-literature context until a primary source is verified directly.
- Two-observation series have no residual degrees of freedom. OLS residual variance is then identically `0` and is not an estimated residual. Those series cap at strong/scalar; the local OLS implementation may recover the mean difference only after its loading/intercept equality conditions pass.
- This is two-group OLS, not MGCFA, not partial invariance, and not alignment optimization.
- The weak/strong/strict labels remain conventional labels here. Meredith (1993) is the primary source for the hierarchy; its Cambridge Core original-paper page and abstract were opened, but its full-text PDF was not. Unpaywall, OpenAlex, Semantic Scholar, archive.org, and Springer `content/pdf` were re-tried 2026-09-01T10:22Z and remain closed (`is_oa: false`, 0 locations; Springer historically returns an HTML stub). No claim in this document relies on text from an unread Meredith full paper.

## Authoritative sources used for the mean gate

Meredith, W. (1993). Measurement invariance, factor analysis and factorial invariance. *Psychometrika, 58*(4), 525–543. https://doi.org/10.1007/BF02294825

The original-paper record and abstract were opened on the Cambridge Core page on 2026-08-21. Meredith defines weak measurement invariance, strong factorial invariance, and strict factorial invariance and relates factorial invariance to group differences. This is the primary source for the hierarchy used by this gate; the implementation deliberately reports the narrower local `strong`/`strict` labels rather than claiming a full multiple-group CFA. Because the full paper was not opened, this document does not attribute a stronger residual-invariance exception to Meredith.

Sörbom, D. (1974). A general method for studying differences in factor means and factor structure between groups. *British Journal of Mathematical and Statistical Psychology, 27*(2), 229–239. https://doi.org/10.1111/j.2044-8317.1974.tb00543.x

The original article record and abstract were opened on the Wiley Online Library page on 2026-08-21. Sörbom's primary model estimates factor means, loadings, and unique variances jointly from group observed means, variances, and covariances while allowing factorial-invariance constraints. It is the direct source for the factor-mean comparison target. The formula implemented here is the scalar two-group OLS reduction obtained by subtracting the observed-mean equation under equal loading and intercept; the source is not being presented as stating this crate-specific OLS formula.

Steenkamp, J.-B. E. M., & Baumgartner, H. (1998). Assessing measurement invariance in cross-national consumer research. *Journal of Consumer Research, 25*(1), 78–90. https://doi.org/10.1086/209528

The Oxford Academic article page and abstract were opened 2026-08-20. Steenkamp and Baumgartner connect sequential measurement-invariance requirements to when comparisons of construct means are meaningful and illustrate the procedure with multisample factor models. This is a primary source for the comparison-purpose boundary used here; the implementation remains a narrower two-group OLS contract.

Baumgartner, H., & Steenkamp, J.-B. E. M. (1998). Multi-group latent variable models for varying numbers of items and factors with cross-national and longitudinal applications. *Marketing Letters, 9*, 21–35. https://doi.org/10.1023/A:1007911903032

The Springer Nature article page and abstract were opened 2026-08-20. Its simulation and empirical study concerns estimates of differences between latent means. In this repository, subtracting the two-group model \(y=ν+λ f+e\) under equal loading and intercept gives \((\bar y_c-\bar y_r)/\lambda\); that algebra is an explicit derivation of this OLS slice, not a claim that the source states the same implementation formula.

## Secondary context, not primary authority for the exception

Putnick, D. L., & Bornstein, M. H. (2016). Measurement invariance conventions and reporting: The state of the art and future directions for psychological research. *Developmental Review, 41*, 71–90. https://doi.org/10.1016/j.dr.2016.06.004

PMC author manuscript (PMC5145197) opened 2026-08-19T22:15Z. Putnick and Bornstein summarize scalar invariance as the point after which latent-factor means are commonly compared and state that residual invariance is not a prerequisite, citing Vandenberg and Lance (2000). That statement is useful secondary context but is **not** used here as primary evidence for a universal residual-invariance exception because the cited primary/underlying source was not verified directly in this run.

Vandenberg, R. J., & Lance, C. E. (2000). A review and synthesis of the measurement invariance literature: Suggestions, practices, and recommendations for organizational research. *Organizational Research Methods, 3*(1), 4–70. https://doi.org/10.1177/109442810031002

This source remains unread in the current evidence set and therefore does not establish a scientific claim in this document.

Opened sources that constrain the surrounding longitudinal/invariance stance:

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling. *Structural Equation Modeling: A Multidisciplinary Journal, 16*(3), 397–438. https://doi.org/10.1080/10705510903008204

Hamaker, E. L., Kuiper, R. M., & Grasman, R. P. P. P. (2015). A critique of the cross-lagged panel model. *Psychological Methods, 20*(1), 102–116. https://doi.org/10.1037/a0038889

## Formula notes

Per group, \(y=\nu+\lambda f+e\) is fit by OLS. Status is:

- configural when \(|\lambda_r-\lambda_c|\) exceeds tolerance;
- metric when loadings match and intercepts differ;
- strong when loadings and intercepts match and residual variances differ, or when residual degrees of freedom are absent;
- strict when both groups have residual degrees of freedom and loadings, intercepts, and residual variances match.

The latent-mean difference is \((\bar y_c-\bar y_r)/\lambda\) with \(\lambda\) the midpoint of the two loadings, and the local implementation evaluates it only after strong or strict. Meredith (1993) supplies the invariance hierarchy, Sörbom (1974) supplies the factor-mean comparison model, and Steenkamp and Baumgartner (1998) provide primary comparison-purpose context. The displayed expression is the explicitly stated scalar OLS derivation for this implementation.

The formula follows by subtracting the group means of \(y=\nu+\lambda f+e\) after the equal-loading/equal-intercept restrictions have been accepted. This implementation policy does not assert that residual equality is scientifically irrelevant in every design; any broader exception requires separately verified primary evidence.

## Verification

- strong/strict series recover a known mean difference with computed RMSE;
- metric-only (equal loading, shifted intercept) and configural series return `StrongInvarianceRequired`;
- two-observation series with matching loading and intercept classify as strong, not strict, and exercise the local strong-gate recovery path;
- `#84` wire-name tests: `metric` licenses shared metric meaning and refuses means; `scalar` maps to the local strong gate; local `strict` is not a `#84` wire name.
