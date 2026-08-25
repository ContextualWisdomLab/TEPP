# Non-lexical modality is not unique content (doctoring)

## Scope

`modality_source` keeps non-lexical modality channels out of unique
latent content and out of global stopword deletion. Recovery is the
computed share of recovered kinds that match known truth.

This slice does not persist method sources, allocate migration `0008`,
or replace `method_effects`, `section_source`, `style_source`,
`copied_text`, or `stopword_deletion`.

## Authority

### Normative TEPP contract

- `docs/adr/0004-shared-multilingual-latent-space.md` and
  `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md`
  — template, section, copied-text, style, modality, and
  corpus-background sources are modeled explicitly and are not
  inferential topic weights or stopword deletions.

### Supporting literature

Bateman (2008) treats modality as a distinct meaning-making resource,
not as lexical content to delete or as the same construct as wording.

Bateman, J. A. (2008). *Multimodality and genre: A foundation for the
systematic analysis of multimodal documents*. Palgrave Macmillan.
