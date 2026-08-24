# Provider-disclosure receipts (doctoring)

## Scope

`provider_receipt` records the purpose and field codes sent to a model
provider. Source text and source identity cannot enter the receipt.
Blanket PII masking is not a disclosure grant. Recovery is the computed
share of field codes that match known truth.

This slice does not send HTTP, persist receipts, or claim CSAP, SOC 2, or
legal sufficiency.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — model/provider payloads
  are evidence-minimized and version/audit bound.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — provider payload minimization and
  raw-source log absence are required tests.

### Supporting literature

ISO/IEC 29100 treats data minimization and purpose specification as
distinct controls. They do **not** authorize copying source text into a
provider audit artifact, and they do not certify TEPP.

International Organization for Standardization and International
Electrotechnical Commission. (2011). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2011).
