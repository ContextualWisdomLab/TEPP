# Analysis-run retry-parent CLI

## Scope

This note doctors the GAP-003A naruon/`LineageWeave` retry-parent CLI slice:

1. `tepp-retry-parent parent` is the published operator CLI for `GET /v1/analysis-runs/{run_id}/parent`;
2. the CLI mints `naruon_analysis_run_retry_parent_exchange` or
   `lineageweave_analysis_run_retry_parent_exchange` and renders onto spawned `tepp-loopback` TCP;
3. success stdout is a metric-free `200 OK` inspect (`"parent": null` when never retried);
4. public bind hosts, `localhost`, non-`https` origins, unpublished consumers, and nonempty GET bodies fail closed.

Postgres persistence, restart/recovery, and Compose execution remain GAP-003B. This slice is not implemented-main. It does not duplicate retry-parent GET (#384), retry-parent consumer parity (#396), retry CLI (#394), retry HTTP (#369), retry consumer parity (#393), stored-request CLI (#395), or status CLI (#392). It does not add GET to `NaruonLiveService` beyond the Naruon-only compatibility inspect.

## Authoritative sources

Fielding, R., Ed., & Reschke, J., Ed. (2014). *Hypertext Transfer Protocol (HTTP/1.1): Semantics and Content* (RFC 9110). RFC Editor. https://www.rfc-editor.org/rfc/rfc9110

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values: Context, process, and purpose. *The American Statistician, 70*(2), 129–133. https://doi.org/10.1080/00031305.2016.1154108

## Application

RFC 9110 requires that a published method be invoked through a documented interface, not an ad-hoc operator wire. The National Academies (2019) require that a computational procedure be runnable from the published interface. Wasserstein and Lazar (2016) refuse to treat a passing threshold as automatic scientific authority, so the CLI emits the same metric-free inspect as the library bind and never treats HTTP success as ADR 0014 promotion. TEPP therefore gives operators a credential-free retry-parent CLI, refuses caller-supplied artifacts, and keeps RMSE, bias, coverage, and scientific acceptance off the inspect receipt (Fielding & Reschke, 2014; National Academies of Sciences, Engineering, and Medicine, 2019; Wasserstein & Lazar, 2016). Meredith (1993) remains unread (Unpaywall/OpenAlex 2026-08-31T16:00Z: `is_oa: false`, 0 locations). Mislevy (1991, *Psychometrika, 56*, 177–196) remains unread on the same terms (DOI `10.1007/bf02294457`).

## Verification

- naruon retry-parent CLI is HTTPS GET `/parent` without credentials or RMSE keys;
- LineageWeave retry-parent CLI changes only `tepp-consumer`;
- public bind, `localhost`, `http://` origins, and nonempty bodies fail closed;
- create then cancel then retry then typed parent CLI stdout is a metric-free non-null parent for both consumers.
