# Typed membership-assignment persistence (doctoring)

## Scope

Migration `0006_typed_membership_assignment` replaces the foundation stub
that stored an untyped `membership_target_id`. The accepted ERD requires two
independent exactly-one constraints: the observed unit is either a document
identity or a `text_segment`, and the membership target is either an
`entity_record` or a `project_record` (Snijders & Bosker, 2012; Beretvas, 2011;
Jensen & Snodgrass, 1999). Validity uses explicit `tstzrange` windows so an
uncertain or open bound is never coerced to a false exact timestamp.

This slice does not persist event-level membership, concurrent-write stress,
or backup/restore.

## Authority

Snijders, T. A. B., & Bosker, R. J. (2012). *Multilevel analysis: An
introduction to basic and advanced multilevel modeling* (2nd ed.). SAGE.

Beretvas, S. N. (2011). Cross-classified and multiple-membership models. In
J. J. Hox & J. K. Roberts (Eds.), *Handbook of advanced multilevel analysis*
(pp. 313–334). Routledge.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

Browne, W. J., Goldstein, H., & Rasbash, J. (2001). Multiple membership
multiple classification (MMMC) models. *Statistical Modelling, 1*(2), 103–124.
https://doi.org/10.1177/1471082X0100100202

Cross-classified and multiple-membership models exist specifically so a
document can belong to several groups at once without collapsing those
contexts into a single row (Snijders & Bosker, 2012; Beretvas, 2011; Browne
et al., 2001). Typed foreign keys preserve target identity and referential
integrity at the storage boundary; multiple assignment rows plus the two
exactly-one constraints preserve those distinct contexts. Migration
`0006_typed_membership_assignment` and
`membership_assignment_contract` encode that storage contract.

## Verification

- contract tests require `entity_record`, `project_record`, `text_segment`,
  and the two exactly-one constraints in the embedded catalog;
- unit tests refuse both-or-neither keys, non-positive weights, and hostile
  labels before SQL is rendered;
- live PostgreSQL CI inserts two entity memberships and one project
  membership for the same document, asserts those three rows persist,
  and rejects both dual-target and dual observed-unit rows when
  `TEPP_LIVE_POSTGRES=1`.
- segment-level membership now inserts the observed `text_segment`
  through the typed adapter rather than ad-hoc SQL. See
  `docs/research/text-segment-sql.md`.
  `TEPP_LIVE_POSTGRES=1`;
- entity and project target rows are now inserted through fail-closed
  SQL helpers rather than raw label interpolation (see
  `docs/research/entity-project-sql.md`).
