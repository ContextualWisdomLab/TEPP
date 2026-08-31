# Analysis-run lifecycle CLI (GAP-003A)

## Scope

This note doctors the published `tepp-lifecycle` CLI stacked on ADR 0028/0029
and issue #166:

1. `tepp-lifecycle running` mints a typed naruon or `LineageWeave` running
   exchange and POSTs it onto spawned `tepp-loopback` TCP;
2. `tepp-lifecycle terminal` requires typed JSON and records a request-bound
   terminal status;
3. empty stdin is admitted for running; public bind, `localhost`, unpublished
   consumers, credential flags, and non-`https` origins fail closed;
4. stdout stays metric-free; `tepp.scientific_acceptance.v1` never appears;
5. `NaruonLiveService` stays POST-only.

This slice does not duplicate the shared-listener lifecycle POST (#360),
lifecycle consumer-parity (#388), GET status (#359), cancel, collection, retry,
the terminal-result DTO (#358), or the engine library (#356). PostgreSQL
persistence remains GAP-003B. HTTP success does not promote an ADR 0014 claim.

## Authoritative sources

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). Internet Engineering Task Force. https://doi.org/10.17487/RFC9110

Peng, R. D. (2011). Reproducible research in computational science.
*Science, 334*(6060), 1226–1227. https://doi.org/10.1126/science.1213847

National Academies of Sciences, Engineering, and Medicine. (2019).
*Reproducibility and replicability in science*. The National Academies Press.
https://doi.org/10.17226/25303

## Application

RFC 9110 §9.3.3 keeps POST as the lifecycle write. The CLI mints the same
exchanges ADR 0029 already publishes and only rewrites the HTTP/1.1 `Host` to
the loopback bind address. Consumer identity stays a header swap so
LineageWeave cannot be forced to mint a Naruon-labelled transition.

## Non-application

This note does not authorize public bind, TLS termination, durable status
storage, opening `NaruonLiveService` to LineageWeave or GET, or treating HTTP
`200` as a scientific claim.
