# Membership target kinds beyond entity/project

## Scope

`membership_target` keeps language, episode, template, department, and
opportunity-pool memberships distinct from the entity/project pair stored
by migration `0006`. Recovery is the computed share of recovered target
kinds that match known truth.

This slice does not allocate migration `0008` or replace `membership_core`.

## Authority

### Normative TEPP contract

- `docs/adr/0003-relational-event-multiple-membership.md` — authors,
  departments, projects, opportunity pools, templates, languages, and
  episodes form cross-classified, time-varying, multiple-membership
  assignments. New membership targets require typed references rather
  than an untyped polymorphic identifier.

### Supporting literature

Browne, Goldstein, and Rasbash (2001) keep crossed classifications
distinct so one unit can belong to several non-nested groups. Collapsing
language or episode membership into an entity column erases that
classification.

Browne, W. J., Goldstein, H., & Rasbash, J. (2001). Multiple membership
multiple classification (MMMC) models. *Statistical Modelling, 1*(2),
103–124. https://doi.org/10.1177/1471082X0100100202
