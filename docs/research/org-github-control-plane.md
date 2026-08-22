# Organization `.github` reusable-workflow interchange

## Scope

This note doctors the `tepp_api` org-central `.github` control-plane binding:

1. reusable workflow identities bind only as CI/review/security authority;
2. table, JDBC, SQL, and `postgres` targets fail closed;
3. `COPILOT_GITHUB_TOKEN` and other GitHub/review-agent secret names are refused;
4. `NVIDIA_NIM_API_KEY` is the only allowed model-credential name;
5. GitHub Check conclusions cannot become scientific or implemented-main claims.

This is a versioned identity/authority gate, not a live reusable-workflow dispatcher. No database migration is allocated.

## Authoritative sources

GitHub. (n.d.). *Reusing workflows*. GitHub Docs. Retrieved August 13, 2026, from https://docs.github.com/en/actions/using-workflows/reusing-workflows

GitHub. (n.d.). *REST API endpoints for workflows*. GitHub Docs. Retrieved August 13, 2026, from https://docs.github.com/en/rest/actions/workflows

ISO/IEC. (2019). *ISO/IEC 27701:2019 Security techniques — Extension to ISO/IEC 27001 and ISO/IEC 27002 for privacy information management — Requirements and guidelines*. International Organization for Standardization.

National Institute of Standards and Technology. (2020). *NIST Privacy Framework: A tool for improving privacy through enterprise risk management* (Version 1.0). U.S. Department of Commerce. https://doi.org/10.6028/NIST.CSWP.01162020

## Application

Reusable workflows are a versioned control-plane composition mechanism, not a scientific estimator (GitHub, n.d.). ISO/IEC 27701 and the NIST Privacy Framework require purpose-bound, minimized disclosure and forbid using review-agent credentials as a product-development path (ISO/IEC, 2019; National Institute of Standards and Technology, 2020). TEPP therefore binds org `.github` identities to `CiReviewSecurity` and refuses Check conclusions as claim-promotion evidence.

## Verification

- a valid `ContextualWisdomLab/.github/.../workflows/*.yml@ref` binds as `CiReviewSecurity`;
- `postgres`, `jdbc`, `sql`, `tables`, and empty identities are denied;
- `COPILOT_GITHUB_TOKEN` and `review-agent-github-token` are denied;
- `NVIDIA_NIM_API_KEY` is an allowed secret name;
- `refuse_check_conclusion_as_scientific_claim` always denies nonempty conclusions.
