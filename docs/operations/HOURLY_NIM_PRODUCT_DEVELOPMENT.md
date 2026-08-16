# Hourly NVIDIA NIM Product Development

The hourly NVIDIA NIM product-development workflow proposes one bounded
commercial-quality increment when the repository has no open pull request. It is
separate from the deterministic minute-17 quality sentinel and never merges,
releases, deploys, approves, or changes reviewer credentials.

## Schedule and queue behavior

`.github/workflows/hourly-nim-product-development.yml` runs at minute 47 of every
hour and supports `workflow_dispatch` with `dry_run=true`. The nonzero minute
avoids the busiest scheduler boundary. A repository-scoped concurrency group
does not cancel an active run.

Before checkout or model execution, the proposal job reads at most one open pull
request. Unreadable inventory, any open PR, a missing `NVIDIA_NIM_API_KEY`, or a
missing Maintainer App configuration produces a stable fail-closed no-op. A dry
run may print the task contract without either credential.

When a PR exists, normal review → repair → exact-head Checks → merge governance
owns the hour. The scheduler does not create a competing branch.

Current executable queue while drafts remain open:

1. Merge the predicted-versus-observed Allen coverage gate
   (`prediction_contradiction` / PR #97). `refuse_promotion` requires
   coverage. Keep PR #93 and PR #94 draft; their named `refuse_promotion`
   `Ok(())` path is not promotion authority.
2. Next buyer-visible slices, in order: naruon live HTTP loopback (PR #87),
   `text_segment` SQL contracts on existing migration `0006`, retention and
   legal-hold migration `0007` (PR #45), foundation known-truth recovery
   study, then CHRONOS forecast Brier calibration (PR #85).
3. Do not open a competing hourly proposal until the open-PR inventory is
   empty. Prefer reviewing, repairing, and merging the existing drafts.

## Required repository configuration

Configure these repository or organization values:

- Secret `NVIDIA_NIM_API_KEY` for the proposal runner only.
- Variable `TEPP_MAINTAINER_APP_CLIENT_ID`.
- Secret `TEPP_MAINTAINER_APP_PRIVATE_KEY`.
- A repository-scoped GitHub App installation with metadata read, contents
  write, and pull-request write permissions only.

Do not place GitHub App credentials in the proposal or verifier jobs. Do not
reuse the existing review App or alter its variable, secret, identity, or
provider route. Do not configure `COPILOT_GITHUB_TOKEN`.

A manual dry run verifies scheduling, queue, and prompt contracts without model
or publication credentials. Missing production credentials leave the hourly
developer disabled rather than falling back to `GITHUB_TOKEN`.

## Three-runner trust boundary

### 1. Proposal runner

The proposal runner has read-only repository and pull-request permissions.
OpenCode is downloaded from an immutable versioned URL and checked against a
committed SHA-256. Its only model provider is NVIDIA NIM.

The OpenCode process receives `NVIDIA_NIM_API_KEY` but has GitHub, OIDC, Actions
runtime/cache, and runner command-file variables removed. Network tools, GitHub
CLI, remote Git operations, commits, pushes, tags, external-directory access,
task delegation, interactive questions, and OpenCode web tools are denied.

The model may edit the local working tree and run repository tests. The trusted
step stages the complete proposal, rejects whitespace errors, symbolic links,
gitlinks, excessive file count, and excessive patch bytes, then uploads one
binary full-index patch with a one-day retention period.

### 2. Fresh verifier

A new runner checks out the exact base SHA and downloads by immutable numeric
artifact ID. It validates the upload digest, patch SHA-256, workflow-run
identity, expiration, base SHA, file count, byte count, and Git modes before
applying the patch.

The verifier receives neither model nor publication credentials. It installs
the hash-locked Python environment, compiles the quality scripts and tests,
validates the workspace and documentation contracts, runs the quality tests
with 100% statement and branch coverage, and executes the pinned Rust format,
lint, test, documentation, dependency, line-coverage, and branch-coverage
gates. Coverage artifacts remain under `$RUNNER_TEMP`. It rejects any
verification mutation and proves the post-verification patch is byte-identical.

### 3. Fresh publisher

A third runner checks out the exact base and copies the trusted PR-message parser
to `RUNNER_TEMP` before applying the proposal. It repeats the immutable artifact
checks and applies the patch only as Git data. It executes no proposed tests,
build scripts, packages, binaries, or shell files.

The copied parser rejects symlinks, non-regular files, malformed UTF-8,
unsupported controls, bidirectional spoofing, and byte-limit violations. Only
after bounded metadata is written does the publisher mint the repository-scoped
Maintainer App token. It then rechecks open-PR inventory and live `main`, pushes
one unique branch, and calls `gh pr create` exactly once.

## Proposal contract

The autonomous prompt requires one buyer-visible gap, standalone and modular MSA
compatibility, realistic test-first evidence, 100% coverage and docstrings,
database naming policy, CHANGELOG and operations updates, and APA 7 doctoring.
LLM work must use or improve Contextual Orchestrator and consider Fugu,
Conductor, TRINITY, workflow stages, access lists, bounded recursion,
role-specific reasoning effort, and ablation.

The model must write `PR_MESSAGE.md` with a bounded title on the first line and a
body describing the product gap, design, RED-to-GREEN evidence, verification,
sources, version decision, and residual risk. Missing metadata receives a
minimal trusted fallback body.

## Failure and recovery

Stable no-op reasons are:

- `pull_request_inventory_unavailable`
- `open_pull_request`
- `nim_api_key_unavailable`
- `maintainer_app_unavailable`

A failed model candidate is discarded before a later candidate runs. A cleanup
or reinstall failure stops fallback. A failed verifier publishes nothing. A
publisher aborts if the artifact, base, queue, or metadata changed. If branch
push succeeds but PR creation fails, the error trap removes the orphan branch.

Investigate the exact run and job log, reproduce the relevant command on the
same commit, add or retain a failing regression, and repair through a normal PR.
Never bypass the verifier, substitute stale check evidence, or grant the model a
write token.

## Disablement and rollback

Disable scheduled development by disabling the workflow, removing its schedule,
or removing either dedicated credential. Removing only the model secret stops
model execution; removing the Maintainer App values stops publication. The
minute-17 deterministic quality sentinel continues independently.

Rollback a faulty workflow through a reviewed revert PR. Do not edit branch
protection, review workflows, or release workflows as an incident shortcut.
Delete orphan `nim-agent/product-dev-*` branches only after confirming no open PR
references them.

## Residual risks

- The model process necessarily receives the NIM credential. A future narrow
  inference broker could keep the upstream secret outside the model process.
- NVIDIA NIM may process repository source; operators must review confidentiality,
  retention, regional, and contractual obligations.
- The verifier executes untrusted code on an ephemeral hosted runner with
  outbound network access, but receives no publication, NIM, OIDC, artifact/cache
  runtime, command-file, or reviewer credential.
- GitHub artifact storage, hosted runners, and pinned actions remain trusted
  infrastructure. Digests prove identity, not semantic correctness.
- GitHub cannot atomically create a PR only when none exists. Final queue and
  base revalidation, unique branches, review, and exact-head Checks bound the
  race.
- The pinned OpenCode release intentionally trails the latest observed upstream
  release until the exact Linux asset digest is independently reviewed.
