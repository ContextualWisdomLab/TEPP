# Global topic activity identity (doctoring)

## Scope

`topic_lineage` keeps one P0 topic identity across `active`, `dormant`, and
`reactivated` states. Reactivation cannot mint a new identity. Identity
recovery is the computed share of recovered identities that match known truth.

This slice does not fit a topic model, implement birth/split/merge/retirement,
or treat activity change as a new latent construct.

## Authority

### Normative TEPP contract

- `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md` —
  one global topic identity set is selected across the modeled period; topics
  may be active, dormant, or reactivated without losing identity; birth, split,
  merge, and retirement are a later explicit lineage extension.

### Supporting literature

Blei and Lafferty (2006) and Roberts, Stewart, and Tingley (2019) motivate
temporal topic prevalence that can rise and fall. They do **not** authorize
minting a new topic identity merely because prevalence returns after a quiet
period.

Blei, D. M., & Lafferty, J. D. (2006). Dynamic topic models. In *Proceedings
of the 23rd International Conference on Machine Learning* (pp. 113–120).
Association for Computing Machinery. https://doi.org/10.1145/1143844.1143859

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for
structural topic models. *Journal of Statistical Software, 91*(2), 1–40.
https://doi.org/10.18637/jss.v091.i02
