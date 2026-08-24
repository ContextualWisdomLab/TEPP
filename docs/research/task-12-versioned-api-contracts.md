# Task 12 — Versioned service/API contracts and exports

## Scope

Task 12 introduces fail-closed versioned wire contracts in `tepp_api` for standalone and modular CWL composition:

1. analysis-run request/accepted response DTOs with contract version, idempotency, tenant workspace, snapshot, knowledge cutoff, model contract, and output profile;
2. content-redacting error envelopes with stable error codes;
3. reproducibility manifests;
4. JSON-LD export envelopes;
5. GraphML export rendering with XML escaping;
6. committed JSON Schema and example payloads under `schemas/` and `examples/`;
7. purpose-bound export authorization that preserves scientific identity linkages and refuses blanket PII masking;
8. corpus-split leakage-audit manifests that report cutoff exclusion counts, relation-component and partition digests, and a canonical digest bound to `corpus_split_manifest` without source text.
9. purpose-bound provider-payload minimization with expired-grant denial and separately authorized re-identification.

A loopback live HTTP/1.1 listener proves analysis-run and export POSTs. Production TLS/`$PORT` routing remains accepted-target. Domain estimation and persistence stay outside this crate except for the `corpus_split` adapter that materialises the leakage-audit DTO.

## Authoritative sources

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics* (RFC 9110). IETF. https://doi.org/10.17487/RFC9110

Klyne, G., & Newman, C. (2002). *Date and time on the Internet: Timestamps* (RFC 3339). IETF. https://doi.org/10.17487/RFC3339

Sporny, M., Longley, D., Kellogg, G., Lanthaler, M., Champin, P.-A., & Lindström, N. (2020). *JSON-LD 1.1* (W3C Recommendation). World Wide Web Consortium. https://www.w3.org/TR/json-ld11/

Brandes, U., Eiglsperger, M., Herman, I., Himsolt, M., & Marshall, M. S. (2002). GraphML progress report: Structural layer proposal. In *Graph Drawing* (pp. 501–512). Springer. https://doi.org/10.1007/3-540-45848-4_59

Wright, A., Andrews, H., Hutton, B., & Dennis, G. (2022). *JSON Schema: A media type for describing JSON documents* (Internet-Draft draft-bhutton-json-schema-01). IETF. https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-01

Tashman, L. J. (2000). Out-of-sample tests of forecasting accuracy: An analysis and review. *International Journal of Forecasting, 16*(4), 437–450. https://doi.org/10.1016/S0169-2070(00)00065-0

Kaufman, S., Rosset, S., Perlich, C., & Stitelman, O. (2012). Leakage in data mining: Formulation, detection, and avoidance. *ACM Transactions on Knowledge Discovery from Data, 6*(4), 1–21. https://doi.org/10.1145/2382577.2382579
ISO/IEC. (2025). *ISO/IEC 27701:2025 Information security, cybersecurity and privacy protection — Privacy information management systems — Requirements and guidance*. International Organization for Standardization.

ISO/IEC. (2019). *ISO/IEC 27701:2019 Security techniques — Extension to ISO/IEC 27001 and ISO/IEC 27002 for privacy information management — Requirements and guidelines*. International Organization for Standardization.

National Institute of Standards and Technology. (2020). *NIST Privacy Framework: A tool for improving privacy through enterprise risk management* (Version 1.0). U.S. Department of Commerce. https://doi.org/10.6028/NIST.CSWP.01162020

National Institute of Standards and Technology. (2015). *Secure Hash Standard (SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4

## Verification

- unit tests for unknown fields, unsupported versions, empty identities, byte limits, GraphML escaping, example payload parsing, cutoff exclusion counts, and relation-leakage refusal;
- unit tests for unknown fields, unsupported versions, empty identities, byte limits, GraphML escaping, example payload parsing, cutoff exclusion counts, relation-leakage refusal, expired/not-yet-valid/inverted/cross-tenant/impossible-calendar grant denial, provider mapping refusal, and audited elevated re-identification replay;
- workspace line and branch coverage gates must remain complete for production modules.
