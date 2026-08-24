# Availability-clock identity (doctoring)

## Scope

`available_clock` keeps availability time distinct from event time and
system time. Recovery is the computed share of availability stamps that
match known truth.

This slice does not persist clocks, replace `temporal_core`, or recreate
`document_clocks`.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — availability time is
  the time evidence became usable; it is not event time or system time.
- Historical analyses may not treat record time as the moment evidence
  was available.

### Supporting literature

Snodgrass (2000) separates valid time from transaction time. Availability
is a third TEPP clock: when the analyst could use the evidence. Neither
valid time nor transaction time is a substitute.

Snodgrass, R. T. (2000). *Developing time-oriented database applications
in SQL*. Morgan Kaufmann.
