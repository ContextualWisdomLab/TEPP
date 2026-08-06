# Immutable evidence records validation

## Scope

This report records the fresh test-first evidence for the first executable
`evidence_core` slice. The validated scope includes:

- independent RFC 9562 `UUIDv7` record identities;
- canonical `SHA-256` content digests;
- immutable source bytes and UTF-8 document text;
- exact half-open byte and Unicode-scalar spans;
- optional bounded page-layout coordinates; and
- strict versioned JSON wire records that reconstruct only through the same
  domain validation boundary.

The wire contract preserves artifact, document, and span identities while
keeping internal storage and derived caches private. It rejects malformed JSON,
missing or unknown fields, unsupported schema versions, malformed identifiers
or digests, digest/content substitution, invalid byte values, stale document
ownership, invalid UTF-8 coordinates, and invalid nested page geometry.
Configured artifact and document byte limits are reapplied during
reconstruction.

This report does not claim source attestation, digital signatures, acquisition
provenance, database persistence, published JSON Schema, JSON-LD or W3C PROV
serialization, temporal reasoning, GPU compute, psychometric estimation,
deployment, or release readiness.

## TDD sequence

| Stage | Commit | Expected result |
|---|---|---|
| Initial identifier RED | `3e497f1ca3bee47f44e558b07a24ab2f0cdbfba7` | `EvidenceId` and `EvidenceError` absent |
| Identifier GREEN | `7141c1056e3b97dd84488fa3f7352f41069358ba` | RFC 9562 vector and generated `UUIDv7` tests pass |
| Record/span RED | `effcf6c9999f22489e835a4b5823e8931e8ff0cd` | digest, record, page, and span APIs absent |
| Record/span GREEN | `ff0688b79ba87040513a86e0d4d7884a54cc8bb0` | minimal immutable record implementation exists |
| Domain coverage repair | `e1a329ce2e7f57b3979339945f1ecb61d77c7490` | strict Clippy and domain coverage gates pass |
| Wire-contract RED | `3c09dcf809ab22db6679b6d4f6d1eb0ebdd1d7bf` | versioned wire APIs and error variants are absent; Rust CI fails for the intended missing behavior |
| Wire DTO GREEN | `ee0889af21c8e465146b11cf144039db5cd074c1` | private strict DTOs, schema versioning, and reconstruction paths exist |
| Artifact/document reconstruction | `6efdc5785438020381b539a9685c8fd1e76c9fed` | identities, digests, content limits, and immutable content revalidate |
| Span reconstruction | `2c65783d181d49e7d40d5f347fe3b66fcdde44ad` | document ownership, exact coordinates, UTF-8 boundaries, and page geometry revalidate |
| Hostile wire cases | `1dee164e0486bef6ac8033f7209208d659fd6c8b` | invalid identifiers, digests, bytes, empty values, and redacted errors are covered |
| Formatting and pedantic lint closure | `642c4fd15fcf9cbfe0838e118330bc69582b73ea` | format, compile, Clippy, tests, rustdoc, dependency policy, line coverage, and branch coverage all pass |

The RED commits are retained in branch history as evidence that behavior
contracts preceded implementation. They are not production defects or accepted
release heads. The wire-contract RED run `31062064135` failed because the new
public methods and fail-closed error variants did not yet exist, which is the
intended pre-implementation failure.

## Exact code-head GitHub evidence

Verified code head: `642c4fd15fcf9cbfe0838e118330bc69582b73ea`.

Rust Foundation CI run `31062751179` completed successfully:

- repository contracts and Python branch coverage: success;
- `cargo fmt --all -- --check`: success;
- `cargo check --workspace --all-targets --all-features`: success;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: success;
- `cargo nextest run --workspace --all-features`: **41 passed, 0 skipped**;
- doctests: success;
- warning-free rustdoc: success;
- `cargo deny check`: advisories, bans, licenses, and sources all passed;
- production line coverage: **100%**; and
- production branch coverage on pinned nightly: **100%**.

Documentation Quality run `31062751111` completed successfully on the same code
head. The later documentation-only commits update the ADR, architecture,
CHANGELOG, research register, and this validation report; their exact final-head
checks are required before the PR leaves Draft.

## Domain and wire test matrix

The permanent suite covers:

- the RFC 9562 Appendix A.6 `UUIDv7` vector and generated version-seven IDs;
- known `SHA-256` vectors and canonical lower-case hexadecimal round trips;
- malformed, non-ASCII, and invalid-position digest input;
- caller-owned byte-buffer mutation after artifact acceptance;
- empty and configured-size-limit rejection before immutable allocation;
- hostile multibyte UTF-8 text containing Latin, Korean, Japanese, combining,
  and supplementary-plane code points;
- exact agreement between half-open byte ranges and Unicode-scalar ranges;
- empty, reversed, byte-overflow, scalar-overflow, start-boundary,
  end-boundary, start-scalar, end-scalar, and cross-document failures;
- zero page numbers, nonfinite or nonpositive page dimensions, nonfinite,
  negative, or empty rectangle components, and horizontal or vertical overflow;
- stable errors that do not include source content or paths;
- identity-preserving JSON round trips for binary artifacts, UTF-8 documents,
  source spans, and nested page locations;
- strict unknown-field rejection at the top level and inside page locations;
- explicit unsupported-version rejection;
- digest recomputation and content-substitution rejection;
- malformed RFC 9562 identifiers, malformed digests, invalid byte values,
  malformed JSON, empty reconstructed content, and configured limit failures;
- source-span reconstruction against the wrong document;
- altered UTF-8 byte boundaries, scalar coordinates, and page rectangles; and
- generated enumeration of every valid nonempty code-point-aligned span in a
  multilingual/decomposed-Unicode corpus, with exact text and JSON round trips.

## Security and compatibility posture

The public domain types do not derive serde serialization. Private DTOs define
an explicit schema rather than exposing Rust field layout or internal caches.
`#[serde(deny_unknown_fields)]` prevents silently ignored extensions from
changing meaning. The current accepted `schema_version` is `1`; other versions
fail closed rather than being guessed or coerced.

Wire reconstruction preserves stable IDs but recomputes the digest from the
supplied content before acceptance. JSON strings and byte arrays are bounded by
the same domain limits as direct construction. Error messages identify the
failed invariant without embedding document text, raw bytes, filesystem paths,
or supplied identifiers.

`SHA-256` equality is treated only as content-verification evidence. It is not
used as proof of origin, authorization, authenticity, or chain of custody.
Those remain separate provenance and security layers.

## Review and merge posture

Task 2's approved code contract is implemented and verified on the recorded
code head. The PR remains stacked on Task 1 and cannot merge before the approved
planning baseline and Task 1 workspace foundation. The final documentation head
must receive fresh exact-head documentation, Rust, security, CodeRabbit,
OpenCode, and independent Noema evidence before the PR is marked Ready.

After the predecessor stack reaches `main`, this PR must be retargeted, rerun
against the protected default branch, reacquire an independent non-author
approval for the exact current head, and merge without protection bypass.
