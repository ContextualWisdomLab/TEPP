# Temporal/Event Foundation Task 1 validation

## Scope

This report covers only the Rust workspace and quality-gate foundation. It does
not validate temporal algebra, event ontology, PostgreSQL migrations, GPU
kernels, psychometric estimation, true-parameter recovery, deployment, or a
release artifact.

## Test-first evidence

The repository-contract tests were authored to fail on the pre-Task-1 state:
there was no root Cargo workspace, approved crate set, Rust CI workflow,
dependency policy, documentation gate, or coverage gate. The permanent tests
retain those fail-closed cases for missing manifests, missing crates, unpinned
Actions, forbidden credentials, undocumented Rust APIs, malformed LLVM coverage
JSON, impossible counts, and incomplete coverage.

## Local verification

The implementation environment did not provide a Rust toolchain, so Rust
compilation and LLVM coverage remain GitHub-hosted exact-head gates. The
repository tooling was executed locally:

```text
python3 scripts/check_workspace_contract.py
TEPP workspace contract: PASS

python3 scripts/check_docstrings.py
Rust documentation contract: PASS

python3 -m unittest discover -s tests/quality -p 'test_*.py'
16 tests passed

python3 -m coverage run --branch -m unittest discover -s tests/quality -p 'test_*.py'
python3 -m coverage report --show-missing
226 statements, 100%
110 branches, 100%
```

## Rust coverage interpretation

Task 1 crate roots contain documentation and lint attributes but no executable
production behavior. LLVM may therefore report zero production lines or
branches. The coverage checker permits this only with an explicit
`0 executable units` message. It rejects every nonzero denominator unless
`covered == count`.

This is a coverage property of the skeleton-only slice, not evidence that TEPP
has implemented its planned domain or statistical behavior.

## Exact-head gates

The PR is ready to merge only after all of the following succeed on its current
head:

- repository contract and Python statement/branch coverage;
- Rust formatting, all-target compile, strict Clippy, and warning-free rustdoc;
- cargo-nextest without retries and separate doctests;
- cargo-deny advisory, license, ban, and source checks;
- stable Rust production line coverage;
- pinned-nightly production branch coverage;
- repository Security Scan, Semgrep, and independent review.

## Next implementation slice

Task 2 introduces immutable evidence identifiers, content hashes, source
artifacts, and exact UTF-8/page/layout spans. That behavior must begin with
failing unit and property tests and must turn the current zero coverage
denominator into measured executable production coverage.
