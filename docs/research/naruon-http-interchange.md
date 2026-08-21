# naruon HTTP interchange (doctoring)

## Scope

naruon may submit analysis-run requests and request purpose-bound exports only
through versioned `https` POST paths owned by TEPP. HTTP method, path, `Host`,
and `Transfer-Encoding` semantics follow current HTTP semantics (Fielding,
Nottingham, & Reschke, 2022). Knowledge-cutoff instants use RFC 3339
(Klyne & Newman, 2002). Fail-closed refusal of table-access URLs,
review/Copilot/NIM/proxy credential headers, reserved-header redefinition,
principal-only idempotency keys, and lexical TEPP inference claims is
repository contract authority (see Internal contract evidence), not an RFC
inference rule.

The live listener is loopback HTTP/1.1 with an installed read/write deadline.
It is not a production TLS/`$PORT` service. Persistence remains TEPP-owned;
naruon never migrates or queries TEPP application tables. Purpose-bound export
disclosure and privacy-management readiness map to published privacy guidance
(ISO/IEC, 2019; National Institute of Standards and Technology, 2020) without
claiming certification.

## Authority

### External standards (HTTP and privacy claims only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

Klyne, G., & Newman, C. (2002). *Date and time on the Internet: Timestamps*
(RFC 3339). IETF. https://doi.org/10.17487/RFC3339

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
- `crates/tepp_api/tests/naruon_live_http_contract.rs` — loopback TCP, deadline, Host, cutoff

## Verification

- committed naruon example builds `POST /v1/analysis-runs` without credentials;
- `postgres` / `jdbc` / `/sql` / `/tables/` and non-`https` origins fail closed;
- review, Copilot, NIM/NVIDIA, proxy-authorization, and bearer headers are
  `AuthorizationDenied`;
- reserved standard headers cannot be redefined via extra headers;
- live `Host` must be loopback; `Transfer-Encoding` is refused;
- `knowledge_cutoff` must be RFC 3339 and must not be after request receipt;
- analysis-run idempotency replay is keyed by tenant plus key;
- export interchange requires `ModularServiceConsumer` and a per-export
  idempotency key distinct from `principal_id` alone, proven over TCP;
- `tfidf` / `bm25` / `keyword` cannot claim TEPP inference.
