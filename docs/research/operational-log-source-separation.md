# Operational log and source separation (doctoring)

## Scope

`operational_log` records action codes, opaque analytical subjects, and
system time. Raw source text and source identity cannot enter the log.
Blanket PII masking is not a log grant. Recovery is the computed share of
replayed lines that match known truth.

This slice does not persist the log, ship it to a SIEM, or claim CSAP,
SOC 2, or legal sufficiency.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — log/source separation
  and auditable privileged access; identity/role/linkage remain
  scientifically required when authorized.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — ordinary logs must not copy raw
  source or re-identification mappings.

### Supporting literature

ISO/IEC 27002 treats logging as an operational control that must not become
an unbounded copy of protected data (International Organization for
Standardization & International Electrotechnical Commission, 2022). It
does **not** authorize writing source text into operational logs, and it
does not certify TEPP.

ISO/IEC 29100 treats data minimization as a privacy-engineering principle
(International Organization for Standardization & International
Electrotechnical Commission, 2011). TEPP uses it to keep source text out
of the log, not as a certification claim.

International Organization for Standardization and International
Electrotechnical Commission. (2022). *Information security, cybersecurity
and privacy protection—Information security controls* (ISO/IEC Standard No.
27002:2022).

International Organization for Standardization and International
Electrotechnical Commission. (2011). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2011).
