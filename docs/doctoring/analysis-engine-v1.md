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
| Fitted topic lineage | `topic_measurement` reference fit projected as `tepp.trsl_topic_lineage.v2`, including candidate-fit evidence and separate source/model-input digests | Read predecessor/successor-aware connectable-post and lineage counts without treating association as causation |
| Posterior topic context | Fit-bound joint Laplace draws assembled as `tepp.topic_context_posterior.v2` with qualified Event Lineage and time-valid BU/PU/team/person membership | Send one complete uncertainty-bearing artifact to the governed influence estimator while retaining multiple predecessor branches |
| Privacy boundary | Artifact contains opaque IDs, counts, and times only | Keep identity mapping in the authorized source boundary |

## Scientific and standards basis

The implementation preserves TEPP's distinct event and availability clocks and
does not infer an event time from availability time. The API payload is explicit
JSON, and the artifact digest is an integrity check rather than proof of origin
or scientific truth. These interpretations follow the existing temporal,
interchange, and hashing register entries (Bray, 2017; International
Organization for Standardization, 2012; National Institute of Standards and
Technology, 2015).

## Research-source acquisition evidence

On 2026-08-28 the local Zotero library was queried by exact title and canonical
DOI before any write. Existing records for Dynamic Topic Models, Box--Muller,
Philox, and Dynamic Topic Models for Temporal Document Networks were retained;
no duplicate item was created. The missing Chang and Blei (2009) record was
added once from its canonical PMLR landing page. The Zotero Connector accepted
the official PMLR full-text URL, but the local library did not materialize a
child attachment. The local `/api/users/0/items` endpoint rejected attachment
writes, so the record remains source-linked rather than falsely documented as
having an archived file. The Project Euclid PDF response was not a PDF and was
therefore not attached. These are acquisition limits, not scientific evidence
gaps, and no arbitrary mirror or duplicate Zotero record was substituted.

## Verification record

The local preflight for this slice uses pinned Rust 1.98.0:

- `cargo fmt --all -- --check`;
- `cargo test -p analysis_engine --all-features`, including unit, integration,
  and doctest collections;
- `cargo clippy -p analysis_engine --all-targets --all-features -- -D warnings`;
- `RUSTDOCFLAGS='-D warnings' cargo doc -p analysis_engine --all-features
  --no-deps`;
- repository documentation validation.

The terminal workspace gates reported 100% production line coverage and 100%
production branch coverage (3,984/3,984 branches) under their documented
source filters. The new producer's four fail-closed binding arms are exercised
independently rather than counted through a short-circuited aggregate case.

The topic-context producer test fits the known synthetic two-topic corpus,
emits three plausible values for each of four documents, preserves one source
with two admitted successors as distinct branch relations, verifies every
required organizational dimension, and reparses canonical JSON without an
IEEE-754 value or digest drift. It does not claim GPU/MLX execution.

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
