# Scientific-acceptance loopback HTTP status (GAP-003A)

## Scope

This note doctors the third GAP-003A executable slice in `tepp_api`
(issue #166):

1. `POST /v1/analysis-runs` remains a metric-free receipt;
2. `GET /v1/analysis-runs/{run_id}` returns accepted/running status without
   scientific-acceptance metrics;
3. only a succeeded status with output profile `scientific_acceptance_v1` may
   return `tepp.scientific_acceptance.v1`;
4. failed-plus-artifact emission, all-zero binding or result digests, digest
   mismatch, receipt RMSE/bias/coverage/SE-gate keys, a GET body, and unknown
   run identities fail closed.

This slice does not copy the terminal-result DTO. Library binding remains on
live PR #356. The API wire DTO remains on live PR #358. PostgreSQL persistence
and Compose recovery remain GAP-003B. HTTP success does not promote an ADR 0014
claim.

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

RFC 9110 §9.3.1 defines GET as a safe read that does not create a new
resource; TEPP therefore refuses a GET body and serves only the stored status
for the opaque run identity (Fielding, Nottingham, & Reschke, 2022). Peng
(2011) and the National Academies (2019) require computational reproducibility
to bind identities without treating a receipt as a scientific claim, so RMSE,
bias, coverage, and SE-gate keys stay off POST receipts and accepted/running
GET bodies. FIPS 180-4 SHA-256 detects whether the HTTP artifact bytes agree
with `result_sha256` and refuses an all-zero digest (National Institute of
Standards and Technology, 2015).

## Verification

- POST with `rmse` (or other named metric keys) fails closed;
- GET accepted and GET running contain neither `scientific_acceptance` nor
  `rmse`;
- GET succeeded with profile `scientific_acceptance_v1` includes
  `tepp.scientific_acceptance.v1` only when the digest matches;
- failed-plus-artifact, all-zero digest, digest mismatch, GET body, unknown
  run, and consumer/idempotency mismatch fail closed.
