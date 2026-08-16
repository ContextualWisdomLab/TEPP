# Membership cutoff eligibility (doctoring)

## Scope

`membership_cutoff` filters multiple-membership observations so a historical
estimation row cannot include a membership whose availability time exceeds
the knowledge cutoff. Recovery is the computed share of eligibility flags
that match known truth.

This slice does not persist memberships, estimate multilevel weights, or
replace `corpus_split` document snapshots.

## Authority

### Normative TEPP contract

- `docs/adr/0002-six-clock-temporal-semantics.md` — no analysis may use
  evidence whose availability time exceeds its cutoff.
- `docs/adr/0003-relational-event-multiple-membership.md` — documents may
  belong to several units at once; each membership is time-varying.

### Supporting literature

Snijders and Bosker (2012) treat multiple membership as a distinct
multilevel structure. They do **not** authorize including later-available
memberships in an earlier cutoff window.

Snijders, T. A. B., & Bosker, R. J. (2012). *Multilevel analysis: An
introduction to basic and advanced multilevel modeling* (2nd ed.). SAGE.
