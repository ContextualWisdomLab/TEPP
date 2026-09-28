# Actions workflow fleet audit

Orphan GitHub Actions identities can remain `active` after their YAML is
removed from protected `main`. Tree-level quality contracts cannot see those
control-plane records. `scripts/actions_workflow_fleet.py` inventories the
registry against the exact default-branch SHA and tree, then optionally
disables only reviewed orphans.

## Classification

Every paginated registry identity is classified as:

- `present` — repository path exists on the bound default-branch tree;
- `orphan` — repository-path identity is active and absent from that tree;
- `disabled` — GitHub `disabled`, `disabled_manually`, `disabled_fork`,
  `disabled_inactivity`, or `deleted`;
- `github_dynamic` — GitHub-owned dynamic path (for example CodeQL).

Name-only bootstrap or repair heuristics are not used. A live workflow whose
name contains those words remains `present` when its path is on protected
`main`.

Protected production paths are never disabled:

- `.github/workflows/ci.yml`
- `.github/workflows/docs-quality.yml`
- `.github/workflows/hourly-nim-product-development.yml`

## Credentials

The auditor accepts only `GITHUB_TOKEN` or `GH_TOKEN`. It does not invent a
TEPP-specific PAT, read `NVIDIA_NIM_API_KEY`, or use
`COPILOT_GITHUB_TOKEN`. Product-development automation remains on
`NVIDIA_NIM_API_KEY` and OpenCode.

## Commands

Read-only inventory:

```bash
GITHUB_TOKEN="$(gh auth token)" python3 scripts/actions_workflow_fleet.py \
  audit --owner ContextualWisdomLab --repo TEPP
```

Plan disable without mutation:

```bash
GITHUB_TOKEN="$(gh auth token)" python3 scripts/actions_workflow_fleet.py \
  disable-orphans --owner ContextualWisdomLab --repo TEPP
```

Apply disable after a live re-fetch of branch SHA, tree, and each identity:

```bash
GITHUB_TOKEN="$(gh auth token)" python3 scripts/actions_workflow_fleet.py \
  disable-orphans --apply --owner ContextualWisdomLab --repo TEPP
```

Apply is fail-closed on permission loss (403), missing identities (404),
upstream 5xx, pagination or tree truncation, default-branch movement,
protected paths, and identities that are no longer active orphans.
GitHub's disable PUT is expected to leave `state: disabled_manually`.

## Recurrence

Retain the JSON inventory (workflow ID, path, state, classification,
default-branch SHA, timestamp, and pagination receipts). Re-run the
read-only audit after any workflow deletion. Coordinate with
`ContextualWisdomLab/.github#945` and `ContextualWisdomLab/appguardrail#929`.
Do not recreate deleted bootstrap, repair, or repository-local PR-maintenance
YAML. PR review and merge scheduling belongs to the central required workflow.

## 2026-08-13 live remediation

Inventory bound to `main` SHA `3810bb73e3606431e1e19497b9746a8335e5d379`
reported 15 identities and 10 orphans. After `--apply`, those 10 records
were `disabled_manually` and `orphan_count` was 0. The then-current protected
repository workflows and GitHub-owned CodeQL remained `active`.
