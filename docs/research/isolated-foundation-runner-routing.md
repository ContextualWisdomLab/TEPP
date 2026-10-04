# Isolated Foundation runner routing rationale

## Scope

Approved PRD v0.4 §17 requires least-privilege execution and fail-closed scientific/security gates. This source-only correction changes five non-secret validation job selectors, not service/tenant authority, an estimator or product target. The dedicated group and capability labels match the current Draft central migration contract; runtime activation stays with linux-cluster-ops#326 and quarantine-sandbox-runtime#136/PR137.

## Primary-source grounding

GitHub (n.d.-a) describes runner groups as repository access policy boundaries and warns that public repository fork PRs can execute dangerous code on self-hosted machines. GitHub (n.d.-b) requires minimum token permissions and explains that self-hosted jobs lack a guaranteed clean ephemeral environment. Consequently, a mutable capability label or persist-credentials:false alone is not runtime isolation evidence. A disposable VM, per-job clean state, host/private-network denial, cache/workspace/HOME separation and least-privilege repository/workflow eligibility must be independently proved before activation.

The changed workflow events, read-only permissions, action/tool pins, cache paths and complete verification commands remain unchanged. Preserving existing caches does not certify cross-trust safety; that remains explicit operator admission work. Credential-bearing hourly NIM/App jobs are unchanged and cannot inherit this public-validation route. This partial conversion does not satisfy the original all-self-hosted objective.

## Verification limits

The selector regression initially failed in both workflow files (four Foundation jobs and one documentation job). Normal actionlint without a custom-label declaration failed; the canonical minimal label configuration is reused without wildcard/ignore rules. Source-contract tests, actionlint and independent review establish configuration correctness, not registered capacity or isolation. Current-head GitHub job execution, cleanup/canary, security and qualifying approvals remain required; billing failures and missing-group failures are non-execution states, not test passes.

## References (APA 7)

GitHub. (n.d.-a). *Managing access to self-hosted runners using groups*. GitHub Docs. Retrieved October 4, 2026, from https://docs.github.com/en/actions/how-tos/manage-runners/self-hosted-runners/manage-access

GitHub. (n.d.-b). *Secure use reference*. GitHub Docs. Retrieved October 4, 2026, from https://docs.github.com/en/actions/reference/security/secure-use

Both pages were actually returned by parent web.run during source preparation. This source retrieval is separate from local runtime execution and operator attestation. No prior failed/queued result is promoted by these references.
