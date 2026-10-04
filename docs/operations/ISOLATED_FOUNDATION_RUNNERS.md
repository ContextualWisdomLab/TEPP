# Isolated Foundation CI: source route and runtime hold

**Status: active-PR / source-only / Draft HOLD.** This routing correction changes neither the approved PRD v0.4 product/measurement target nor scientific acceptance. No runner is provisioned or attested by this file.

## Exact consumer scope

Repository `ContextualWisdomLab/TEPP` requests the canonical group `CWL CI isolated` with `[self-hosted, linux, x64, cwlab-ci-isolated]` for:

| Workflow | Jobs | Workload |
|---|---|---|
| `.github/workflows/ci.yml` | `repository-contracts`, `rust-quality`, `production-coverage`, `live-postgres` | Existing Python/Rust quality, complete coverage, SBOM/provenance and disposable PostgreSQL16 tests |
| `.github/workflows/docs-quality.yml` | `validate` | Existing documentation and whitespace checks |

Current events remain normal `pull_request`, `push` to main, and existing manual dispatch. Default permissions remain `contents: read`, and checkout remains `persist-credentials: false`. No `pull_request_target`, workflow secrets, label-only selector or fallback to control/scanner/GPU runners is introduced. The source commands, tool pins, action pins, coverage thresholds and CI cache paths are preserved exactly.

The credential-bearing `.github/workflows/hourly-nim-product-development.yml` is **unchanged**. The remaining three hosted declarations are outside this bounded non-secret validation slice. The original all-self-hosted objective remains unfinished: the hourly NIM/App-secret execution and generated proposal verification must acquire their own trusted/disposable split via the existing central owner, not inherit a public-validation group.

## Provisioning and authority

- Existing central source routing: `ContextualWisdomLab/.github#2565`, inspected head `ab0c865989012c88d2c10c717f6649a76ea49e27`; Draft/HOLD, not shipped.
- Existing operator-owned provisioning/access: `ContextualWisdomLab/linux-cluster-ops#326`.
- Isolation authority: `ContextualWisdomLab/quarantine-sandbox-runtime#136` / PR137, inspected head `fdb1f3543044507f9fc4dd4a087afe96a4cc5ca3`; intentional RED, not activated.

The October4 inventory has no `CWL CI isolated` group; Default has no runners. Restricted control/scanner groups must not be borrowed, moved, relabeled or widened to drain the queue. New public/untrusted work remains prohibited until operator-owned evidence satisfies the existing isolation contract.

## Mandatory execution admission and completion

The existing operator must bind minimum repository/workflow/event/ref/fork access and return actual disposable Linux X64 runner-image/tool readiness. Required evidence includes dedicated VM under the approved `/data` boundary, kernel-enforced denial of host/LAN/private-network access, no inherited credentials/privileged sockets, fresh per-job workspace/HOME/tool state, bounded cleanup/reset, immutable image identity and actual exact-head GitHub canary plus post-job cleanup. Point reachability failures alone are not whole-CIDR enforcement proof.

Existing `~/.cargo/bin` cache restoration must remain separated across untrusted/trusted runs; process cleanup and a fresh HOME alone do not attest cache trust. PostgreSQL service Docker remains inside the disposable VM; no host Docker socket or central inference service is exposed. Missing sudo/APT/Python/Rust/Docker capability must fail setup rather than bypass a gate. Operator attestation, group naming, labels, local YAML tests and actionlint each prove different things; none by itself permits execution or merge.

Current PR755/757 jobs failed before runner allocation with billing-lock annotations, not executed test failures. This source change removes the hosted selector only in these five jobs, but missing isolated group/capacity remains an explicit fail-closed admission condition. Keep the rollout Draft; normal exact-head checks and qualifying independent approval are required before protected-main integration. Do not rerun unchanged old workflow generations, purchase capacity, self-approve or bypass requirements.

## Primary references (APA 7)

GitHub. (n.d.-a). *Managing access to self-hosted runners using groups*. GitHub Docs. Retrieved October4, 2026, from https://docs.github.com/en/actions/how-tos/manage-runners/self-hosted-runners/manage-access

GitHub. (n.d.-b). *Secure use reference*. GitHub Docs. Retrieved October4, 2026, from https://docs.github.com/en/actions/reference/security/secure-use

Actual parent web.run retrieved both pages during this source-preparation turn. Retrieval chronology is not backdated to precede the initial draft. Group access policies and public-repository untrusted-code risk support the boundary rationale; they are not TEPP runner registration or isolation evidence. The matching APA7 source note is `docs/research/isolated-foundation-runner-routing.md`.
