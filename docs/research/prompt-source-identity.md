# Prompt boilerplate is not unique content (doctoring)

## Scope

`prompt_source` keeps instruction and prompt boilerplate out of unique
latent content and out of global stopword deletion. Recovery is the
computed share of recovered kinds that match known truth.

This slice does not persist method sources, allocate migration `0008`,
or replace `method_effects`, `section_source`, `style_source`,
`copied_text`, `modality_source`, `corpus_background`, or
`stopword_deletion`.

## Authority

### Normative TEPP contract

- `docs/adr/0004-shared-multilingual-latent-space.md` and
  `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md`
  — method and template sources are modeled explicitly and are not
  inferential topic weights or stopword deletions.

### Supporting literature

Liu et al. (2023) treat prompting as a method condition that shapes
emissions. Prompt text is not the document's unique latent meaning.

Liu, P., Yuan, W., Fu, J., Jiang, Z., Hayashi, H., & Neubig, G. (2023).
Pre-train, prompt, and predict: A systematic survey of prompting methods
in natural language processing. *ACM Computing Surveys, 55*(9), Article
195. https://doi.org/10.1145/3560815
