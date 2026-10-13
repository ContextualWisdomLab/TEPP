# Authenticated central CI evidence

These non-executable fixtures preserve raw Git objects from central commit
`af81ef49683809ce8b302e078455e3960cf0acd1`. Both consumer callers use that
same immutable pin. `scripts/effective_ci_source.py` authenticates the commit,
root tree, `.github` tree, workflow tree, and the selected workflow blob before
checking its independent SHA-256 digest and its exact consumer caller digest.
Only the Rust foundation and documentation workflow names are admitted. There
is no Git metadata requirement or network fallback.

The foundation workflow is the original 16,776-byte blob
`36c063a0a480064d1f445f69aa1dcd4c2d9bd617`, SHA-256
`3a441902290088d0b6f884057c84dba5ab5b3ba4c7e2b27319b2ddfea6153ec8`.
The documentation workflow is the original 1,275-byte blob
`d28d4c0be766912ebcaa9ce14791238cb27730c4`, SHA-256
`d1d62e9ba4ff5ecc0ef499061dae5ea84b05db4a5d94f42b0290d83b8f5885d5`.
All six files were regenerated from the actual central Git objects at this pin;
no signature or workflow bytes were reconstructed or normalized. The signed
`commit.object` retains its exact-path binary attribute. The supplier provisions
private Python environments for all five jobs and hash-verifies official
Rustup 1.29.1 before Rust toolchain installation in the three Rust jobs.

Caller tests extract the actual top-level job IDs from each authenticated
supplier body: `repository-contracts`, `rust-quality`, `production-coverage`,
`live-postgres`, and `validate`. Controls alter or omit each actual ID and
exercise both blob-custody rejection and the independent ID assertion.
Documentation controls also reject absent/corrupt objects, caller pin and
permission drift, disabled or additional caller jobs, digest drift, and
non-allowlisted supplier names. These are static source-custody checks, not
execution of the five GitHub Actions jobs or proof of hosted acceptance.
