# Immutable evidence records validation

## Scope

This report records the fresh test-first evidence for the first executable
`evidence_core` slice. The validated scope is limited to independent RFC 9562
`UUIDv7` record identities, canonical `SHA-256` content digests, immutable
source bytes and UTF-8 document text, exact byte and Unicode-scalar spans, and
optional bounded page-layout coordinates.

It does not claim source attestation, digital signatures, acquisition
provenance, versioned wire DTOs, database persistence, temporal reasoning,
GPU compute, psychometric estimation, deployment, or release readiness.

## TDD sequence

| Stage | Commit | Expected result |
|---|---|---|
| Initial identifier RED | `3e497f1ca3bee47f44e558b07a24ab2f0cdbfba7` | `EvidenceId` and `EvidenceError` absent |
| Identifier GREEN | `7141c1056e3b97dd84488fa3f7352f41069358ba` | RFC 9562 vector and generated `UUIDv7` tests pass |
| Record/span RED | `effcf6c9999f22489e835a4b5823e8931e8ff0cd` | digest, record, page, and span APIs absent |
| Record/span GREEN | `ff0688b79ba87040513a86e0d4d7884a54cc8bb0` | minimal implementation exists |
| Branch/format repair | `2e756ef2fbab01bcd9267706f8d725d4ecea8970` | all validation branches are exercised and formatted |
| Trust-boundary doctoring | `f383cd50c27cf1e94f8fe774af7f6f56a44902eb` | ADR, architecture, CHANGELOG, and APA 7 doctoring added |
| Task 1 refresh | `ca7e43a8112f2d3a698c6c8277e5d238c508f48e` | final workspace foundation incorporated |
| Exact coverage repair | `e1a329ce2e7f57b3979339945f1ecb61d77c7490` | strict Clippy and coverage gates all pass |

The RED commits are retained in branch history as evidence that behavior
contracts preceded implementation. They are not production defects or accepted
release heads.

## Exact-head GitHub evidence

Verified code head: `e1a329ce2e7f57b3979339945f1ecb61d77c7490`.

Rust Foundation CI run `31007663689` completed successfully:

- repository contracts and Python branch coverage: success;
- `cargo fmt --all -- --check`: success;
- `cargo check --workspace --all-targets --all-features`: success;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: success;
- `cargo nextest run --workspace --all-features`: success without retries;
- doctests: success;
- warning-free rustdoc: success;
- `cargo deny check`: success;
- production line coverage: **297/297, 100%**;
- production branch coverage on pinned nightly: **58/58, 100%**.

The workspace executed 25 integration and crate-contract tests. The executable
`evidence_core` behavior includes four identifier tests and eleven record/span
tests; the remaining foundation crates each retain their package identity
contract without placeholder production behavior.

Documentation Quality run `31007663509` also completed successfully on the same
code head.

## Realistic and hostile cases

The permanent suite covers:

- the RFC 9562 Appendix A.6 `UUIDv7` vector and generated version-seven IDs;
- known `SHA-256` vectors and canonical lower-case hexadecimal round trips;
- malformed, non-ASCII, and invalid-position digest input;
- caller-owned byte-buffer mutation after artifact acceptance;
- empty and configured-size-limit rejection before immutable allocation;
- hostile multibyte UTF-8 text containing Latin and supplementary-plane code
  points;
- exact agreement between half-open byte ranges and Unicode-scalar ranges;
- empty, reversed, byte-overflow, scalar-overflow, start-boundary,
  end-boundary, start-scalar, end-scalar, and cross-document failures;
- zero page numbers, nonfinite or nonpositive page dimensions, nonfinite,
  negative, or empty rectangle components, and horizontal or vertical overflow;
- stable errors that do not include source content or paths.

## Review and merge posture

The implementation remains a stacked Draft PR. It cannot merge before the
approved planning baseline and Task 1 workspace foundation. Versioned DTO
serialization and generated/property-style span contracts remain Task 2 work.
After the stack is retargeted to the protected default branch, every required
check and independent non-author approval must be reacquired on the exact final
head without bypass.
