# A model checkpoint is not the estimator (doctoring)

## Scope

`checkpoint_authority` keeps serialized model checkpoints out of CPU
`f64` estimator authority. A checkpoint may be accepted as a run
artifact only after identity, canonical `SHA-256`, and model-run
provenance validate. Recovery is the computed share of recovered roles
that match known truth.

This slice does not persist artifacts, allocate migration `0008`, or
replace `validation_core` recovery metrics or `persistence_postgres`
model-run SQL.

## Authority

### Normative TEPP contract

- `docs/adr/0001-rust-first-modular-msa.md` — production arithmetic is
  the Rust CPU `f64` reference; optimized backends and artifacts cannot
  redefine the estimand.
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` —
  checkpoint bytes cannot promote a scientific claim without
  claim-specific recovery evidence.
- `AGENTS.md` — model checkpoints stay untrusted until identity,
  provenance, size/depth, authorization, and scientific semantics
  validate.

### Supporting literature

Peng (2011) and the National Academies (2019) treat a reproducible
computational artifact as evidence of a procedure, not as the
procedure. FIPS PUB 180-4 supplies the canonical digest used to detect
checkpoint substitution without granting estimator authority.

Peng, R. D. (2011). Reproducible research in computational science.
*Science, 334*(6060), 1226–1227. https://doi.org/10.1126/science.1213847

National Academies of Sciences, Engineering, and Medicine. (2019).
*Reproducibility and replicability in science*. The National Academies
Press. https://doi.org/10.17226/25303

National Institute of Standards and Technology. (2015). *Secure Hash
Standard (SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4
