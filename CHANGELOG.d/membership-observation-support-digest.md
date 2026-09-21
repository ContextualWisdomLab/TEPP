## Changed

- Bind each owner-derived longitudinal Membership classification to a versioned SHA-256 identity for the exact opaque observation multiset and the active Membership topology resolved at each observation event time.
- Canonicalize observation and active-assignment ordering while preserving duplicate observation multiplicity; frame variable-length fields and hash exact binary64 membership-weight bits so equivalent input ordering cannot change support identity and distinct same-window supports cannot alias silently.
- Keep the support digest as provenance identity only. Released analytical contracts must still carry privacy-appropriate, reconstructable event-time/window/cohort/support coordinates and verify that representation against the owner digest rather than treating a digest as reconstruction evidence.
