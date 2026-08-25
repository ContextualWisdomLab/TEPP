# Customer/competitor role contradiction (doctoring)

## Scope

`role_contradiction` keeps customer, partner, and competitor as
time-varying contextual roles. A customer/competitor pair in one group
fails closed. Recovery is the computed share of recovered role labels
that match known truth.

This slice does not persist memberships, allocate migration `0008`, or
replace `membership_core`.

## Authority

### Normative TEPP contract

- `docs/adr/0003-relational-event-multiple-membership.md` — customer,
  partner, and competitor are role assignments, not immutable entity
  classes; contradictory role assertions fail closed.

### Supporting literature

Biddle (1986) treats roles as contextual, potentially conflicting
positions rather than permanent types. Browne, Goldstein, and Rasbash
(2001) keep crossed classifications distinct so one unit can belong to
several non-nested groups without collapsing those roles.

Biddle, B. J. (1986). Recent developments in role theory. *Annual Review
of Sociology, 12*, 67–92. https://doi.org/10.1146/annurev.so.12.080186.000435

Browne, W. J., Goldstein, H., & Rasbash, J. (2001). Multiple membership
multiple classification (MMMC) models. *Statistical Modelling, 1*(2),
103–124. https://doi.org/10.1177/1471082X0100100201
