# Corpus-split leakage-audit wire contract (doctoring)

## Scope

`tepp_api::CorpusSplitManifest` v1 is the buyer-visible audit of a relation-aware
split. It answers which documents were eligible at the knowledge cutoff, how
many were excluded as future evidence, which governed link kinds forced
co-partitioning, and which canonical digest binds the payload to the
`corpus_split_manifest` identity stored by migration `0003`.

The contract does not export source text, partition member lists, or HTTP
routes. Those remain follow-on work.

## Authority

Tashman, L. J. (2000). Out-of-sample tests of forecasting accuracy: An analysis and review. *International Journal of Forecasting, 16*(4), 437–450. https://doi.org/10.1016/S0169-2070(00)00065-0

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

Kish, L. (1965). *Survey sampling*. John Wiley & Sons.

Kaufman, S., Rosset, S., Perlich, C., & Stitelman, O. (2012). Leakage in data mining: Formulation, detection, and avoidance. *ACM Transactions on Knowledge Discovery from Data, 6*(4), 1–21. https://doi.org/10.1145/2382577.2382579

Tashman (2000) requires out-of-sample evaluation that respects temporal order.
Jensen and Snodgrass (1999) distinguish valid time from when evidence became
known; TEPP's availability/cutoff pair is the analysis-time analogue. Kaufman
et al. (2012) treat train/test contamination as a detectable leakage class;
revision, translation, copied-variant, and same-episode links are the TEPP
instances of that class. Kish (1965) remains the authority for
duplicate-aware weights computed after the split, not inside this wire DTO.

## Verification

- unit tests refuse empty identities, empty eligible snapshots, overlapping or
  incomplete partitions, relation leakage, unknown fields, unsupported
  versions, hostile link-kind tokens, and tampered digests;
- a synthetic four-document snapshot plus one late-available candidate proves
  `excluded_unavailable_at_cutoff_count = 1` while the late identity stays out
  of every partition digest;
- the committed example under `examples/corpus_split_manifest_v1.json` parses
  through the live contract.

Exact-head CI on the landing PR is the promotion evidence. Local or
predecessor-head runs do not make this implemented-main.
