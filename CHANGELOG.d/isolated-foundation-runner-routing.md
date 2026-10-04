# Isolated Foundation and documentation CI routing

- Require the dedicated `CWL CI isolated` runner group and `self-hosted`, `linux`, `x64`, `cwlab-ci-isolated` labels for the four Foundation CI jobs and one documentation job.
- Preserve all existing events, exact-head checkout, read-only permissions, tests, coverage thresholds, caches and job commands. Declare the canonical custom labels for repository-normal actionlint; label declarations are not isolation evidence.
- Leave the credential-bearing hourly NIM/App workflow unchanged. Its trusted execution and disposable-work separation belong to the existing central migration and runtime isolation owners, not this public-validation slice.
- Source-only/Draft rollout: missing isolated capacity, kernel-denial/reset and image/cache/HOME cleanup evidence, least-privilege access, real canary and current-head approvals block execution acceptance and protected-main integration. See `docs/operations/ISOLATED_FOUNDATION_RUNNERS.md`.
