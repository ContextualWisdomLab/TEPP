# Analysis Engine v1 — Evidence Doctoring

## Claim boundary

The stacked PR proves one deterministic temporal-evidence-readiness execution
slice. It does not prove production psychometric estimation, topic validity,
GPU performance, HTTP deployment, certification, or customer-wide scale.

## Decision-to-evidence mapping

| Contract | Implementation evidence | Customer action enabled |
|---|---|---|
| Historical cutoff safety | `available_time <= knowledge_cutoff` filter in `analysis_engine` | Re-run a historical snapshot without future-availability leakage |
| Multiple membership | `membership_count` is summed for every eligible unit | Inspect inclusive counts without atomistic single-group collapse |
| Terminal completion | `AnalysisRunTerminalResult` is built from the accepted request and receipt | Poll one stable terminal contract instead of treating acceptance as completion |
| Artifact integrity | Canonical JSON and SHA-256 digest | Verify that a downloaded result matches the published artifact identity |
| Fitted topic lineage | `topic_measurement` reference fit projected as `tepp.trsl_topic_lineage.v1` | Read predecessor/successor-aware connectable-post and lineage counts without treating association as causation |
| Privacy boundary | Artifact contains opaque IDs, counts, and times only | Keep identity mapping in the authorized source boundary |

## Scientific and standards basis

The implementation preserves TEPP's distinct event and availability clocks and
does not infer an event time from availability time. The API payload is explicit
JSON, and the artifact digest is an integrity check rather than proof of origin
or scientific truth. These interpretations follow the existing temporal,
interchange, and hashing register entries (Bray, 2017; International
Organization for Standardization, 2012; National Institute of Standards and
Technology, 2015).

## Verification record

The local preflight for this slice passed with Rust 1.97.1:

- `cargo fmt --all -- --check`;
- `cargo test -p analysis_engine` — 5 unit tests, 1 crate-contract test, 2
  end-to-end tests, and doctest collection;
- `cargo clippy -p analysis_engine --all-targets -- -D warnings`.
- `cargo test -p analysis_engine` — 8 unit tests, 1 crate-contract test, 3
  readiness integration tests, 2 topic-lineage integration tests, and doctest
  collection;
- `cargo clippy -p analysis_engine --all-targets -- -D warnings`.
- exact authored coverage — 148/148 lines and 74/74 branches.

The protected-hosted exact-head checks and qualifying independent reviews are
still pending. This document must not be used as implemented-main or release
evidence before that merge.

## APA 7th references

Bray, T. (Ed.). (2017). *The JavaScript Object Notation (JSON) data interchange
format* (RFC 8259). RFC Editor. https://doi.org/10.17487/RFC8259

International Organization for Standardization. (2012). *Language resource
management—Semantic annotation framework (SemAF)—Part 1: Time and events
(SemAF-Time, ISO-TimeML)* (ISO Standard No. 24617-1:2012).

National Institute of Standards and Technology. (2015). *Secure Hash Standard
(SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4
