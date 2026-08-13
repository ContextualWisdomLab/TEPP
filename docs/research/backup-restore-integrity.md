# Backup/restore integrity (doctoring)

## Scope

A restored PostgreSQL copy is untrusted. TEPP must not mark analytical state
usable until tenant identity, canonical content digests, knowledge-cutoff
eligibility, temporal window order, and append-only triggers are revalidated
(Jensen & Snodgrass, 1999; National Institute of Standards and Technology,
2010). This slice adds that fail-closed gate without a new migration number.

It is not a substitute for operator backup tooling, RPO/RTO measurement, or
a claim of disaster-recovery certification.

## Authority

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

National Institute of Standards and Technology. (2010). *Contingency planning
guide for federal information systems* (NIST SP 800-34 Rev. 1). U.S.
Department of Commerce. https://doi.org/10.6028/NIST.SP.800-34r1

Restored rows can silently invert valid/system windows or drop immutability
controls. Revalidation before use is the scientific recovery contract, not
an availability SLO.

## Verification

- valid reconstructed snapshots may be marked usable;
- missing tenant, non-canonical digest, future-available evidence, inverted
  valid windows, and missing append-only triggers fail closed;
- probe SQL raises `restore integrity failed` for digest, window, cutoff, and
  trigger checks;
- live PostgreSQL CI runs the probes after applying the embedded catalog.
