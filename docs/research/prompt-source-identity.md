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

Brown et al. (2020) provide primary evidence that textual prompts and
demonstrations condition language-model task behavior, while Reynolds and
McDonell (2021) study prompt programming as a method for directing model
behavior. Neither study defines TEPP's latent-content labels. The statement
that prompt boilerplate is not unique latent content is therefore a normative
TEPP measurement contract derived from ADR 0004 and ADR 0012, not a universal
empirical claim about every prompt or corpus.

Brown, T. B., Mann, B., Ryder, N., Subbiah, M., Kaplan, J. D., Dhariwal, P.,
Neelakantan, A., Shyam, P., Sastry, G., Askell, A., Agarwal, S., Herbert-Voss,
A., Krueger, G., Henighan, T., Child, R., Ramesh, A., Ziegler, D., Wu, J.,
Winter, C., … Amodei, D. (2020). Language models are few-shot learners.
*Advances in Neural Information Processing Systems, 33*, 1877–1901.
https://papers.neurips.cc/paper/2020/hash/1457c0d6bfcb4967418bfb8ac142f64a-Abstract.html

Reynolds, L., & McDonell, K. (2021). Prompt programming for large language
models: Beyond the few-shot paradigm. In *Extended abstracts of the 2021 CHI
conference on human factors in computing systems*. Association for Computing
Machinery. https://doi.org/10.1145/3411763.3451760
