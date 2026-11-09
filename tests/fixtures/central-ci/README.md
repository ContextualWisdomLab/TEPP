# Authenticated central CI evidence

These non-executable fixtures preserve raw Git objects from central commit
`3136c940235a3762b804bd99eb20fbe129e70c06`. Both consumer callers use that
same immutable pin. `scripts/effective_ci_source.py` authenticates the commit,
root tree, `.github` tree, workflow tree, and the selected workflow blob before
checking its independent SHA-256 digest and its exact consumer caller digest.
Only the Rust foundation and documentation workflow names are admitted. There
is no Git metadata requirement or network fallback.

The foundation workflow is the original 17,328-byte blob
`b9e420e5c44592e49c8d7e9fce6cfcf7b6e890fa`, SHA-256
`c0022bb22b8ebf0f70c65251f3902773f6d323fcdde62d042e17fc5ddccfd01a`.
The documentation workflow is the original 1,275-byte blob
`d28d4c0be766912ebcaa9ce14791238cb27730c4`, SHA-256
`d1d62e9ba4ff5ecc0ef499061dae5ea84b05db4a5d94f42b0290d83b8f5885d5`.
All six files were regenerated from the actual central Git objects at this pin;
no signature or workflow bytes were reconstructed or normalized. The signed
`commit.object` retains its exact-path binary attribute. The supplier provisions
private Python environments for all five jobs and hash-verifies official
Rustup 1.29.1 before Rust toolchain installation in the three Rust jobs. Each
Rust job owns fresh Cargo/Rustup directories. Pinned tools use explicit private
install roots and locked versions, with no cached executable restore or cache-hit
skip. The cargo-llvm-cov 0.8.6 version probe supplies its required `llvm-cov`
plugin argument. Obsolete restore-label assertions are replaced by those actual
install/probe argv; independent negative controls remove or alter root, lock,
version and tool identity without bypassing authenticated-source checks.

Caller tests extract the actual top-level job IDs from each authenticated
supplier body: `repository-contracts`, `rust-quality`, `production-coverage`,
`live-postgres`, and `validate`. Controls alter or omit each actual ID and
exercise both blob-custody rejection and the independent ID assertion.
Documentation controls also reject absent/corrupt objects, caller pin and
permission drift, disabled or additional caller jobs, digest drift, and
non-allowlisted supplier names. These are static source-custody checks, not
execution of the five GitHub Actions jobs or proof of hosted acceptance.
