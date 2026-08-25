# Audit-event persistence (doctoring)

## Scope

`audit_event` already exists on the foundation schema as an append-only
identity (`0004` refuses UPDATE/DELETE/TRUNCATE). This slice adds the
fail-closed insert contract so an action code cannot be persisted when it
is empty, oversized, or contains a control character or SQL-hostile token,
and so `append_audit_sql` cannot render `INSERT INTO audit_event` until
`operational_log::try_record` has refused source text, source identity, and
blanket-mask grants. Audit rows remain distinct from reproducibility
manifests: they record an action against a subject identity, not a
scientific digest triple (Jensen & Snodgrass, 1999).

This does not add a new migration number. Physical immutability already
lives in `0004`.

## Authority

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

National Institute of Standards and Technology. (2020). *Security and
privacy controls for information systems and organizations* (NIST Special
Publication 800-53 Rev. 5). U.S. Department of Commerce.
https://doi.org/10.6028/NIST.SP.800-53r5

Audit trails are a control family, not a reproducibility manifest
(National Institute of Standards and Technology, 2020). Collapsing an
untrusted action token into SQL, or treating the audit log as the
scientific identity of a model run, would destroy both the control
boundary and temporal replay (Jensen & Snodgrass, 1999).

## Verification

- contract tests reject empty, apostrophe, semicolon, backslash, control,
  and oversized action codes;
- a valid `revise` action with clear inspection renders `INSERT INTO audit_event`;
- author/customer/project source text, source identity, and a blanket mask
  refuse the insert and never appear in the SQL;
- recording-session coverage refuses a hostile action after a valid insert;
- live PostgreSQL CI appends a valid action and refuses a hostile code
  when `TEPP_LIVE_POSTGRES=1`.
