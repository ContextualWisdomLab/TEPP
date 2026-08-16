# Selective disclosure field grants (doctoring)

## Scope

`selective_disclosure` authorizes purpose-bound field sets. Scientific
exports keep authorship, event-time, and membership linkage present on
the source. Operational monitoring and scientific purposes refuse
direct identity and source text. Re-identification is the only identity
grant. Blanket PII masking is not a disclosure authorization. Recovery
is the computed share of disclosed field sets that match known truth.

This slice does not persist grants, encrypt mappings, or claim CSAP,
SOC 2, or legal sufficiency.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — selective
  disclosure without blanket masking.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — allowed fields/source classes
  are part of every protected disclosure evaluation.

### Supporting literature

ISO/IEC 29100 treats use, retention and disclosure limitation, and
data minimization, as applying to disclosed as well as collected
personal data. They do **not** authorize destroying authorship,
temporal, or membership linkage in order to "mask PII," and they do
not certify TEPP.

International Organization for Standardization and International
Electrotechnical Commission. (2011). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2011).

National Institute of Standards and Technology. (2020). *NIST privacy
framework: A tool for improving privacy through enterprise risk
management, version 1.0*. https://doi.org/10.6028/NIST.CSWP.01162020
