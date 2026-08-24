# Document assertion and document time (doctoring)

## Scope

`document_clocks` requires a document analytical row to carry assertion
time and document time as first-class clocks. Event/valid time, system
time, and availability time are not substitutes. Recovery is the computed
share of completeness flags that match known truth.

This slice does not persist document rows, allocate migration `0008`, or
replace `persistence_postgres` insert SQL.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — event, assertion,
  document, system, available, and knowledge-cutoff clocks are distinct
  typed values.
- `docs/adr/0013-bitemporal-persistence-reproducibility-and-split-authority.md`
  — `document_record` already has `assertion_time` and `document_time`
  columns; omitting them is not an accepted analytical row.

### Supporting literature

Jensen and Snodgrass (1996) distinguish valid time (when a fact is true
in the world) from transaction/system time (when the fact is recorded).
TEPP keeps those two clocks and additionally requires assertion time
(when the claim was made) and document time (the time stated by the
document). Substituting event or system time for either omitted clock
collapses those meanings.

Jensen, C. S., & Snodgrass, R. T. (1996). Semantics of time-varying
information. *Information Systems, 21*(4), 311–352.
https://doi.org/10.1016/0306-4379(96)00017-8
