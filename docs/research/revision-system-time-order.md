# Document revision system-time order (doctoring)

## Scope

`revision_order` requires a later document revision number to carry a
strictly later system time. Recovery is the computed share of order flags
that match known truth.

This slice does not persist revisions, allocate migration `0008`, or
replace `persistence_postgres` interval CHECKs.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — system/record time is
  distinct from event time; later assertions cannot rewrite earlier
  system-time order.
- `docs/adr/0013-bitemporal-persistence-reproducibility-and-split-authority.md`
  — document versions are bitemporal; revision identity is ordered.

### Supporting literature

Snodgrass (2000) treats transaction/system time as the time a fact was
recorded. A later recorded version cannot precede an earlier one in
system time.

Snodgrass, R. T. (2000). *Developing time-oriented database applications
in SQL*. Morgan Kaufmann.
