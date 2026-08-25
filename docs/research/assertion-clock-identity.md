# Assertion-clock identity (doctoring)

## Scope

`assertion_clock` keeps assertion time distinct from event, system,
document, and availability time. Recovery is the computed share of
assertion stamps that match known truth.

This slice does not persist clocks or recreate `document_clocks`,
`available_clock`, or `cutoff_clock`.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — assertion time is when
  a source claimed something; it is not event time, document time, or
  system time.

### Supporting literature

Snodgrass (2000) separates valid time from transaction time. Assertion
time is the TEPP clock for when the claim was made; valid, document, and
transaction times are not substitutes.

Snodgrass, R. T. (2000). *Developing time-oriented database applications
in SQL*. Morgan Kaufmann.
