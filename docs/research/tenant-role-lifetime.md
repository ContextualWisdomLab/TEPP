# Tenant, role, and lifetime access adapters (doctoring)

## Scope

`tenant_access` binds one principal to one tenant role for a system-time
lifetime window. A grant cannot authorize a different tenant, principal, or
role. Event, document, availability, and knowledge-cutoff clocks cannot
authorize access. Blanket PII masking is not authorization. Recovery is the
computed share of recovered tenant/role pairs that match known truth.

This slice does not persist grants, allocate a migration, or make a legal
sufficiency or certification claim.

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — purpose/tenant/role/lifetime
  evidence for protected operations without blanket masking.
- `docs/adr/0002-six-clock-temporal-semantics.md` — grant lifetime is system or
  assertion time; event time is not an access clock.
- `docs/PRIVACY_DATA_GOVERNANCE.md` — tenant/resource scope, role, and valid
  time/lifetime of the grant.

### Supporting literature

Ferraiolo, D. F., Sandhu, R., Gavrila, S., Kuhn, D. R., & Chandramouli, R.
(2001). Proposed NIST standard for role-based access control. *ACM Transactions
on Information and System Security, 4*(3), 224–274.
https://doi.org/10.1145/501978.501980

Bertino, E., Bonatti, P. A., & Ferrari, E. (2001). TRBAC: A temporal
role-based access control model. *ACM Transactions on Information and System
Security, 4*(3), 191–233. https://doi.org/10.1145/501978.501979

Hu, V. C., Ferraiolo, D., Kuhn, R., Schnitzer, A., Sandlin, K., Miller, R., &
Scarfone, K. (2014). *Guide to attribute based access control (ABAC) definition
and considerations* (NIST Special Publication 800-162). National Institute of
Standards and Technology. https://doi.org/10.6028/NIST.SP.800-162

National Institute of Standards and Technology. (2020). *NIST Privacy
Framework: A tool for improving privacy through enterprise risk management*
(Version 1.0). U.S. Department of Commerce.
https://doi.org/10.6028/NIST.CSWP.01162020

International Organization for Standardization and International
Electrotechnical Commission. (2011). *Information technology—Security
techniques—Privacy framework* (ISO/IEC Standard No. 29100:2011).

These sources inform tenant isolation, role assignment, and temporal grant
windows. They do **not** certify TEPP and do not authorize replacing
authorization with a global mask.

## Verification

Contract tests prove cross-tenant denial, expired and not-yet-valid windows,
inverted lifetimes, event-time clock refusal, multiple-role membership for one
principal, and a higher computed recovery rate than a collapsed single-role
assignment. Persistence of grants and live HTTP adapters remain later work.
