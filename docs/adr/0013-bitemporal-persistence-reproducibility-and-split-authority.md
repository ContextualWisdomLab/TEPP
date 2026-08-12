# ADR 0013 — Bitemporal persistence, reproducibility manifests, and split authority

**Decision status:** Accepted  
**Implementation maturity:** partial — migration contracts, cutoff eligibility, in-memory bitemporal adapters, live SQL session/migration port, document SQL contracts, `DATABASE_URL` SQLx gate, optional `live-sqlx` `PgPool` open/execute driver, exact-head live PostgreSQL CI, and tenant RLS (`tepp_app_runtime` + session GUC) implemented; full physical ERD, concurrent write stress, and backup/restore remain accepted-target  
**Date:** 2026-08-12  
**Supersedes:** None; complements ADR 0002 (temporal semantics), ADR 0008 (evidence identity), and ADR 0011 (service ownership).

## Context

TEPP's temporal/event measurements are invalid if persistence collapses uncertain time to false exact timestamps, allows historical evidence to be rewritten, or permits train/validation/test partitions to ignore revision, translation, copy, or episode relationships. Reproducibility also requires more than an audit string: a model run must bind immutable evidence, preprocessing, concept, model, seed, backend, code, dependency, and split identities.

## Decision

PostgreSQL is TEPP's reference relational persistence adapter, but the domain contracts remain storage-independent. Persistent analytical state uses descriptive two-or-more-word `snake_case` object names and preserves event/valid time, system/record time, availability, uncertainty/precision, tenant/authorization boundaries, lifecycle, and provenance explicitly.

The persistence layer must provide:

- immutable/append-only source and reproducibility identities;
- explicit bitemporal or equivalent valid/system-time semantics where records can change;
- exact or uncertain/open interval representations without coercion to false points;
- relation-aware corpus split manifests bound to `knowledge_cutoff`;
- immutable reproducibility manifests covering source/evidence hashes, preprocessing and concept versions, model contract/configuration, random seed manifest, compute backend/version, dependency lock, Git commit, and provenance identity;
- immutable model-artifact content hashes linked through the originating model run;
- tenant isolation and least-privilege access, with row-level policy where the deployment model requires it;
- idempotent writes, concurrency controls, migration/rollback evidence, backup/restore, retention/deletion/legal-hold behavior, and integrity revalidation after recovery.

A relation-aware split groups revisions, translations, copied/template-derived variants, shared event episodes, or other governed dependency components whenever separation would create leakage. `available_time <= knowledge_cutoff` remains mandatory for every historical partition.

## Alternatives considered

1. **Store only current normalized rows** — rejected because it erases temporal/provenance history and makes historical replay unreliable.
2. **Treat audit logs as the reproducibility manifest** — rejected because audit events need not bind every scientific identity and can be mutable/partial.
3. **Random document split** — rejected because related revisions/translations/episodes can leak across partitions.
4. **Bitemporal/immutable evidence plus relation-aware split/manifests** — accepted.

## Consequences

- model runs can be reproduced from explicit identities rather than inferred from database state;
- recovery and reprocessing re-run temporal eligibility and lineage checks instead of trusting restored rows blindly;
- database migrations become scientific-interface changes when they alter temporal, membership, evidence, or model identity semantics;
- object-store references are trusted only when immutable/versioned and digest-verified.

## Failure and recovery

Missing or mutable referenced artifacts, manifest digest mismatch, stale split identity, inconsistent time precision, cross-tenant references, broken relation components, or migration/restore integrity failures fail closed. Recovery reconstructs from immutable evidence and validates authorization, temporal cutoff, relation-aware split, manifest digests, and model/backend compatibility before analytical state is marked usable.

## Security, privacy, and governance impact

Persistence follows ADR 0009 purpose-bound PII rules. Identity mappings, sensitive raw evidence, derived psychometric results, and provider/export artifacts can have different access scopes and retention rules. Backups inherit the same protection and deletion/legal-hold governance as primary data.

## Compatibility and migration

Every schema change has forward and rollback plans, compatibility notes, invariant tests, and a clear ownership boundary. Direct cross-service application-table access remains prohibited under ADR 0011. External stores are adapters behind versioned ports rather than alternate sources of scientific truth.

## Verification

Required tests cover bitemporal replay, uncertain/open interval round trips, relation-aware split leakage, exact knowledge-cutoff inclusion/exclusion, append-only manifest enforcement, digest mismatch, idempotency, concurrent writes, tenant isolation, migration/rollback, backup/restore, retention/deletion/legal hold, and full run-to-artifact provenance reconstruction.

## Rollback and supersession

Rollback restores the last schema and adapters that preserve all accepted identities and temporal semantics; it never drops evidence required to interpret already-published artifacts. Supersede only through an ADR that preserves reproducibility, historical eligibility, and service authority or explicitly migrates them with equivalent evidence.
