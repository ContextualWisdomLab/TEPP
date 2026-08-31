# Analysis-run lifecycle consumer parity (GAP-003A)

## Scope

This note doctors the LineageWeave running/terminal exchange builders and the
Naruon compatibility-listener lifecycle POST stacked on ADR 0028 / issue #166:

1. `lineageweave_analysis_run_running_exchange` and
   `lineageweave_analysis_run_terminal_exchange` reuse the Naruon builders and
   replace only `tepp-consumer`;
2. `NaruonLiveService` records metric-free running and request-bound terminal
   status for Naruon only;
3. LineageWeave remains refused on `NaruonLiveService` and uses
   `AnalysisRunLiveService`;
4. `tepp-loopback` proves create-then-running over loopback TCP;
5. GET is still refused on `NaruonLiveService`.

This slice does not duplicate the shared-listener lifecycle POST (#360), GET
status (#359), status consumer-parity (#383), cancel (#361/#373), collection
GET, retry, the terminal-result DTO (#358), or the engine library (#356).
PostgreSQL persistence remains GAP-003B. HTTP success does not promote an
ADR 0014 claim.

## Authoritative sources

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). Internet Engineering Task Force. https://doi.org/10.17487/RFC9110

National Institute of Standards and Technology. (2015). *Secure Hash Standard
(SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4

Peng, R. D. (2011). Reproducible research in computational science.
*Science, 334*(6060), 1226–1227. https://doi.org/10.1126/science.1213847

National Academies of Sciences, Engineering, and Medicine. (2019).
*Reproducibility and replicability in science*. The National Academies Press.
https://doi.org/10.17226/25303

## Application

RFC 9110 §9.3.3 keeps POST as the lifecycle write. The compatibility listener
already admits Naruon POSTs; extending it to `/running` and `/terminal` does
not require GET. Consumer identity stays a header swap so LineageWeave cannot
be forced to mint a Naruon-labelled transition. FIPS 180-4 digest binding on
succeeded scientific-acceptance terminals is unchanged from ADR 0028.

## Non-application

This note does not authorize public bind, TLS termination, durable status
storage, opening `NaruonLiveService` to LineageWeave, or treating HTTP `200`
as a scientific claim.
