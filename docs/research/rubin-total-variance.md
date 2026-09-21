# Rubin total variance on draw-level OLS loadings

## Scope

Adds Rubin combining for complete-data OLS loadings across posterior indicator draws. The existing arithmetic-mean helper remains a point-estimate summary and still must not be described as Rubin pooling.

## Claim boundary

`T_m = \bar U_m + (1+1/m)B_m` is the complete-data combining rule. This slice does **not** implement Mislevy plausible values. The 1991 *Psychometrika* paper was not opened in this cycle (Unpaywall/OpenAlex 2026-09-01T10:22Z: still closed, `is_oa: false`, 0 OA locations; ETS landing page is HTML; ERIC ED334221 is Singer & Willett, 1991). Do not cite that paper as having been read.

## Authoritative sources

Rubin, D. B. (1996). Multiple imputation after 18+ years. *Journal of the American Statistical Association, 91*(434), 473–489. https://doi.org/10.1080/01621459.1996.10476908

The scanned page 473 of the reviewed Rubin (1996) article restates \(T_m=\bar U_m+(1+1/m)B_m\). The 1987 book was not opened; this document's authority for the combining rule is the reviewed Rubin (1996) source above, and Rubin (1987) is named only as the origin that source points to.

## Formula notes

For \(m\ge 2\) complete-data OLS loadings \(\hat Q_\ell\) with sampling variances \(U_\ell=\hat\sigma^{2}_\ell/\sum(f-\bar f)^{2}\):

- \(\bar Q_m=m^{-1}\sum\hat Q_\ell\)
- \(\bar U_m=m^{-1}\sum U_\ell\)
- \(B_m=(m-1)^{-1}\sum(\hat Q_\ell-\bar Q_m)^{2}\)
- \(T_m=\bar U_m+(1+1/m)B_m\)

Two-point lines have residual variance 0 and therefore \(U_\ell=0\). Symmetric draw noise can still produce \(B_m>0\).

## Verification

- the combined mean recovers a known loading with machine-scale computed RMSE and matches the point-estimate mean;
- reported \(T\) equals \(\bar U+(1+1/m)B\) on the same draws;
- raw-proportion, singleton-draw, and singular designs fail closed.
