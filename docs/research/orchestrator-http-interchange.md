# contextual-orchestrator HTTPS interchange

## Scope

This note doctors the `tepp_api` interpretation-port builder for `contextual-orchestrator`:

1. requests are `POST https://<host>/v1/interpretation-runs` with no credentials;
2. hosts that look like table or SQL access fail closed;
3. `COPILOT_GITHUB_TOKEN` and other GitHub/review-agent secret names are refused;
4. `NVIDIA_NIM_API_KEY` is the only allowed model-credential name;
5. orchestrator output cannot become scientific acceptance.

This is a versioned request builder, not a live HTTP server. No database migration is allocated.

## Authoritative sources

Fielding, R. T., & Reschke, J. (Eds.). (2014). *Hypertext Transfer Protocol (HTTP/1.1): Semantics and content* (RFC 7231). IETF. https://doi.org/10.17487/RFC7231

ISO/IEC. (2019). *ISO/IEC 27701:2019 Security techniques — Extension to ISO/IEC 27001 and ISO/IEC 27002 for privacy information management — Requirements and guidelines*. International Organization for Standardization.

National Institute of Standards and Technology. (2020). *NIST Privacy Framework: A tool for improving privacy through enterprise risk management* (Version 1.0). U.S. Department of Commerce. https://doi.org/10.6028/NIST.CSWP.01162020

## Application

RFC 7231 supplies POST semantics for an interpretation-run resource (Fielding & Reschke, 2014). ISO/IEC 27701 and the NIST Privacy Framework require purpose-bound, minimized disclosure and forbid using review-agent credentials as a product-development path (ISO/IEC, 2019; National Institute of Standards and Technology, 2020). TEPP therefore keeps scientific authority inside deterministic gates and treats the orchestrator as an untrusted interpreter.

## Verification

- a valid host yields `POST` without authorization/cookie/token/copilot/github headers;
- `postgres`, `jdbc`, `sql`, `tables`, empty, and punctured hosts are denied;
- `COPILOT_GITHUB_TOKEN` and `review-agent-github-token` are denied;
- `NVIDIA_NIM_API_KEY` is an allowed secret name;
- `refuse_orchestrator_as_scientific_acceptance` always denies.
