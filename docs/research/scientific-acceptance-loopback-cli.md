# Scientific-acceptance loopback CLI (GAP-003A)

## Scope

This note doctors the fifth GAP-003A executable slice in `tepp_api`
(issue #166):

1. `tepp-analysis-run create` POSTs a metric-free analysis-run receipt;
2. `tepp-analysis-run running` POSTs metric-free running status;
3. `tepp-analysis-run terminal` POSTs a request-bound terminal status;
4. `tepp-analysis-run status` GETs the current status;
5. accepted and running stdout stay metric-free, and only a succeeded status
   whose request profile is `scientific_acceptance_v1` may print
   `tepp.scientific_acceptance.v1`;
6. non-loopback hosts, credential-shaped flags, failed-plus-artifact emission,
   receipt RMSE/bias/coverage/SE-gate keys, and consumer mismatch fail closed.

This slice does not copy the terminal-result DTO. Library binding remains on
live PR #356. The API wire DTO remains on live PR #358. The GET status path
remains on live PR #359 / ADR 0027. The lifecycle POST path remains on live
PR #360 / ADR 0028. PostgreSQL persistence and Compose recovery remain
GAP-003B. CLI success does not promote an ADR 0014 claim.

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

RFC 9110 §9.3.3 defines POST as the method that processes a representation
according to the resource's own semantics, which remains the verb for
lifecycle transitions; GET remains a safe read (Fielding, Nottingham, &
Reschke, 2022). The CLI is a client of those methods, not a second HTTP
authority. Peng (2011) and the National Academies (2019) require
computational reproducibility to bind identities without treating a receipt
as a scientific claim, so RMSE, bias, coverage, and SE-gate keys stay off
create and running stdout. FIPS 180-4 SHA-256 continues to identify canonical
artifact bytes on the write path (National Institute of Standards and
Technology, 2015).

## Verification

- CLI create and CLI running contain neither `scientific_acceptance` nor
  `rmse`;
- CLI terminal succeeded with profile `scientific_acceptance_v1` then CLI
  status includes `tepp.scientific_acceptance.v1` only when the digest matches;
- failed-plus-artifact, non-loopback host, credential flags, and
  consumer/idempotency mismatch fail closed.
