# Purpose-bound authorization grants (doctoring)

## Scope

`purpose_authorization` binds one processing purpose to one principal. A grant
cannot authorize a different purpose, and blanket PII masking is not
authorization. Recovery is the computed share of recovered purposes that match
known truth.

This slice does not implement export adapters, persistence of grants, or a
legal sufficiency claim.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — purpose-bound
  authorization without blanket masking; identity/role/linkage remain
  scientifically required when authorized.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — opaque analytical identifiers,
  separately protected identity mapping, and auditable privileged access.

### Supporting literature

International Organization for Standardization and International
Electrotechnical Commission. (2019). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2011/Amd 1:2018).
Purpose specification and use limitation are privacy-engineering controls.
They do **not** certify TEPP and do not authorize replacing authorization
with a global mask.

International Organization for Standardization and International
Electrotechnical Commission. (2019). *Information security, cybersecurity and
privacy protection—Privacy information management systems—Requirements*
(ISO/IEC Standard No. 27701:2019).
