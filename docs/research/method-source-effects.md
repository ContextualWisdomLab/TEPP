# Estimator-side method sources (doctoring)

## Scope

`method_effects` records template, section, copied-text, style, modality, and
corpus-background language as explicit method sources. Those sources cannot be
used as inferential topic weights. Recovery is the computed share of recovered
source labels that match known truth.

This slice does not remove stopwords, apply TF-IDF or BM25 as estimator
weights, or fit a topic model.

## Authority

### Normative TEPP contract

- `docs/adr/0004-shared-multilingual-latent-space.md` and
  `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md` —
  repeated template, section, copied-text, style, modality, source, and
  corpus-background effects are modeled explicitly; stopword deletion is not
  the default; TF-IDF and BM25 are not inferential weights.

### Supporting literature

Roberts, Stewart, and Tingley (2019) treat prevalence covariates and
content covariates as distinct channels. They do **not** authorize collapsing
boilerplate or copied text into inferential topic weights.

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for
structural topic models. *Journal of Statistical Software, 91*(2), 1–40.
https://doi.org/10.18637/jss.v091.i02
