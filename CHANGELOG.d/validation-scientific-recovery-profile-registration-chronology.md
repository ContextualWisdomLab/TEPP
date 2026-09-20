# Scientific recovery requires pre-execution profile approval chronology

Scientific-recovery promotion now requires a versioned `ScientificRecoveryProfileChronologyV1` in addition to the existing recovery profile, exact-head receipt, represented payload, seed-manifest membership, and per-replication execution receipts.

The chronology binds the exact recovery-profile SHA-256 to an immutable owner-ledger identity and registration-entry identity, records the registration approval state and monotonic sequence, and binds every planned execution artifact to a unique later ledger entry. Promotion fails closed when registration is pending or rejected, the chronology refers to a different profile or execution mapping, or any execution entry is not strictly later than the registration entry.

This closes represented pre-execution ordering at the Validation Evidence boundary without using caller wall-clock timestamps. Ledger identities and sequence positions remain trusted-adapter evidence; this change does not claim signature verification, trusted timestamping, GitHub artifact attestation, Sigstore verification, or SLSA provenance. Those authenticity controls remain a separate integration/release obligation.
