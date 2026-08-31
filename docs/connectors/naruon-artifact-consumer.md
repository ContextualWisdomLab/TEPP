# naruon modular consumer contract for TEPP artifacts

**Status:** Partial — versioned DTO and HTTP interchange are implemented-main; the loopback live listener is active-PR #157; production TLS/`$PORT` remaining
**Status:** Partial — versioned DTO and HTTP interchange are implemented-main at protected head `c45be17a9dbce95ef81cee230e9d128abc7160ac`; the loopback live listener and terminal-result contract are composed on the active product branch; production TLS/`$PORT` remaining
**Last reviewed:** 2026-08-16

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
| corpus-split leakage audit | `tepp_api` `CorpusSplitManifest` v1 | TEPP → naruon |
| JSON-LD export envelope | `tepp_api` `JsonLdExport` v1 | TEPP → naruon |
| GraphML relation export | `tepp_api` `GraphMlExport` | TEPP → naruon |
| purpose-bound export auth | `tepp_api` `authorize_export` with `ModularServiceConsumer` | TEPP gate |
| HTTP analysis-run create | `tepp_api` `naruon_analysis_run_exchange` → `POST /v1/analysis-runs` | naruon → TEPP |
| HTTP analysis-run cancel | `tepp_api` `naruon_analysis_run_cancel_exchange` → `POST /v1/analysis-runs/{run_id}/cancel` | naruon → TEPP |
| HTTP analysis-run cancel (LineageWeave) | `tepp_api` `lineageweave_analysis_run_cancel_exchange` → `POST /v1/analysis-runs/{run_id}/cancel` | lineageweave → TEPP |
| HTTP export authorize | `tepp_api` `naruon_export_exchange` → `POST /v1/exports` | naruon → TEPP |
| Live loopback POST | `tepp_api` `NaruonLiveService` → `POST /v1/analysis-runs`, `/v1/analysis-runs/{run_id}/cancel`, and `/v1/exports` | naruon → TEPP |

Committed examples live under `examples/`. Schemas for analysis-run requests and corpus-split manifests live under `schemas/`.

## Purpose-bound disclosure

When naruon requests an export, TEPP evaluates `AnalyticalPurpose::ModularServiceConsumer`. Free-text source bodies remain optional and purpose-gated; opaque analytical identifiers and membership/relation structure must not be blanket-masked when required for multilevel measurement (ADR 0009).

## Failure modes

- unknown wire fields → reject;
- unsupported contract version → reject;
- knowledge cutoff / availability violations → reject in TEPP domain crates;
- authorization deny → `authorization_denied` envelope without policy leakage;
- `postgres` / `jdbc` / `/sql` / `/tables/` or non-`https` origins → reject;
- review, Copilot, NIM/NVIDIA, proxy-authorization, or bearer credential headers → reject;
- non-loopback `Host` or `Transfer-Encoding` on the live listener → reject;
- non-RFC 3339 or future-dated `knowledge_cutoff` → reject;
- redefinition of reserved headers (`content-type`, `tepp-consumer`,
  `tepp-contract-version`, `idempotency-key`) via extra headers → reject;
- export interchange without a nonempty per-export idempotency key → reject;
- lexical method codes (`tfidf`, `bm25`, `keyword`) claiming TEPP inference → reject;
- scientific-metric keys (`rmse`, `bias`, `coverage`, `se_gate`, `scientific_acceptance`, `report`) on a cancel body → reject;
- cancel of a succeeded, failed, or unknown analysis run → reject.

## Authority sources

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics* (RFC 9110). IETF. https://doi.org/10.17487/RFC9110

Klyne, G., & Newman, C. (2002). *Date and time on the Internet: Timestamps* (RFC 3339). IETF. https://doi.org/10.17487/RFC3339

ISO/IEC. (2019). *ISO/IEC 27701:2019 Security techniques — Extension to ISO/IEC 27001 and ISO/IEC 27002 for privacy information management — Requirements and guidelines*. International Organization for Standardization.

National Institute of Standards and Technology. (2020). *NIST Privacy Framework: A tool for improving privacy through enterprise risk management* (Version 1.0). U.S. Department of Commerce. https://doi.org/10.6028/NIST.CSWP.01162020
