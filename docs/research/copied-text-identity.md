# Copied-text residue is not unique content (doctoring)

## Scope

`copied_text` keeps copied and boilerplate passages out of unique latent
content and out of global stopword deletion. Recovery is the computed
share of recovered kinds that match known truth.

This slice does not persist method sources, allocate migration `0008`,
or replace `method_effects`, `section_source`, `style_source`, or
`stopword_deletion`.

## Authority

### Normative TEPP contract

- `docs/adr/0004-shared-multilingual-latent-space.md` and
  `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md`
  — template, section, copied-text, style, modality, and
  corpus-background sources are modeled explicitly and are not
  inferential topic weights or stopword deletions.

### Supporting literature

Kohlschütter, Fankhauser, and Nejdl (2010) separate boilerplate and
copied residue from unique page content using shallow text features.
Copied wording is structure, not the unique latent meaning TEPP must
keep distinct.

Kohlschütter, C., Fankhauser, P., & Nejdl, W. (2010). Boilerplate
detection using shallow text features. In *Proceedings of the Third ACM
International Conference on Web Search and Data Mining* (pp. 441–450).
Association for Computing Machinery.
https://doi.org/10.1145/1718487.1718542
