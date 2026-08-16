# Operational log and source separation (doctoring)

## Scope

`operational_log` records action codes, opaque analytical subjects, and
system time. `try_record` is the only recording API and inspects
caller-supplied source text, source identity, and blanket-mask intent
before a line is created. `OperationalLogRecord::new` is crate-private.
A source-identity `&str` cannot be converted into an `AnalyticalSubject`;
`from_opaque` still binds an already-separated `u128`. Blanket PII
masking is not a log grant. Recovery is the computed share of replayed
lines that match known truth, including a collapsed-subject baseline.

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
Electrotechnical Commission, 2024). TEPP uses it to keep source text out
of the log, not as a certification claim. The 2011 edition and its 2018
amendment are withdrawn.

International Organization for Standardization and International
Electrotechnical Commission. (2022). *Information security, cybersecurity
and privacy protection—Information security controls* (ISO/IEC Standard No.
27002:2022).

International Organization for Standardization and International
Electrotechnical Commission. (2024). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2024).
https://www.iso.org/standard/85938.html
