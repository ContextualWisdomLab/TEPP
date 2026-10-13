# Authenticated central CI evidence

These non-executable fixtures preserve raw Git objects from central commit
`9dab3c2a607ddef69b5ff8cb0f761e90f6bc28de`. Both consumer callers use that
same immutable pin. `scripts/effective_ci_source.py` authenticates the commit,
root tree, `.github` tree, workflow tree, and the selected workflow blob before
checking its independent SHA-256 digest and its exact consumer caller digest.
Only the Rust foundation and documentation workflow names are admitted. There
is no Git metadata requirement or network fallback.

The documentation workflow is the original 649-byte blob
`8a0ad78cd8687b2b8b0f1437e230b04e95e4aea3`, SHA-256
`76d5f64f05fbf953c8c96b6bf8129880a904498b1bbce355567108975bfb00aa`.
It reuses the existing commit and three authenticated trees, without modifying
their bytes. The signed `commit.object` retains its exact binary attribute.

Caller tests extract the actual top-level job IDs from each authenticated
supplier body: `repository-contracts`, `rust-quality`, `production-coverage`,
`live-postgres`, and `validate`. Controls alter or omit each actual ID and
exercise both blob-custody rejection and the independent ID assertion.
Documentation controls also reject absent/corrupt objects, caller pin and
permission drift, disabled or additional caller jobs, digest drift, and
non-allowlisted supplier names. These are static source-custody checks, not
execution of the five GitHub Actions jobs or proof of hosted acceptance.
