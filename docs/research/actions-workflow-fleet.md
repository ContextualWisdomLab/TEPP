# Actions workflow fleet audit (doctoring)

## Scope

Temporary bootstrap, diagnosis, and repair workflows can be deleted from the
repository tree while their independent GitHub Actions registry records remain
`active`. TEPP treats those orphan identities as a control-plane supply-chain
defect: tree-level CI cannot observe them, and name-only heuristics must not
disable current production automation.

`scripts/actions_workflow_fleet.py` binds every inventory to the exact
protected default-branch SHA and tree, paginates the complete workflow list,
classifies present/orphan/disabled/GitHub-dynamic identities, and disables
only re-fetched active orphans. GitHub's disable endpoint sets
`disabled_manually`, not a bare `disabled` token (GitHub, n.d.-a).

## Authority

GitHub. (n.d.-a). *REST API endpoints for workflows*. GitHub Docs. Retrieved
August 13, 2026, from
https://docs.github.com/en/rest/actions/workflows

GitHub. (n.d.-b). *Using pagination in the REST API*. GitHub Docs. Retrieved
August 13, 2026, from
https://docs.github.com/en/rest/using-the-rest-api/using-pagination-in-the-rest-api

GitHub. (n.d.-c). *Disabling and enabling a workflow*. GitHub Docs. Retrieved
August 13, 2026, from
https://docs.github.com/en/actions/how-tos/manage-workflow-runs/disable-and-enable-workflows

OpenSSF. (2023). *Supply-chain Levels for Software Artifacts (SLSA)
specification, version 1.0*. https://slsa.dev/spec/v1.0/

OWASP Foundation. (2023). *OWASP Top 10 CI/CD Security Risks*
(CICD-SEC-4: Poisoned Pipeline Execution).
https://owasp.org/www-project-top-10-ci-cd-security-risks/

The REST workflow catalog is independent of the git tree (GitHub, n.d.-a).
Link-header pagination must be consumed to completion or the listing is
truncated (GitHub, n.d.-b). Disable is an explicit control-plane mutation
that requires write authority and leaves a disabled registry identity rather
than deleting history (GitHub, n.d.-c). SLSA and OWASP CI/CD guidance treat
unreviewed pipeline identities as a supply-chain integrity risk (OpenSSF,
2023; OWASP Foundation, 2023).

## Verification

- unit tests for path encoding/case, pagination receipts, 403/404/5xx
  fail-closed reasons, protected-path refusal, branch-movement CAS,
  identity-change refusal, and official `disabled_manually` confirmation;
- Coverage.py 7.15.2 100% statement and branch coverage for the auditor;
- live read-only inventory against ContextualWisdomLab/TEPP bound to the
  current default-branch SHA.
