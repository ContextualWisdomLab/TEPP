# Scientific-acceptance published loopback binary (GAP-003A)

## Scope

This note doctors the GAP-003A published-binary slice:

1. the packaged `tepp-loopback` binary binds `ScientificAcceptanceLoopbackService` so `POST /v1/analysis-runs/{run_id}/execute` is reachable without embedding `analysis_engine`;
2. the binary still serves bounded `POST /v1/temporal-context`;
3. POST create then POST execute without `scientific_acceptance_json` then GET through the spawned binary returns `tepp.scientific_acceptance.v1`;
4. `tepp_api` no longer ships `tepp-loopback` (crate cycle; `cargo --bin` ambiguity).

Postgres persistence, restart/recovery, and Compose execution remain GAP-003B. This slice is not implemented-main. It does not duplicate the engine-execute library (#370), loopback CLI (#362), collection CLI (#371), GET (#359), lifecycle POST (#360), cancel HTTP (#361), collection GET (#368), retry HTTP (#369), DTO (#358), or engine library (#356).

## Authoritative sources

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values: Context, process, and purpose. *The American Statistician, 70*(2), 129–133. https://doi.org/10.1080/00031305.2016.1154108

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

## Application

The National Academies (2019) require that a computational procedure be runnable from the published interface, not only from an embedded library. Wasserstein and Lazar (2016) refuse to treat a passing threshold as automatic scientific authority, so the packaged listener produces the same `tepp.scientific_acceptance.v1` evidence as the library bind and never treats HTTP `200` as ADR 0014 promotion. Wilson (1927) supplies the coverage interval already implemented in `validation_core`. TEPP therefore moves `tepp-loopback` onto the engine wrapper, keeps temporal-context health checks, and still reports RMSE, bias, coverage, temporal order, and the SE-aware gate only after engine completion (National Academies of Sciences, Engineering, and Medicine, 2019; Wasserstein & Lazar, 2016; Wilson, 1927). Meredith (1993) remains unread (Unpaywall/OpenAlex 2026-08-31T11:00Z: `is_oa: false`, 0 locations). Mislevy (1991, *Psychometrika, 56*, 177–196) remains unread on the same terms (DOI `10.1007/bf02294457`).

## Verification

- the packaged binary returns `200` for one bounded temporal-context request;
- POST create then POST execute without `scientific_acceptance_json` then GET through the spawned binary returns `tepp.scientific_acceptance.v1`;
- Dockerfile builds `-p analysis_engine --bin tepp-loopback`.
