# Relation absence is not evidence of no relationship (doctoring)

## Scope

`relation_absence` keeps observed, inferred, and unobserved statuses
distinct. Recovery is the computed share of recovered statuses that
match known truth.

This slice does not persist the graph, promote inferred edges, or
replace `relation_graph`.

## Authority

### Normative TEPP contract

- `docs/adr/0003-relational-event-multiple-membership.md` — observed
  relation evidence, inferred relations, and promoted transition edges
  remain distinct. Relation absence is not silently interpreted as
  evidence of no relationship.

### Supporting literature

Altman and Bland (1995) separate a missing comparison from a negative
finding. Treating an unobserved pair as "no relationship" converts
non-observation into a closed-world denial.

Altman, D. G., & Bland, J. M. (1995). Absence of evidence is not
evidence of absence. *BMJ, 311*(7003), 485.
https://doi.org/10.1136/bmj.311.7003.485
