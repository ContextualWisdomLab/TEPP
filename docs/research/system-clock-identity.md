# System-clock identity (doctoring)

## Scope

`system_clock` keeps system/record time distinct from event, assertion,
document, availability, and knowledge-cutoff time. Recovery is the
computed share of system stamps that match known truth.

This slice does not persist clocks or recreate `document_clocks`,
`available_clock`, `cutoff_clock`, `assertion_clock`, or `event_clock`.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — system time is when
  TEPP recorded a change; it is not event time or availability time.

### Supporting literature

Snodgrass (2000) treats transaction time as the time a fact was recorded.
That is the TEPP system clock. Valid time and other TEPP clocks are not
substitutes.

Snodgrass, R. T. (2000). *Developing time-oriented database applications
in SQL*. Morgan Kaufmann.
