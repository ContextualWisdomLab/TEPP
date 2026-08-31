# Scientific-acceptance execute exchange on loopback TCP

## Scope

This note doctors the GAP-003A typed execute-exchange loopback TCP slice:

1. `loopback_http1_from_execute_exchange` renders a typed naruon/`LineageWeave` POST `/execute` onto the spawned `tepp-loopback` TCP listener;
2. the exchange keeps its HTTPS origin; only HTTP/1.1 `Host` is the loopback bind address;
3. public bind hosts, `localhost`, credential headers, and non-execute exchanges fail closed before any socket is opened;
4. POST create then the typed execute exchange over TCP then GET returns `tepp.scientific_acceptance.v1` for both consumers.

Postgres persistence, restart/recovery, and Compose execution remain GAP-003B. This slice is not implemented-main. It does not duplicate the execute consumer-exchange builders (#381), published binary (#375), engine-execute library (#370), cancel consumer parity (#373), loopback CLI (#362), collection CLI (#371), retry (#369), GET (#359), lifecycle POST (#360), cancel HTTP (#361), collection GET (#368), DTO (#358), or engine library (#356).

## Authoritative sources

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values: Context, process, and purpose. *The American Statistician, 70*(2), 129–133. https://doi.org/10.1080/00031305.2016.1154108

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

## Application

The National Academies (2019) require that a computational procedure be invoked through the published interface, not an ad-hoc consumer wire and not only an in-memory handler. Wasserstein and Lazar (2016) refuse to treat a passing threshold as automatic scientific authority, so the TCP path produces the same `tepp.scientific_acceptance.v1` evidence as the library bind and never treats HTTP `200` as ADR 0014 promotion. Wilson (1927) supplies the coverage interval already implemented in `validation_core`. TEPP therefore renders the typed execute exchange onto the spawned `tepp-loopback` listener, refuses public bind and `localhost`, and reports RMSE, bias, coverage, temporal order, and the SE-aware gate only after engine completion (National Academies of Sciences, Engineering, and Medicine, 2019; Wasserstein & Lazar, 2016; Wilson, 1927). Meredith (1993) remains unread (Unpaywall/OpenAlex 2026-08-31T13:00Z: `is_oa: false`, 0 locations). Mislevy (1991, *Psychometrika, 56*, 177–196) remains unread on the same terms (DOI `10.1007/bf02294457`).

## Verification

- public bind hosts and `localhost` fail closed without opening a socket;
- GET status exchanges are refused by the execute renderer;
- naruon and `LineageWeave` typed execute exchanges over spawned `tepp-loopback` TCP then GET return `tepp.scientific_acceptance.v1`.
