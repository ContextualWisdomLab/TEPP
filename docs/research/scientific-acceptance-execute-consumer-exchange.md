# Scientific-acceptance execute consumer exchange

## Scope

This note doctors the GAP-003A naruon/`LineageWeave` execute consumer-exchange slice:

1. `ScientificAcceptanceExecuteRequest` is the typed execute body naruon and `LineageWeave` may POST;
2. `naruon_analysis_run_execute_exchange` and `lineageweave_analysis_run_execute_exchange` mint credential-free HTTPS `POST /v1/analysis-runs/{run_id}/execute`;
3. the body carries corpus, recovery, seed, and pre-registered SE-gate `k` and must not carry `scientific_acceptance_json`;
4. LLM-authored recovery, receipt metric keys, unknown artifact fields, and non-`https` origins fail closed.

Postgres persistence, restart/recovery, and Compose execution remain GAP-003B. This slice is not implemented-main. It does not duplicate the published binary (#375), engine-execute library (#370), cancel consumer parity (#373), loopback CLI (#362), collection CLI (#371), retry (#369), GET (#359), lifecycle POST (#360), cancel HTTP (#361), collection GET (#368), DTO (#358), or engine library (#356).

## Authoritative sources

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values: Context, process, and purpose. *The American Statistician, 70*(2), 129–133. https://doi.org/10.1080/00031305.2016.1154108

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

## Application

The National Academies (2019) require that a computational procedure be invoked through a published interface, not an ad-hoc consumer wire. Wasserstein and Lazar (2016) refuse to treat a passing threshold as automatic scientific authority, so the typed exchange produces the same `tepp.scientific_acceptance.v1` evidence as the library bind and never treats HTTP `200` as ADR 0014 promotion. Wilson (1927) supplies the coverage interval already implemented in `validation_core`. TEPP therefore gives naruon and `LineageWeave` a credential-free execute exchange, refuses caller-supplied artifacts and LLM-authored recovery, and reports RMSE, bias, coverage, temporal order, and the SE-aware gate only after engine completion (National Academies of Sciences, Engineering, and Medicine, 2019; Wasserstein & Lazar, 2016; Wilson, 1927). Meredith (1993) remains unread (Unpaywall/OpenAlex 2026-08-31T13:00Z: `is_oa: false`, 0 locations). Mislevy (1991, *Psychometrika, 56*, 177–196) remains unread on the same terms (DOI `10.1007/bf02294457`).

## Verification

- naruon execute exchange is HTTPS POST `/execute` without credentials or `scientific_acceptance_json`;
- LineageWeave execute exchange changes only `tepp-consumer`;
- LLM recovery, metric keys, unknown artifact fields, and `http://` origins fail closed;
- POST create then the typed execute exchange then GET returns `tepp.scientific_acceptance.v1` for both consumers.
