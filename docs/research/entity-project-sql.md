# Entity and project target SQL (doctoring)

## Scope

Migration `0006` already stores `entity_record` and `project_record` so
membership assignments can use typed foreign keys. This slice adds
application-layer insert and primary-key lookup SQL that refuse empty,
oversized, or hostile type/status labels before any statement is rendered
(Snijders & Bosker, 2012; Beretvas, 2011; Jensen & Snodgrass, 1999).

No new migration number is allocated. `text_segment` persistence, encrypted
identity mapping, and estimator fitting remain later work.

## Authority

Snijders, T. A. B., & Bosker, R. J. (2012). *Multilevel analysis: An
introduction to basic and advanced multilevel modeling* (2nd ed.). SAGE.

Beretvas, S. N. (2011). Cross-classified and multiple-membership models. In
J. J. Hox & J. K. Roberts (Eds.), *Handbook of advanced multilevel analysis*
(pp. 313–334). Routledge.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

Cross-classified and multiple-membership models require distinct, referentially
intact group identities (Snijders & Bosker, 2012; Beretvas, 2011). Seeding
those identities from attacker-controlled labels would collapse the typed
target contract into string interpolation. Fail-closed validation at the
application SQL boundary preserves target identity without treating a
contextual type/status code as a direct personal identifier (Jensen &
Snodgrass, 1999).

## Verification

- unit and crate contract tests refuse empty, quoted, semicolon, backslash,
  control, and 129-byte labels before SQL is produced;
- live PostgreSQL CI inserts author/department entities and one active
  project through the same helpers, looks them up by primary key, and still
  rejects a wrong-tenant GUC under FORCE RLS when `TEPP_LIVE_POSTGRES=1`.
