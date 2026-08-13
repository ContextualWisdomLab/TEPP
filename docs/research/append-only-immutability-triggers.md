# Append-only immutability triggers (doctoring)

## Scope

Migration `0004` adds defense-in-depth so identity and manifest tables cannot be rewritten after insert:

- `tepp_app_runtime` loses `UPDATE`, `DELETE`, and `TRUNCATE` privileges;
- one `BEFORE UPDATE OR DELETE OR TRUNCATE` trigger is attached to every append-only table; and
- triggers run `FOR EACH STATEMENT`, so a destructive statement is rejected even when its predicate would affect zero rows.

PostgreSQL permits multiple trigger events in one definition, but `TRUNCATE` triggers are statement-level only. The shared trigger function does not inspect row images, so a statement-level trigger is the narrowest contract that blocks all three destructive operations consistently.

The migration does not claim protection from a database superuser or owner who deliberately drops or disables the control. It provides enforceable application-role least privilege and a table-owner mutation tripwire within the governed schema lifecycle.

## Authority

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

International Organization for Standardization. (2011). *Information technology—Database languages—SQL—Part 2: Foundation (SQL/Foundation)* (ISO/IEC Standard No. 9075-2:2011).

PostgreSQL Global Development Group. (2026). *CREATE TRIGGER*. In *PostgreSQL 18 documentation*. https://www.postgresql.org/docs/18/sql-createtrigger.html

## Verification

- catalog validation requires the rejection function, multi-word trigger identities, and `UPDATE`/`DELETE` revokes for every append-only table;
- an executable migration contract test verifies every embedded trigger is bound to its intended table, covers `UPDATE`, `DELETE`, and `TRUNCATE`, is statement-level, and invokes `reject_append_only_mutation`;
- the same contract verifies `TRUNCATE` is revoked without being newly granted by rollback; and
- live PostgreSQL CI proves representative `UPDATE`/`DELETE` mutation attempts on `reproducibility_manifest` fail closed when `TEPP_LIVE_POSTGRES=1`.
