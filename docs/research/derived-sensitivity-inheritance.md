# Derived sensitivity inheritance (doctoring)

## Scope

`derived_sensitivity` keeps topic, factor, and relation artifacts in the
source sensitivity class. Derivation is not declassification to public.
Blanket PII masking is not a declassification grant. Recovery is the
computed share of inherited classes that match known truth.

This slice does not persist classifications, score topics, or claim CSAP,
SOC 2, or legal sufficiency.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — derived
  relation/topic/factor data is not automatically non-sensitive.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — derived-sensitive-data
  classification is a required privacy test.

### Supporting literature

ISO/IEC 29100 treats data minimization and use limitation as applying to
derived as well as collected personal data. They do **not** authorize
treating a topic proportion or factor score as public merely because it
is computed, and they do not certify TEPP.

International Organization for Standardization and International
Electrotechnical Commission. (2011). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2011).
