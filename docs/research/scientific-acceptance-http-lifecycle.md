# Scientific-acceptance loopback HTTP lifecycle POST (GAP-003A)

## Scope

This note doctors the fourth GAP-003A executable slice in `tepp_api`
(issue #166):

1. `POST /v1/analysis-runs` remains a metric-free receipt;
2. `POST /v1/analysis-runs/{run_id}/running` records metric-free running status;
3. `POST /v1/analysis-runs/{run_id}/terminal` records a request-bound terminal
   status, with canonical `tepp.scientific_acceptance.v1` bytes only when the
   request profile is `scientific_acceptance_v1` and the run succeeded;
4. reverse transitions, mutating a terminal run, failed-plus-artifact emission,
   receipt RMSE/bias/coverage/SE-gate keys, an unknown run, and consumer
   mismatch fail closed.

This slice does not copy the terminal-result DTO. Library binding remains on
live PR #356. The API wire DTO remains on live PR #358. The GET status path
remains on live PR #359 / ADR 0027. PostgreSQL persistence and Compose recovery
remain GAP-003B. HTTP success does not promote an ADR 0014 claim.

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
according to the resource's own semantics, which is the correct verb for a
lifecycle transition; GET remains a safe read (Fielding, Nottingham, &
Reschke, 2022). Peng (2011) and the National Academies (2019) require
computational reproducibility to bind identities without treating a receipt as
a scientific claim, so RMSE, bias, coverage, and SE-gate keys stay off POST
receipts and running bodies. FIPS 180-4 SHA-256 hashes the canonical artifact
bytes carried as `scientific_acceptance_json` and refuses an all-zero digest
(National Institute of Standards and Technology, 2015).

## Verification

- POST running contains neither `scientific_acceptance` nor `rmse`;
- POST terminal succeeded with profile `scientific_acceptance_v1` then GET
  includes `tepp.scientific_acceptance.v1` only when the digest matches;
- failed-plus-artifact, reverse transitions, unknown run, and
  consumer/idempotency mismatch fail closed.
