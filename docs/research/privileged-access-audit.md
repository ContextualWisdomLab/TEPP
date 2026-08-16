# Privileged-access audit replay (doctoring)

## Scope

`privileged_access` records re-identification and grant-use decisions as
allow/deny events bound to an opaque analytical subject and a purpose code.
Source identity cannot appear in the log. Blanket PII masking is not an audit
grant. Recovery is the computed share of replayed records that match known
truth.

This slice does not persist the log, encrypt it, or claim CSAP, SOC 2, or
legal sufficiency.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — auditable privileged
  access; identity/role/linkage remain scientifically required when
  authorized.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — privileged operations are evaluated
  with purpose, tenant, role, and lifetime evidence; forensic/audit
  evidence must stay bounded.

### Supporting literature

ISO/IEC 27002 treats privileged-access rights and logging as distinct
operational controls. They do **not** authorize writing source identity into
an audit artifact, and they do not certify TEPP.

ISO/IEC 29100 treats accountability as a privacy-engineering principle. TEPP
uses it to require a replayable decision log, not as a certification claim.

International Organization for Standardization and International
Electrotechnical Commission. (2022). *Information security, cybersecurity
and privacy protection—Information security controls* (ISO/IEC Standard No.
27002:2022).

International Organization for Standardization and International
Electrotechnical Commission. (2011). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2011).
