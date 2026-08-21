# Corpus-background wording is not unique content (doctoring)

## Scope

`corpus_background` keeps corpus-level background language out of unique
latent content and out of global stopword deletion. Recovery is the
computed share of recovered kinds that match known truth.

This slice does not persist method sources, allocate migration `0008`,
or replace `method_effects`, `section_source`, `style_source`,
`copied_text`, `modality_source`, `stopword_deletion`, or the in-flight
TF-IDF/BM25 inferential-weight refusal.

## Authority

### Normative TEPP contract

- `docs/adr/0004-shared-multilingual-latent-space.md` and
  `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md`
  — template, section, copied-text, style, modality, and
  corpus-background sources are modeled explicitly and are not
  inferential topic weights or stopword deletions.

### Supporting literature

Chemudugunta, Smyth, and Steyvers (2007) separate a shared background
word distribution from document-specific topical content. Background
mass is not unique latent meaning and is not deleted by a stopword
list.

Chemudugunta, C., Smyth, P., & Steyvers, M. (2007). Modeling general
and specific aspects of documents with a probabilistic topic model. In
B. Schölkopf, J. Platt, & T. Hoffman (Eds.), *Advances in Neural
Information Processing Systems 19* (pp. 241–248). MIT Press.
