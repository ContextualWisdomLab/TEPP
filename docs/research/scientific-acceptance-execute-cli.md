# Scientific-acceptance execute loopback CLI

## Scope

This note doctors the GAP-003A execute CLI slice:

1. `tepp-execute execute` POSTs `/v1/analysis-runs/{run_id}/execute` from a typed naruon/`LineageWeave` execute exchange;
2. the exchange keeps its HTTPS origin; only `--host` is the loopback bind address printed by `tepp-loopback`;
3. public bind hosts, `localhost`, credential-shaped flags, empty stdin, LLM recovery, metric keys, and `http://` origins fail closed before any socket is opened;
4. naruon and `LineageWeave` CLI execute against spawned `tepp-loopback` TCP print `tepp.scientific_acceptance.v1`.

Postgres persistence, restart/recovery, and Compose execution remain GAP-003B. This slice is not implemented-main. It does not duplicate the TCP renderer (#382), execute builders (#381), published binary (#375), engine-execute (#370), lifecycle CLI (#362), create CLI (#385), cancel consumer-parity (#373), cancel CLI (#378), stored-request GET (#377), stored-request consumer-parity (#387), retry-children (#379), idempotency (#380), retry-parent (#384), collection GET (#368), GET (#359), lifecycle POST (#360), cancel HTTP (#361), collection CLI (#371), retry (#369), engine-library (#356), DTO (#358), persistence (#287), Leiden (#351), Driver p.16, CWC/Rubin/ESEM/OLS, GAP-010, or GAP-003C.

## Authoritative sources

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values: Context, process, and purpose. *The American Statistician, 70*(2), 129–133. https://doi.org/10.1080/00031305.2016.1154108

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

## Application

The National Academies (2019) require that a computational procedure be invoked through the published interface, not an ad-hoc consumer wire. Wasserstein and Lazar (2016) refuse to treat a passing threshold as automatic scientific authority, so the CLI produces the same `tepp.scientific_acceptance.v1` evidence as the library bind and never treats process exit 0 as ADR 0014 promotion. Wilson (1927) supplies the coverage interval already implemented in `validation_core`. TEPP therefore publishes `tepp-execute` as the operator client of the typed execute exchange on spawned `tepp-loopback` TCP, refuses public bind and `localhost`, and reports RMSE, bias, coverage, temporal order, and the SE-aware gate only after engine completion (National Academies of Sciences, Engineering, and Medicine, 2019; Wasserstein & Lazar, 2016; Wilson, 1927). Meredith (1993) remains unread (Unpaywall/OpenAlex 2026-08-31T14:00Z: `is_oa: false`, 0 locations). Mislevy (1991, *Psychometrika, 56*, 177–196) remains unread on the same terms (DOI `10.1007/bf02294457`).

## Verification

- public bind hosts and `localhost` fail closed without opening a socket;
- credential-shaped flags, empty stdin, LLM recovery, metric keys, and `http://` origins fail closed;
- naruon and `LineageWeave` `tepp-execute execute` against spawned `tepp-loopback` TCP print `tepp.scientific_acceptance.v1`.
