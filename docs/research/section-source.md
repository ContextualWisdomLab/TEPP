# Report section boilerplate is not unique content (doctoring)

## Scope

`section_source` keeps report section headings and other section
boilerplate from being treated as unique latent content or erased by a
stopword list. Recovery is the computed share of recovered section
kinds that match known truth.

This slice does not persist tokens, allocate migration `0008`, apply
TF-IDF/BM25 inferential weights, or replace `method_effects` or
`stopword_deletion`.

## Authority

### Normative TEPP contract

- `docs/adr/0004-shared-multilingual-latent-space.md` —
  repeated template/section/copied wording is modeled as
  method/background structure.
- `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md` —
  section effects are explicit method/background structure, not
  stopword deletion.

### Supporting literature

Chemudugunta, Smyth, and Steyvers (2007) separate general document
wording from specific content. Section headings are general structure
and must not be collapsed into unique latent meaning.

Chemudugunta, C., Smyth, P., & Steyvers, M. (2007). Modeling general
and specific aspects of documents with a probabilistic topic model. In
B. Schölkopf, J. C. Platt, & T. Hoffman (Eds.), *Advances in neural
information processing systems 19* (pp. 241–248). MIT Press.
https://papers.nips.cc/paper/3048-modeling-general-and-specific-aspects-of-documents-with-a-probabilistic-topic-model
