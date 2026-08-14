# Identity mapping and re-identification export (doctoring)

## Scope

`identity_mapping` keeps source identities out of ordinary compute. Exporting
the analytical-id to source-identity map requires an explicit re-identification
purpose. Analytical purpose and blanket PII masking cannot unlock the map.
Recovery is the computed share of exported pairs that match known truth.

This slice does not persist the mapping, encrypt the store, or claim CSAP,
SOC 2, or legal sufficiency.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — purpose-bound
  authorization, opaque analytical identifiers, and separately protected
  identity mapping; blanket masking is not the primary control.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — re-identification is a privileged
  operation distinct from analytical use.

### Supporting literature

ISO/IEC 29100 treats purpose specification and use limitation as distinct
privacy-engineering controls. They do **not** authorize substituting a global
mask for a re-identification grant, and they do not certify TEPP.

ISO/IEC 20889 defines re-identification as recovering identity from
de-identified data. TEPP treats that recovery as an explicitly authorized
export, not as a side effect of analysis.

International Organization for Standardization and International
Electrotechnical Commission. (2011). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2011).

International Organization for Standardization and International
Electrotechnical Commission. (2018). *Privacy enhancing data
de-identification terminology and classification of techniques* (ISO/IEC
Standard No. 20889:2018).
