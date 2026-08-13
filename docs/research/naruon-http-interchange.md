# naruon HTTP interchange (doctoring)

## Scope

naruon may submit analysis-run requests and request purpose-bound exports only
through versioned `https` POST paths owned by TEPP. This slice adds the
fail-closed interchange builder so a consumer cannot aim at application tables,
carry review or Copilot credentials, or claim TEPP topic inference from a
lexical heuristic (Fielding & Reschke, 2014).

This is not a live HTTP server. Persistence remains TEPP-owned; naruon never
migrates or queries TEPP tables (ISO/IEC, 2019).

## Authority

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

## Verification

- committed naruon example builds `POST /v1/analysis-runs` without credentials;
- `postgres` / `jdbc` / `/sql` / `/tables/` and non-`https` origins fail closed;
- review, Copilot, and bearer headers are `AuthorizationDenied`;
- export interchange requires `ModularServiceConsumer`;
- `tfidf` / `bm25` / `keyword` cannot claim TEPP inference.
