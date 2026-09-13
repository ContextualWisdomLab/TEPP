# Hourly Contextual Orchestrator Product Development

The hourly product-development workflow proposes one bounded commercial-quality increment only when the repository has no open pull request or open issue. It is separate from deterministic quality gates and never merges, releases, deploys, approves, or changes reviewer credentials.

## Schedule and queue behavior

`.github/workflows/hourly-nim-product-development.yml` runs at minute 47 of every hour and also supports `workflow_dispatch` with `dry_run=true`. Its repository-scoped concurrency group does not cancel an active run.

Before checkout or model execution, the proposal job reads at most one open pull request and one open issue. Unreadable inventory, any open PR or issue, an unavailable immutable contextual-orchestrator release, an unavailable released gateway, or missing Maintainer App configuration produces a stable fail-closed no-op. A dry run may print the task contract without model or publication credentials.

When a PR or issue exists, normal review → repair → exact-head Checks → merge governance owns the hour. The scheduler does not create a competing branch.

The live queue is intentionally not duplicated here. `docs/product-technical-gap-baseline.md` and the queue-authority vehicle hold point-in-time queue evidence; GitHub is authoritative for current counts and exact heads.

## Required repository configuration

Configure these repository or organization values:

- variable `CONTEXTUAL_ORCHESTRATOR_RELEASE`: an immutable published contextual-orchestrator release tag;
- variable `CONTEXTUAL_ORCHESTRATOR_BASE_URL`: the HTTPS base URL for the released gateway;
- secret `CONTEXTUAL_ORCHESTRATOR_GATEWAY_TOKEN`: the gateway credential used by the proposal runner;
- variable `TEPP_MAINTAINER_APP_CLIENT_ID`;
- secret `TEPP_MAINTAINER_APP_PRIVATE_KEY`;
- a repository-scoped GitHub App installation with metadata read, contents write, and pull-request write permissions only.

TEPP does not configure or consume provider API credentials for this workflow. Provider discovery, provider/model/group selection, free/paid admission, fallback, and provider execution remain contextual-orchestrator responsibilities. The proposal job requests only `orchestrator/free`.

Do not place GitHub App credentials in the proposal or verifier jobs. Do not reuse the existing review App or alter its variable, secret, identity, or route. A manual dry run verifies scheduling, queue, and prompt contracts without model or publication credentials.

## Three-runner trust boundary

### 1. Proposal runner

The proposal runner has read-only repository, issue, and pull-request permissions. OpenCode is downloaded from an immutable versioned URL and checked against the committed SHA-256.

Before OpenCode runs, the workflow resolves `CONTEXTUAL_ORCHESTRATOR_RELEASE` with the GitHub Releases API and requires the matching release to be neither draft nor prerelease and to report `.immutable == true`. A mutable protected-main commit, open PR head, source archive, or checksum-pinned snapshot is not a production routing contract.

The configured gateway URL must be HTTPS. The runner checks gateway liveness over HTTPS and makes an authenticated `/v1/models` request using only the gateway credential. The returned catalog must expose `orchestrator/free`; otherwise the workflow fails closed with `contextual_orchestrator_gateway_unavailable`. The workflow does not start a loopback gateway and does not perform local provider discovery, price ranking, or model selection.

OpenCode receives only the gateway credential through its OpenAI-compatible adapter and requests `contextual-orchestrator/orchestrator/free`. Network tools, GitHub CLI, remote Git operations, commits, pushes, tags, external-directory access, task delegation, interactive questions, and OpenCode web tools remain denied by the generated configuration.

There is no model/reasoning elapsed-time kill switch in the OpenCode command. The GitHub proposal job retains an explicit 55-minute administrative job budget. That administrative limit is distinct from user cancellation, provider termination, streaming/tool-call completion, and model reasoning duration.

The model may edit the local working tree and run repository tests. A trusted step stages the complete proposal, rejects whitespace errors, symbolic links, gitlinks, excessive file count, and excessive patch bytes, then uploads one binary full-index patch with one-day retention.

### 2. Fresh verifier

A new runner checks out the exact proposal base SHA and downloads the artifact by immutable numeric ID. It validates the upload digest, patch SHA-256, workflow-run identity, expiration, base SHA, file count, byte count, and Git modes before applying the patch.

The verifier receives neither model nor publication credentials. It installs the hash-locked Python environment, compiles quality scripts and tests, validates workspace and documentation contracts, runs quality tests with 100% statement and branch coverage, and executes pinned Rust format, lint, test, documentation, dependency, line-coverage, and branch-coverage gates. Coverage artifacts remain under `$RUNNER_TEMP`. It rejects any verification mutation and proves the post-verification patch is byte-identical.

### 3. Fresh publisher

A third runner checks out the exact base and copies the trusted PR-message parser to `RUNNER_TEMP` before applying the proposal. It repeats immutable artifact checks and applies the patch only as Git data. It executes no proposed tests, build scripts, packages, binaries, or shell files.

The copied parser rejects symlinks, non-regular files, malformed UTF-8, unsupported controls, bidirectional spoofing, and byte-limit violations. Only after bounded metadata is written does the publisher mint the repository-scoped Maintainer App token. It then rechecks open-PR and open-issue inventory plus live `main`, pushes one unique branch, and calls `gh pr create` exactly once. The live default-branch SHA is compared with the same `EXPECTED_BASE` captured by the proposal runner; an advance fails with `base_branch_advanced`.

## Proposal contract

The autonomous prompt requires one buyer-visible gap, standalone and modular MSA compatibility, realistic test-first evidence, 100% coverage and docstrings, database naming policy, CHANGELOG and operations updates, and APA 7 doctoring. Semantic LLM work crosses the released contextual-orchestrator boundary and requests `orchestrator/free`; TEPP does not select provider/model/group or a paid fallback. Fugu, Conductor, and TRINITY remain experimental orchestration hypotheses evaluated through reasoning-effort and topology ablations rather than substitutes for deterministic scientific gates.

Operational logging preserves the existing audit boundary: privileged persistence inserts use `audit_event`; in-memory records are created only through `operational_log::try_record` / `OperationalLogRecord::new`; raw source content and source identity do not enter operational logs.

The model writes `PR_MESSAGE.md` with a bounded title on the first line and a body describing the product gap, design, RED-to-GREEN evidence, verification, sources, version decision, and residual risk. Missing metadata receives a minimal trusted fallback body.

## Failure and recovery

Stable no-op or refusal reasons include:

- `pull_request_inventory_unavailable`;
- `open_pull_request`;
- `issue_inventory_unavailable`;
- `open_issue`;
- `contextual_orchestrator_release_unavailable`;
- `contextual_orchestrator_gateway_unavailable`;
- `maintainer_app_unavailable`;
- `base_branch_advanced`;
- `open_pull_request_after_generation`;
- `open_issue_after_generation`.

There is no consumer-side provider, paid, unknown-price, or mutable-source fallback. If contextual-orchestrator has no compatible immutable release, scheduled semantic execution is intentionally unavailable until the owner publishes one and TEPP adopts its released contract.

A failed verifier publishes nothing. A publisher aborts if the artifact, base, queue, or metadata changed. If branch push succeeds but PR creation fails, the error trap removes the orphan branch.

Investigate the exact run and job log, reproduce the relevant command on the same commit, add or retain a failing regression, and repair through a normal PR. Never bypass the verifier, substitute stale check evidence, grant the model a write token, or reintroduce consumer-side routing to make the schedule run.

## Disablement and rollback

Disable scheduled development by disabling the workflow or removing its schedule. Removing the released-contract tag, HTTPS gateway URL, gateway credential, or Maintainer App configuration leaves production execution fail-closed. The deterministic quality sentinel continues independently.

Rollback a faulty workflow through a reviewed revert PR. Do not edit branch protection, review workflows, release workflows, released-contract validation, or fail-closed routing as an incident shortcut. Delete orphan `nim-agent/product-dev-*` branches only after confirming no open PR references them.

## Residual risks

- Gateway availability and the correctness of `orchestrator/free` admission are external owner responsibilities; TEPP validates release identity and route presence but cannot infer owner-side provider economics from consumer metadata.
- Each orchestrator-selected provider may process repository source. Operators must review confidentiality, retention, regional, and contractual obligations before enabling the released gateway.
- The verifier executes untrusted code on an ephemeral hosted runner with outbound network access, but receives no publication, gateway, OIDC, artifact/cache runtime, command-file, or reviewer credential.
- GitHub artifact storage, hosted runners, pinned actions, release metadata, and the configured HTTPS gateway remain trusted infrastructure. Digests and immutable-release metadata establish identity, not semantic correctness.
- GitHub cannot atomically create a PR only when none exists. Final queue and base revalidation, unique branches, review, and exact-head Checks bound that race.
