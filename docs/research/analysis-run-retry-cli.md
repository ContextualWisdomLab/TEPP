# Analysis-run retry CLI

## Scope

This note doctors the GAP-003A naruon/`LineageWeave` retry CLI slice:

1. `tepp-retry retry` is the published operator CLI for `POST /v1/analysis-runs/{run_id}/retry`;
2. the CLI mints `naruon_analysis_run_retry_exchange` or `lineageweave_analysis_run_retry_exchange` and renders onto spawned `tepp-loopback` TCP;
3. success stdout is a metric-free child `202 Accepted` with a new `run_id` and a new idempotency key;
4. public bind hosts, `localhost`, non-`https` origins, unpublished consumers, and retry of accepted parents fail closed.

Postgres persistence, restart/recovery, and Compose execution remain GAP-003B. This slice is not implemented-main. It does not duplicate retry HTTP (#369), retry consumer parity (#393), execute CLI (#390), cancel CLI (#378), create CLI (#385), status CLI (#392), collection CLI (#371), or lifecycle CLI (#362).

## Authoritative sources

Fielding, R., Ed., & Reschke, J., Ed. (2014). *Hypertext Transfer Protocol (HTTP/1.1): Semantics and Content* (RFC 9110). RFC Editor. https://www.rfc-editor.org/rfc/rfc9110

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values: Context, process, and purpose. *The American Statistician, 70*(2), 129–133. https://doi.org/10.1080/00031305.2016.1154108

## Application

RFC 9110 requires that a published method be invoked through a documented interface, not an ad-hoc operator wire. The National Academies (2019) require that a computational procedure be runnable from the published interface. Wasserstein and Lazar (2016) refuse to treat a passing threshold as automatic scientific authority, so the CLI emits the same metric-free child `202 Accepted` as the library bind and never treats HTTP success as ADR 0014 promotion. TEPP therefore gives operators a credential-free retry CLI, refuses caller-supplied artifacts, and keeps RMSE, bias, coverage, and scientific acceptance off the retry receipt (Fielding & Reschke, 2014; National Academies of Sciences, Engineering, and Medicine, 2019; Wasserstein & Lazar, 2016). Meredith (1993) remains unread (Unpaywall/OpenAlex 2026-08-31T15:00Z: `is_oa: false`, 0 locations). Mislevy (1991, *Psychometrika, 56*, 177–196) remains unread on the same terms (DOI `10.1007/bf02294457`).

## Verification

- naruon retry CLI is HTTPS POST `/retry` without credentials or RMSE keys;
- LineageWeave retry CLI changes only `tepp-consumer`;
- public bind, `localhost`, `http://` origins, and accepted parents fail closed;
- create then cancel then typed retry CLI then stdout is a new metric-free `202 Accepted` for both consumers.
