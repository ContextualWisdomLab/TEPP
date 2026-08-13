# Provider payload minimization and purpose-bound re-identification

## Scope

This note doctors the `tepp_api` provider-payload adapter that implements ADR 0009 without a new database migration:

1. a time-bounded `PurposeGrant` is evaluated at a decision instant (`YYYY-MM-DDTHH:MM:SSZ`); expired and not-yet-valid grants fail closed;
2. model-provider payloads keep opaque analytical identifiers and membership roles so multilevel measurement is not destroyed by blanket masking;
3. free-text source bodies follow the existing purpose-bound export gate;
4. direct identity mappings never enter a provider payload or an ordinary disclosure log, even when a separate re-identification flag is set;
5. re-identification is a distinct elevated path limited to scientific validation with an explicit grant flag and matching tenant.

HTTP posting to NVIDIA NIM, naruon, or contextual-orchestrator remains a later connector slice. This adapter is the fail-closed payload contract those connectors must call.

## Authoritative sources

ISO/IEC. (2025). *ISO/IEC 27701:2025 Information security, cybersecurity and privacy protection — Privacy information management systems — Requirements and guidance*. International Organization for Standardization.

National Institute of Standards and Technology. (2020). *NIST Privacy Framework: A tool for improving privacy through enterprise risk management* (Version 1.0). U.S. Department of Commerce. https://doi.org/10.6028/NIST.CSWP.01162020

ISO/IEC. (2019). *ISO/IEC 27701:2019 Security techniques — Extension to ISO/IEC 27001 and ISO/IEC 27002 for privacy information management — Requirements and guidelines*. International Organization for Standardization.

## Application

ISO/IEC 27701:2025 is the current standalone Privacy Information Management System standard and is cited for purpose limitation and disclosure minimization (ISO/IEC, 2025). The 2019 edition remains recorded because earlier TEPP doctoring referenced it as an extension to ISO/IEC 27001 (ISO/IEC, 2019). The NIST Privacy Framework supplies the Core functions (Identify-P, Control-P, Communicate-P) used to separate provider disclosure from re-identification and to keep logs free of source bodies (National Institute of Standards and Technology, 2020). These citations are readiness mappings, not certification or legal sufficiency.

## Verification

- scientific payloads retain opaque IDs, roles, and authorized source text;
- operational/partner source-text offers are denied;
- expired, not-yet-valid, inverted, and cross-tenant grants fail closed;
- attached identity mappings are refused on the provider path;
- elevated scientific re-identification returns the mapping; other purposes and missing flags are denied;
- disclosure logs never contain source text or mapping strings.
