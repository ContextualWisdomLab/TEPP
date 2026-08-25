# Fitted candidate-K scoring (doctoring)

## Claim boundary

`select_fitted_candidate_k` fits each caller-supplied candidate `K` with the
CPU `f64` TRSL-TM reference and scores the actual in-sample mixture
log-likelihood. It does not run GPU inference, full Bayesian sampling, or
topic birth/split/merge, and it does not close issue #167.

## Numeric constants

| Constant | Value | Provenance |
|---|---|---|
| prior variance `σ²` | `1.0` | Identical copy of `topic_measurement` `DEFAULT_PRIOR_VARIANCE`; ADR 0012 names `σ²` in the MAP objective |
| relation strength `λ` | `0.25` | Identical copy of `topic_measurement` `DEFAULT_RELATION_STRENGTH`; ADR 0012 names `λ` in `R(Θ,G)` |
| ridge `ρ` | `0.01` | Identical copy of `topic_measurement` `DEFAULT_RIDGE`; ADR 0012 names `ρ` on `(Γ,u)` |
| topic smoothing | `0.05` | Identical copy of `topic_measurement` `DEFAULT_TOPIC_SMOOTHING` for the smoothed multinomial `β` update |
| GEM step | `0.2` | Identical copy of `topic_measurement` `DEFAULT_STEP_SIZE` |
| Schwarz penalty | `ℓ − (p ln N)/2` | Schwarz (1978, p. 461) maximizer `log M_j − (1/2) k_j log n` |

These copies are not a second heuristic set. A value that diverged from the
`topic_measurement` reference would be a defect.

## Primary sources

Schwarz, G. (1978). Estimating the dimension of a model. *The Annals of
Statistics, 6*(2), 461–464. https://doi.org/10.1214/aos/1176344136
(Project Euclid PDF opened 2026-08-25T06:08Z from
https://projecteuclid.org/journalArticle/Download?urlId=10.1214%2Faos%2F1176344136)

Roberts, M. E., Stewart, B. M., Tingley, D., Lucas, C., Leder-Luis, J.,
Gadarian, S. K., Albertson, B., & Rand, D. G. (2014). Structural topic models
for open-ended survey responses. *American Journal of Political Science,
58*(4), 1064–1082. https://doi.org/10.1111/ajps.12103

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for
structural topic models. *Journal of Statistical Software, 91*(2), 1–40.
https://doi.org/10.18637/jss.v091.i02
