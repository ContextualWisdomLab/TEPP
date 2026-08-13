# TEPP Operability, Recovery, and Release Guide

**Status:** Accepted target operating baseline with current maturity explicit.  
**Last reviewed:** 2026-08-13

TEPP is still an implementation-stage research/product platform. Protected main currently contains the Rust workspace/evidence foundation; PR #5/#6 add temporal foundations. Database, model fitting, GPU, services, visual analytics, and production deployment are later targets. This guide defines the operating evidence those stages must satisfy rather than claiming they already exist.

## Operating principles

- immutable source evidence is never overwritten to repair a downstream model;
- analyses are reproducible from source hashes, configuration, code, dependency lock, seeds, and knowledge cutoff;
- availability-time leakage gates precede model fitting;
- statistical uncertainty and abstention/failure are observable states;
- GPU/LLM/provider failure must have bounded degraded/fallback behavior where the product contract permits it;
- current-main versus active-PR versus target architecture is explicit in operator-facing evidence.

## Current foundation operation

The Rust domain packages are embedded/library boundaries. They require no production database or service deployment yet. Current recovery is reconstructing validated evidence objects from immutable authorized source artifacts and versioned wire records.

## Planned service SLIs

When corresponding services exist, track:

- source ingest success/rejection and exact error class;
- evidence/span count and lineage completeness;
- future-evidence exclusion count at each knowledge cutoff;
- temporal contradiction/path-consistency and budget exhaustion counts;
- event/link/tracking confidence and calibration;
- semantic-unit unknown/abstention rate by language;
- model convergence/ELBO/objective and posterior diagnostics;
- true-recovery/validation drift against release benchmark;
- CPU/GPU parity and fallback count;
- VRAM/RSS/transfer/kernel time;
- model/LLM provider failures and evidence-verifier rejection;
- artifact/export provenance completeness;
- tenant authorization/audit anomalies.

Do not expose raw PII/source text in ordinary metrics/logs merely to gain observability.

## Data snapshot and replay

A model run is pinned to an immutable corpus/evidence snapshot and `knowledge_cutoff`. Re-run/recovery must not silently include documents that became available later. Snapshot manifests record source hashes and relation-aware split identity sufficient to reproduce inclusion/exclusion.

## GPU degradation

Before admitting a GPU job, estimate budget and reserve margin. On OOM: classify, release transient allocations, shrink micro-batch a bounded number of times, then use the CPU reference or fail with an explicit resource state. Never silently change numerical precision/model specification to obtain success.

## LLM degradation

LLM-backed semantic/interpreter functions use strict bounded requests and cached/versioned results where appropriate. Provider failure may retry only under bounded policy, route through contextual-orchestrator when configured, or return deferred/unresolved evidence. It must not corrupt deterministic/statistical results or expose credentials/source beyond approved policy.

## Database target recovery

Before PostgreSQL becomes production state, prove migrations and rollback, tenant isolation/RLS, temporal/lineage constraints, idempotency/concurrency, backup/restore, retention/deletion, and reconstruction from immutable artifacts. Concurrent document first-insert and revise stress is implemented on the active PR (atomic open-row close plus typed SQLSTATE mapping) and is not a protected-main claim until integration. Backup/restore and post-restore leakage/lineage revalidation remain accepted-target. A database recovery must re-run leakage/lineage validation before analytical state is marked usable.

## Model release/cutover

A model artifact is promoted only after convergence, posterior diagnostics, true-parameter/recovery benchmarks, invariance/fairness/language evidence, uncertainty/calibration, security/privacy, and reproducibility gates meet the versioned policy. Model-selection or LLM review disagreement can require human scientific review.

## Incident RCA

Trace the first failing boundary: evidence, temporal typing/reasoning, event/relation, membership, preprocessing/concept, topic estimator, psychometric estimator, compute backend, network/cluster, LLM interpretation, persistence, export/UI, or delivery pipeline. Fix the owning layer and add a realistic regression rather than compensating downstream.

## Actions workflow fleet

GitHub Actions registry identities survive YAML deletion. After any
bootstrap, diagnosis, or repair workflow is removed from the tree, run
`scripts/actions_workflow_fleet.py audit` and retain the JSON inventory
(workflow ID, path, state, classification, default-branch SHA, timestamp,
pagination receipts). Disable only re-fetched active orphans with
`disable-orphans --apply`. Never disable the protected CI, documentation,
hourly NIM, or hourly PR-maintenance paths, and never recreate deleted
bootstrap/repair YAML. The auditor uses only `GITHUB_TOKEN`/`GH_TOKEN`.
Product-development automation continues to use `NVIDIA_NIM_API_KEY` and
must not receive `COPILOT_GITHUB_TOKEN`. Operator procedure:
`docs/operations/ACTIONS_WORKFLOW_FLEET.md`.

## Release gate

A software release requires exact protected-head CI/security/review, 100% production coverage/docs, validated migrations/rollback where present, scientific benchmark artifacts, SBOM/provenance, reproducible packages/images, operator runbooks, accessibility for product UI, CHANGELOG/version/tag consistency, and post-publish verification. TEPP has not reached that integrated release state merely because individual foundation PRs merge.