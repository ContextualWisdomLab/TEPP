# Export retrieval CLI

## Scope

This note doctors the GAP-003A naruon export-retrieval CLI slice:

1. `tepp-export-get get` is the published operator CLI for
   `GET /v1/exports/{export_id}`;
2. the CLI mints `naruon_export_retrieval_exchange` and renders onto spawned
   `tepp-loopback` TCP;
3. success stdout is a metric-free `200 OK` identity;
4. public bind hosts, `localhost`, non-`https` origins, unpublished consumers,
   LineageWeave, and nonempty GET bodies fail closed.

Postgres persistence, restart/recovery, and Compose execution remain GAP-003B.
This slice is not implemented-main. It does not duplicate export retrieval GET
(#411), export-authorize CLI (#410), GET-by-id, wait CLI, lookup CLI, or
temporal-context CLI. It does not add GET to `NaruonLiveService`.

## Authoritative sources

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

National Academies of Sciences, Engineering, and Medicine. (2019).
*Reproducibility and replicability in science*. The National Academies Press.
https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values:
Context, process, and purpose. *The American Statistician, 70*(2), 129–133.
https://doi.org/10.1080/00031305.2016.1154108

## Application

RFC 9110 requires that a published method be invoked through a documented
interface, not an ad-hoc operator wire. The National Academies (2019) require
that a computational procedure be runnable from the published interface.
Wasserstein and Lazar (2016) refuse to treat a passing threshold as automatic
scientific authority, so the CLI emits the same metric-free identity as the
library bind and never treats HTTP success as ADR 0014 promotion. TEPP
therefore gives operators a credential-free export-retrieval CLI, refuses
LineageWeave on this naruon-owned adapter, and keeps RMSE, bias, coverage, and
scientific acceptance off the retrieval receipt (Fielding, Nottingham, &
Reschke, 2022; National Academies of Sciences, Engineering, and Medicine,
2019; Wasserstein & Lazar, 2016).

## Verification

- naruon export-retrieval CLI is HTTPS GET `/v1/exports/{export_id}` without
  credentials or RMSE keys;
- LineageWeave, public bind, `localhost`, `http://` origins, and nonempty
  bodies fail closed;
- POST mint then typed GET CLI stdout matches the minted `artifact_id`;
- `NaruonLiveService` still refuses GET.
