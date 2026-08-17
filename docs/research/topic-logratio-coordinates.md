# Logistic-normal topic coordinates

## Scope

This note doctors the first `topic_measurement` production slice (ADR 0012):

1. raw topic proportions are compositional rather than unconstrained Euclidean indicators;
2. additive log-ratio coordinates implement the reference-dependent logistic-normal map used by correlated topic models;
3. ALR is full rank but not an orthonormal Aitchison-distance isometry;
4. sequential Egozcue ILR supplies the orthonormal Aitchison-distance isometry when that Euclidean geometry is the estimand;
5. max-shifted inverses recover representable extreme coordinates without overflow;
6. TF-IDF, BM25, and keyword scores are refused as inferential coordinates.

The temporal STM backend, global topic identity, method-effect model, and K-selection remain accepted-target. No database migration is allocated.

## Authoritative sources

Aitchison, J., & Shen, S. M. (1980). Logistic-normal distributions: Some properties and uses. *Biometrika, 67*(2), 261–272. https://doi.org/10.1093/biomet/67.2.261

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Blei, D. M., & Lafferty, J. D. (2007). A correlated topic model of Science. *The Annals of Applied Statistics, 1*(1), 17–35. https://doi.org/10.1214/07-AOAS114

Egozcue, J. J., Pawlowsky-Glahn, V., Mateu-Figueras, G., & Barceló-Vidal, C. (2003). Isometric logratio transformations for compositional data analysis. *Mathematical Geology, 35*(3), 279–300. https://doi.org/10.1023/A:1023818214614

## Application

Aitchison and Shen (1980) define the logistic-normal family via the additive log-ratio map; Aitchison (1982) is the compositional-data authority that forbids treating parts of a whole as unconstrained Euclidean coordinates. Blei and Lafferty (2007) use that same reference-dependent map for correlated topic models. TEPP therefore uses `additive_log_ratio` for logistic-normal regression and psychometric interfaces, but does not claim that ALR preserves Aitchison distance. Egozcue et al. (2003) construct the sequential orthonormal ILR basis whose Euclidean norm equals Aitchison distance; `isometric_log_ratio` implements that basis and `from_isometric_log_ratio` inverts it through a max-shifted centered-log-ratio reconstruction. Analyses whose estimand is orthonormal Euclidean Aitchison geometry must use ILR rather than ALR. `from_additive_log_ratio` treats the omitted reference component as logit zero, max-shifts all `K` logits together, and normalizes only after exponentiation. The forward ALR map subtracts logarithms rather than forming a potentially overflowing ratio. Both inverses fail closed when an `f64` simplex part would underflow to zero (Aitchison & Shen, 1980; Aitchison, 1982; Blei & Lafferty, 2007; Egozcue et al., 2003).

## Verification

- closed-form simplex `(2,3,1)/6` maps to ALR `(ln 2, ln 3)` and sequential ILR `(√(2/3) ln(2√3/3), √(1/2) ln 3)` with computed RMSE below `1e-15`;
- representable ALR coordinates `(710, 709)` and representable ILR coordinates round-trip through max-shifted inverses without exponential overflow;
- extremes that would underflow a strictly positive `f64` simplex part fail closed;
- equal shares map to the ALR and ILR origins;
- two-part ILR preserves Aitchison distance `√(1/2) ln(0.8/0.2)` for `(0.8, 0.2)`;
- zero, negative, non-unit-sum, non-finite, empty, and one-part vectors fail closed;
- `tfidf`, `bm25`, and `keyword` labels are refused.
