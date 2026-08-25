# Evidence-bounded LLM interpretation (doctoring)

## Scope

`interpretation_gateway` records an LLM interpretation as an untrusted
hypothetical proposal. The proposal must cite at least one evidence span. It
cannot be treated as a statistical estimator result or as an observed fact.
Unsupported-claim rate is computed from known-truth support labels: the share
of unsupported claims that a decider marks supported.

This slice does not select an orchestration mode, call a live model, or promote
an interpretation into TEPP scientific authority.

## Authority

### Normative TEPP contract

- `docs/adr/0010-adaptive-llm-orchestration.md` — LLM output is always an
  untrusted proposal tied to evidence; statistical estimation and release
  authority remain outside LLM authority.
- `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md` —
  LLM review is blinded and statistically gated; it never defines the numerical
  optimum.

### Supporting literature

Chang et al. (2009) and Stammbach et al. (2023) treat LLM or human topic
judgments as complementary evaluation, not as a replacement for predictive or
recovery evidence. They do **not** authorize an uncited interpretation as an
estimator or observed event.

Chang, J., Gerrish, S., Wang, C., Boyd-Graber, J. L., & Blei, D. M. (2009).
Reading tea leaves: How humans interpret topic models. In *Advances in Neural
Information Processing Systems 22*.

Stammbach, D., Zouhar, V., Hoyle, A., Sachan, M., & Ash, E. (2023). Revisiting
automated topic model evaluation with large language models. In *Proceedings of
the 2023 Conference on Empirical Methods in Natural Language Processing*
(pp. 9348–9357). Association for Computational Linguistics.
https://doi.org/10.18653/v1/2023.emnlp-main.581
