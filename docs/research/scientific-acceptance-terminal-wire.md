# Scientific-acceptance terminal wire (GAP-003A)

## Scope

This note doctors the second GAP-003A executable slice in `tepp_api`
(issue #166):

1. `AnalysisRunRequest` and `AnalysisRunAccepted` remain metric-free receipts;
2. only a succeeded terminal result with output profile
   `scientific_acceptance_v1` may carry `tepp.scientific_acceptance.v1`;
3. RMSE, bias, both standard errors, coverage, Wilson bounds, temporal-order,
   SE-gate, and nested report keys on a receipt fail closed;
4. a scientific-acceptance profile without the artifact, a failed terminal with
   the artifact, a digest mismatch, an all-zero or `run_id`-mismatched binding
   digest, a model that does not match the request, a future or malformed
   cutoff, negative RMSE/SEs, out-of-range coverage, inverted Wilson bounds,
   or an `se_gate_accepted` flag inconsistent with `|RMSE| ≤ k · SE(RMSE)`
   fail closed.

`analysis_engine` library binding remains on live PR #356. PostgreSQL
persistence and Compose recovery remain GAP-003B. This slice does not promote
an ADR 0014 claim authority.

## Authoritative sources

National Academies of Sciences, Engineering, and Medicine. (2019).
*Reproducibility and replicability in science*. The National Academies Press.
https://doi.org/10.17226/25303

Peng, R. D. (2011). Reproducible research in computational science.
*Science, 334*(6060), 1226–1227. https://doi.org/10.1126/science.1213847

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values:
Context, process, and purpose. *The American Statistician, 70*(2), 129–133.
https://doi.org/10.1080/00031305.2016.1154108

National Institute of Standards and Technology. (2015). *Secure Hash Standard
(SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4

## Application

Peng (2011) and the National Academies (2019) require computational
reproducibility to bind identities without treating a receipt as a scientific
claim. Wasserstein and Lazar (2016) refuse to treat a numeric threshold as
automatic authority, so TEPP keeps RMSE/bias/coverage off
`AnalysisRunRequest` / `AnalysisRunAccepted` and allows those fields only on a
digest-bound terminal artifact. FIPS 180-4 SHA-256 detects whether the artifact
bytes agree with `result_sha256` (National Institute of Standards and
Technology, 2015).

## Verification

- metric keys on request or accepted JSON fail closed, including both
  standard errors, Wilson upper, and temporal-order accuracy;
- accepted/running status cannot carry a terminal artifact;
- `succeeded_scientific_acceptance` round-trips with matching digest and
  schema;
- profile mismatch, missing artifact, failed-terminal artifact, identity
  mismatch, model mismatch, future cutoff, impossible metrics, inconsistent
  SE-gate flag, and digest tamper fail closed.
