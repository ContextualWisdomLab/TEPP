# Organization `.github` control-plane contract for TEPP

**Status:** Accepted-target modular integration contract; reusable-workflow bindings are on the active PR  
**Last reviewed:** 2026-08-13

## Boundary

Organization `.github` reusable workflows own CI, review, security, and release-control functions only (ADR 0011; ADR 0015). They must not:

- become runtime scientific authority or promote implemented-main / recovery claims (ADR 0014);
- read or write TEPP application tables;
- receive `COPILOT_GITHUB_TOKEN` or existing independent review-agent credentials as a product-development path;
- replace deterministic TEPP validation with a GitHub Check conclusion.

TEPP remains the scientific authority for estimation, recovery metrics, temporal eligibility, and purpose-bound export decisions.

## Allowed control-plane surfaces

| Surface | Contract | Direction |
|---|---|---|
| reusable workflow identity | `tepp_api` `bind_org_github_workflow` | org `.github` → TEPP binding |
| authority class | `OrgWorkflowAuthority::CiReviewSecurity` | TEPP gate |
| Check conclusion | `refuse_check_conclusion_as_scientific_claim` | TEPP gate |
| secret names | `refuse_org_workflow_secret` | TEPP gate |

`NVIDIA_NIM_API_KEY` is the only allowed model-credential name for product-development workflows. Live reusable-workflow dispatch remains accepted-target.

## Purpose-bound disclosure

Control-plane workflows receive repository metadata, check identities, and SBOM/provenance artifacts they are authorized to process. They do not receive application-table credentials or identity-mapping stores (ADR 0009).

## Failure modes

- empty workflow identity → reject;
- table, JDBC, SQL, or `postgres` targets → reject;
- Copilot / GitHub / review-agent secret names → reject;
- any Check conclusion used as scientific acceptance → reject.

## Authority sources

GitHub. (n.d.). *Reusing workflows*. GitHub Docs. Retrieved August 13, 2026, from https://docs.github.com/en/actions/using-workflows/reusing-workflows

Fielding, R. T., & Reschke, J. (Eds.). (2014). *Hypertext Transfer Protocol (HTTP/1.1): Semantics and content* (RFC 7231). IETF. https://doi.org/10.17487/RFC7231

ISO/IEC. (2019). *ISO/IEC 27701:2019 Security techniques — Extension to ISO/IEC 27001 and ISO/IEC 27002 for privacy information management — Requirements and guidelines*. International Organization for Standardization.
