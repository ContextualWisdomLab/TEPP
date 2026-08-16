# naruon HTTP interchange (doctoring)

## Scope

naruon may submit analysis-run requests and request purpose-bound exports only
through versioned `https` POST paths owned by TEPP. HTTP method, path, and
header semantics for that interchange follow HTTP/1.1 (Fielding & Reschke,
2014). Fail-closed refusal of table-access URLs, review/Copilot credential
headers, reserved-header redefinition, principal-only idempotency keys, and
lexical TEPP inference claims is repository contract authority (see Internal
contract evidence), not an RFC inference rule.

A loopback-only live HTTP/1.1 listener (`NaruonLiveService`) now serves the
same POST paths for local and standalone proof. It is not TLS termination and
does not bind non-loopback addresses. Persistence remains TEPP-owned; naruon
never migrates or queries TEPP application tables. Purpose-bound export
disclosure and privacy-management readiness map to published privacy guidance
(ISO/IEC, 2019; National Institute of Standards and Technology, 2020) without
claiming certification.

## Authority

### External standards (HTTP and privacy claims only)

Fielding, R. T., & Reschke, J. (Eds.). (2014). *Hypertext Transfer Protocol
(HTTP/1.1): Semantics and content* (RFC 7231). IETF.
https://doi.org/10.17487/RFC7231

ISO/IEC. (2019). *ISO/IEC 27701:2019 Security techniques — Extension to
ISO/IEC 27001 and ISO/IEC 27002 for privacy information management —
Requirements and guidelines*. International Organization for Standardization.

National Institute of Standards and Technology. (2020). *NIST Privacy
Framework: A tool for improving privacy through enterprise risk management*
(Version 1.0). U.S. Department of Commerce.
https://doi.org/10.6028/NIST.CSWP.01162020

### Internal contract evidence

- `docs/API_CONTRACT.md` — versioned analysis-run and export surfaces
- `docs/adr/0011-standalone-modular-msa-boundary.md` — no cross-service table access
- `crates/tepp_api/tests/naruon_http_contract.rs` — fail-closed interchange proofs
- `crates/tepp_api/tests/naruon_live_http_contract.rs` — loopback live HTTP/1.1 proofs

## Verification

- committed naruon example builds `POST /v1/analysis-runs` without credentials;
- `postgres` / `jdbc` / `/sql` / `/tables/` and non-`https` origins fail closed;
- review, Copilot, and bearer headers are `AuthorizationDenied`;
- reserved standard headers cannot be redefined via extra headers;
- export interchange requires `ModularServiceConsumer` and a per-export
  idempotency key distinct from `principal_id` alone;
- `tfidf` / `bm25` / `keyword` cannot claim TEPP inference;
- loopback `NaruonLiveService` accepts valid POSTs and refuses non-loopback
  binds, table-access hosts, credential headers, and conflicting idempotency.
