# House-voice style residue is not unique content (doctoring)

## Scope

`style_source` keeps house-voice and style residue out of unique latent
content and out of global stopword deletion. Recovery is the computed
share of recovered kinds that match known truth.

This slice does not persist method sources, allocate migration `0008`,
or replace `method_effects`, `section_source`, or `stopword_deletion`.

## Authority

### Normative TEPP contract

- `docs/adr/0004-shared-multilingual-latent-space.md` and
  `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md`
  — template, section, copied-text, style, modality, and
  corpus-background sources are modeled explicitly and are not
  inferential topic weights or stopword deletions.

### Supporting literature

Roberts, Stewart, and Tingley (2019) treat style and other
document-level covariates as explicit structure in a structural topic
model, not as tokens to delete.

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package
for structural topic models. *Journal of Statistical Software, 91*(2),
1–40. https://doi.org/10.18637/jss.v091.i02
