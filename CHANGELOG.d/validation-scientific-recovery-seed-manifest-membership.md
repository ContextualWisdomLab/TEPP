# Validation: prove scientific-recovery seed membership against the declared manifest

`validation_core` now derives a versioned `ScientificRecoverySeedManifestV1` identity from the ordered unique planned RNG-state or seed-entry SHA-256 values. Scientific recovery reconstructs that manifest from the presented per-replication receipts and requires its identity to equal the seed-manifest SHA-256 already committed by `ScientificRecoveryProfileV1`.

This closes the #635 gap where arbitrary unique seed-state digests could satisfy #634 while not belonging to the declared manifest. Receipt order, cardinality, profile identity, represented payload identity, and exact-head authority gates remain fail closed. The manifest digest is content identity only; it does not prove pre-execution approval chronology or that an execution artifact was actually produced from the declared RNG state.
