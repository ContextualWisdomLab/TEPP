# Default stopword deletion is not a valid method (doctoring)

## Scope

`stopword_deletion` keeps a default or global stopword list from erasing
repeated report language. Recovery is the computed share of recovered
deletion kinds that match known truth.

This slice does not persist tokens, allocate migration `0008`, apply
TF-IDF/BM25 inferential weights, or replace `method_effects`.

## Authority

### Normative TEPP contract

- `docs/adr/0004-shared-multilingual-latent-space.md` —
  stopword deletion is not the default; repeated template/section/copied
  wording is modeled as method/background structure.
- `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md` —
  stopword deletion is not the default preprocessing rule.

### Supporting literature

Schofield, Magnusson, and Mimno (2017) show that stopword removal is not
a harmless default for topic models. A global list can remove
substantive terms and hide the method-source structure TEPP must keep
explicit.

Schofield, A., Magnusson, M., & Mimno, D. (2017). Pulling out the stops:
Rethinking stopword removal for topic models. In *Proceedings of the 15th
Conference of the European Chapter of the Association for Computational
Linguistics: Volume 2, Short Papers* (pp. 432–436). Association for
Computational Linguistics. https://doi.org/10.18653/v1/E17-2069
