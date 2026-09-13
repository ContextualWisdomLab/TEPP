# Citation-edge analysis-run composition

**Active slice:** ADR 0064 / `citation_edge_v1`
**Decision status:** Proposed
**Protected-main status:** not implemented-main

`citation_edge` already refuses to treat citation, translation, revision,
and retrospective-report edges as forward state transitions. This slice
binds those refusals to a digest-bound Analysis Run artifact without copying
the domain vocabulary.

The branch originally described itself as cutoff-safe while each
`CitationEdgeDocument` had only an identity and provenance kind. That allowed
future evidence to enter a historical census and made it impossible to prove
which snapshot supplied a row. The repaired input contract carries immutable
`snapshot_id` and `AvailableTime`; request and executor cutoffs bind by parsed
`KnowledgeCutoff::instant()` rather than RFC 3339 text.

Admission order is part of the scientific contract. Raw input above
`MAX_EVIDENCE_UNITS` fails before identity allocation. Cross-snapshot rows fail
closed. Same-snapshot rows unavailable at the requested cutoff are excluded
before duplicate and provenance-domain admission. Duplicate identities among
rows actually visible at the cutoff remain fail-closed. `document_count`
therefore means cutoff-admitted identities rather than raw input length.

Historical replay is explicit: prepending a future-unavailable row that reuses
a visible identity cannot change the artifact or terminal result. The branch
also verifies that two legal RFC 3339 spellings of one instant bind
identically, while a different instant remains a mismatch.

The protected-main provider currently returns
`CitationEdgeError::ProvenanceIsNotTransition` for every closed
`ProvenanceKind`, while its error enum is non-exhaustive. The adapter therefore
accepts only that exact refusal and routes unexpected success or any other
present/future provider error through a directly exercised fail-closed guard.
It does not silently assume the provider can never evolve.

The untrusted `from_json` boundary retains its 256 KiB cap. Canonical output no
longer carries a second, unreachable post-validation size branch: a focused
proof uses the maximum 256-byte run/snapshot identifiers, the maximum strict
RFC 3339 spelling permitted by `temporal_core` (four-digit date, up to nine
fractional digits and explicit offset), and `MAX_EVIDENCE_UNITS` census counts
to demonstrate the largest valid artifact remains below that input cap.

The artifact inference status is `provenance_is_not_a_state_transition`.
Terminal provider validation is separately `validated`.
`edge_kind_recovery_rate` stays library-side. This is not a
lineage-criterion fit, not corpus-background, not modality-source, not
prompt-source, not style-source, not copy-identity, not a simulation
method-effect census, not GPU, not MCMC, and not topic birth/split/merge.

Current repair lineage:

- `74d6ea13c211e3218bb4f61654f53a5b44badeb7` — RED for equivalent cutoff
  instants and provider/domain status separation;
- `3aac0968af752dee7b12ce542b8c73c3e1c7f036` — explicit snapshot and
  availability provenance, cutoff-before-identity admission, raw census bound,
  admitted count semantics and terminal `validated` state;
- `e05a38ecd52f60aee8dcfaa039862c7526d35570` — historical replay,
  cross-snapshot and raw-census integration contracts;
- `742b65bc0dc4d8c745cf3bc1f7b924abd1de7b80` — canonical
  `corpus_split::cutoff_eligible` promoted to the production dependency set;
- `74f48935674a1ab4d7f4c563bf9887a3ba627ae5` — ADR 0064 returned from
  premature `Accepted` branch authority to `Proposed`;
- `b780479fa9bcb9624e53dc79521899708163a05c` — provider-result drift moved
  behind a directly testable guard; maximal-valid output proof added before
  removing only the redundant egress limit branch;
- `f9e5a17f52e97efd0d2b564d31e8af4bdf8d93db` — wire-bound proof strengthened
  to maximum identifier lengths and the longest strict RFC 3339 timestamp
  form;
- `ccf8c4ed05ee975f6256c495fca9a9a88dcc1b15` — ADR currentized with those
  bounded/fail-closed decisions.

No predecessor workflow receipt is evidence for a later head. The eventual
surviving Analysis Run vehicle must reacquire exact-head line/branch coverage,
documentation/security/CodeQL gates and qualifying independent review after
conflict-resolving consolidation.
