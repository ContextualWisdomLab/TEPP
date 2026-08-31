# Scientific-acceptance loopback engine execute

## Scope

This note doctors the GAP-003A engine-on-loopback slice:

1. `POST /v1/analysis-runs/{run_id}/execute` runs `submit_validation_run` and `complete_validation_run` against an accepted loopback run;
2. the execute body carries corpus, recovery, seed, and pre-registered SE-gate `k` and must not carry `scientific_acceptance_json`;
3. the engine records running then terminal so GET returns `tepp.scientific_acceptance.v1` without a caller-supplied artifact;
4. wrong profile, LLM-authored recovery, receipt metric keys, unknown run, consumer mismatch, already-terminal status, and digest mismatch fail closed.

Postgres persistence, restart/recovery, and Compose execution remain GAP-003B. This slice is not implemented-main. It does not duplicate the engine library (#356), terminal-result DTO (#358), GET (#359), lifecycle POST (#360), cancel HTTP (#361), loopback CLI (#362), or collection GET (#368).

## Authoritative sources

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values: Context, process, and purpose. *The American Statistician, 70*(2), 129–133. https://doi.org/10.1080/00031305.2016.1154108

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

## Application

The National Academies (2019) separate computational reproducibility (same binding, same digest) from a scientific claim that recovery is correct. Wasserstein and Lazar (2016) refuse to treat a passing threshold as automatic scientific authority, so execute produces the same `tepp.scientific_acceptance.v1` evidence as the library bind, including `se_gate_accepted = false` when the pre-registered gate fails, and never treats HTTP `200` as ADR 0014 promotion. Wilson (1927) supplies the coverage interval already implemented in `validation_core`. TEPP therefore executes the engine on the loopback lifecycle path, refuses caller-supplied artifacts and LLM-authored recovery, and reports RMSE, bias, coverage, temporal order, and the SE-aware gate only after engine completion (National Academies of Sciences, Engineering, and Medicine, 2019; Wasserstein & Lazar, 2016; Wilson, 1927). Meredith (1993) remains unread (Unpaywall/OpenAlex 2026-08-31T10:00Z: `is_oa: false`, 0 locations). Mislevy (1991, *Psychometrika, 56*, 177–196) remains unread on the same terms (DOI `10.1007/bf02294457`).

## Verification

- POST create then POST execute without `scientific_acceptance_json` then GET returns `tepp.scientific_acceptance.v1`;
- a `calibrated_event_measurement` run stays accepted and metric-free after execute is refused;
- LLM authorship, receipt metric keys, unknown-field `scientific_acceptance_json`, unknown run, consumer mismatch, and a second execute on a terminal run fail closed;
- the raw `AnalysisRunLiveService` still returns `400` for `/execute`.
