# Logistic-normal topic coordinates

## Scope

This note doctors the first `topic_measurement` production slice (ADR 0012):

1. raw topic proportions are not Euclidean indicators;
2. additive log-ratio coordinates implement the logistic-normal map used by correlated topic models;
3. inverse ALR recovers a known simplex with a computed RMSE;
4. TF-IDF, BM25, and keyword scores are refused as inferential coordinates.

The temporal STM backend, global topic identity, method-effect model, and K-selection remain accepted-target. No database migration is allocated.

## Authoritative sources

Aitchison, J., & Shen, S. M. (1980). Logistic-normal distributions: Some properties and uses. *Biometrika, 67*(2), 261–272. https://doi.org/10.1093/biomet/67.2.261

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Blei, D. M., & Lafferty, J. D. (2007). A correlated topic model of Science. *The Annals of Applied Statistics, 1*(1), 17–35. https://doi.org/10.1214/07-AOAS114

## Application

Aitchison and Shen (1980) define the logistic-normal family via the additive log-ratio map; Aitchison (1982) is the compositional-data authority that forbids treating parts of a whole as unconstrained Euclidean coordinates. Blei and Lafferty (2007) use that same map for correlated topic models. TEPP therefore converts a strictly positive unit simplex through `additive_log_ratio` before any Euclidean or psychometric operation, and recovers the simplex with `from_additive_log_ratio` (Aitchison & Shen, 1980; Aitchison, 1982; Blei & Lafferty, 2007).

## Verification

- closed-form simplex `(2,3,1)/6` maps to `(ln 2, ln 3)` and inverts with computed RMSE below `1e-15`;
- equal shares map to a zero ALR vector;
- zero, negative, non-unit-sum, non-finite, empty, and one-part vectors fail closed;
- `tfidf`, `bm25`, and `keyword` labels are refused.
