# Logistic-normal topic coordinates

## Scope

This note doctors the first `topic_measurement` production slice (ADR 0012):

1. raw topic proportions are compositional rather than unconstrained Euclidean indicators;
2. additive log-ratio coordinates implement the reference-dependent logistic-normal map used by correlated topic models;
3. ALR is full rank but not an orthonormal Aitchison-distance isometry; ILR is required when that Euclidean geometry is the estimand;
4. max-shifted inverse ALR and log-difference forward ALR recover representable extreme coordinates without overflow;
5. TF-IDF, BM25, and keyword scores are refused as inferential coordinates.

The temporal STM backend, global topic identity, method-effect model, and K-selection remain accepted-target. No database migration is allocated.

## Authoritative sources

Aitchison, J., & Shen, S. M. (1980). Logistic-normal distributions: Some properties and uses. *Biometrika, 67*(2), 261–272. https://doi.org/10.1093/biomet/67.2.261

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Blei, D. M., & Lafferty, J. D. (2007). A correlated topic model of Science. *The Annals of Applied Statistics, 1*(1), 17–35. https://doi.org/10.1214/07-AOAS114

## Application

Aitchison and Shen (1980) define the logistic-normal family via the additive log-ratio map; Aitchison (1982) is the compositional-data authority that forbids treating parts of a whole as unconstrained Euclidean coordinates. Blei and Lafferty (2007) use that same reference-dependent map for correlated topic models. TEPP therefore uses `additive_log_ratio` for logistic-normal regression and psychometric interfaces, but does not claim that ALR preserves Aitchison distance. Analyses whose estimand is orthonormal Euclidean Aitchison geometry must use ILR. `from_additive_log_ratio` uses a max-shifted inverse softmax and the forward map subtracts logarithms, avoiding avoidable exponential and ratio overflow while failing closed when an `f64` simplex part would underflow to zero (Aitchison & Shen, 1980; Aitchison, 1982; Blei & Lafferty, 2007).

## Verification

- closed-form simplex `(2,3,1)/6` maps to `(ln 2, ln 3)` and inverts with computed RMSE below `1e-15`;
- representable coordinates `(710, 709)` round-trip through the max-shifted inverse without exponential overflow;
- extremes that would underflow a strictly positive `f64` simplex part fail closed;
- equal shares map to a zero ALR vector;
- zero, negative, non-unit-sum, non-finite, empty, and one-part vectors fail closed;
- `tfidf`, `bm25`, and `keyword` labels are refused.
