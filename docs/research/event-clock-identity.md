# Event-clock identity (doctoring)

## Scope

`event_clock` keeps event/valid time distinct from assertion, system,
document, and availability time. Recovery is the computed share of event
stamps that match known truth.

This slice does not persist clocks or recreate `document_clocks`,
`available_clock`, `cutoff_clock`, or `assertion_clock`.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — event/valid time is
  when a state occurred; it is not assertion time, document time, system
  time, or availability time. Forward transitions require an event-time
  partial order.

### Supporting literature

Snodgrass (2000) separates valid time from transaction time. Event time
is TEPP's valid-time clock. Assertion, document, and record times are
not substitutes for when the event occurred.

Snodgrass, R. T. (2000). *Developing time-oriented database applications
in SQL*. Morgan Kaufmann.
