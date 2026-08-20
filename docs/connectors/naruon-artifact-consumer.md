# naruon modular consumer contract for TEPP artifacts

**Status:** Partial — versioned DTO plus HTTP interchange on the active PR; live HTTP service remaining
**Last reviewed:** 2026-08-13

## Boundary

`naruon` may consume versioned TEPP analytical artifacts and submit evidence/analysis requests through published wire contracts. It must not:

- read TEPP application tables directly;
- treat lexical heuristics as TEPP topic inference;
- rewrite TEPP scientific acceptance criteria or knowledge cutoffs;
- receive model credentials belonging to TEPP review or merge authorities.

TEPP remains the scientific authority for estimation, recovery metrics, temporal eligibility, and purpose-bound export decisions (ADR 0011; `docs/API_CONTRACT.md`).

## Allowed consumption surfaces

| Surface | Contract | Direction |
|---|---|---|
| analysis-run create | `tepp_api` `AnalysisRunRequest` v1 | naruon → TEPP |
| reproducibility binding | `tepp_api` `ReproducibilityManifest` v1 | TEPP → naruon |
| JSON-LD export envelope | `tepp_api` `JsonLdExport` v1 | TEPP → naruon |
| GraphML relation export | `tepp_api` `GraphMlExport` | TEPP → naruon |
| purpose-bound export auth | `tepp_api` `authorize_export` with `ModularServiceConsumer` | TEPP gate |
| HTTP analysis-run create | `tepp_api` `naruon_analysis_run_exchange` → `POST /v1/analysis-runs` | naruon → TEPP |
| HTTP export authorize | `tepp_api` `naruon_export_exchange` → `POST /v1/exports` | naruon → TEPP |

Committed examples live under `examples/`. Schema for analysis-run requests lives under `schemas/analysis_run_request_v1.json`.

## Purpose-bound disclosure

When naruon requests an export, TEPP evaluates `AnalyticalPurpose::ModularServiceConsumer`. Free-text source bodies remain optional and purpose-gated; opaque analytical identifiers and membership/relation structure must not be blanket-masked when required for multilevel measurement (ADR 0009).

## Failure modes

- unknown wire fields → reject;
- unsupported contract version → reject;
- knowledge cutoff / availability violations → reject in TEPP domain crates;
- authorization deny → `authorization_denied` envelope without policy leakage;
- `postgres` / `jdbc` / `/sql` / `/tables/` or non-`https` origins → reject;
- review, Copilot, or bearer credential headers → reject;
- redefinition of reserved headers (`content-type`, `tepp-consumer`,
  `tepp-contract-version`, `idempotency-key`) via extra headers → reject;
- export interchange without a nonempty per-export idempotency key → reject;
- lexical method codes (`tfidf`, `bm25`, `keyword`) claiming TEPP inference → reject.

## Authority sources

Fielding, R. T., & Reschke, J. (Eds.). (2014). *Hypertext Transfer Protocol (HTTP/1.1): Semantics and content* (RFC 7231). IETF. https://doi.org/10.17487/RFC7231

ISO/IEC. (2019). *ISO/IEC 27701:2019 Security techniques — Extension to ISO/IEC 27001 and ISO/IEC 27002 for privacy information management — Requirements and guidelines*. International Organization for Standardization.

National Institute of Standards and Technology. (2020). *NIST Privacy Framework: A tool for improving privacy through enterprise risk management* (Version 1.0). U.S. Department of Commerce. https://doi.org/10.6028/NIST.CSWP.01162020
