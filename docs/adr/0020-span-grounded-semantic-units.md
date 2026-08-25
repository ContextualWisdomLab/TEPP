# ADR 0020 — Span-grounded semantic units

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-24
**Supersedes:** None. Complements ADR 0004 and ADR 0008. Does not replace ADR 0012 topic estimation or ADR 0005 psychometrics.
**Figma File ID:** N/A — this increment is a Rust domain crate with no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Issue #168 requires multilingual documents to enter a shared latent space while
preserving exact native-language evidence. Protected `main` already stores
immutable byte and Unicode-scalar spans (`evidence_core`, ADR 0008). It does not
yet bind those spans into semantic units, and language metadata must not become
a substitute identity or silently retokenize unresolved text.

## Decision

Add standalone crate `semantic_core` as the first ADR 0004 production slice:

- a semantic unit is identified by exact `SourceSpan` coordinates
  (document identity plus byte start/end);
- a language profile is optional metadata (`unresolved` or a primary ISO 639
  subtag with an optional region validated against the pinned IANA Language
  Subtag Registry snapshot dated 2026-08-08);
- unresolved language keeps the caller-supplied span and does not switch
  segmentation heuristic;
- `SemanticIdentity::from_language_tag` fails closed;
- Korean and English surfaces of comparable meaning remain distinct units.

This slice does not unitize raw text, align concepts, estimate topics, or claim
measurement invariance.

## Non-goals

- do not tokenize or morphologically analyze text;
- do not treat BCP 47 as a complete language-identification system;
- do not merge cross-language units into one concept;
- do not grant LLM-proposed units authority without exact spans.

## Alternatives considered

1. **Use the language tag as the unit key** — rejected because language is
   identifying metadata, not evidence identity, and mixed-language documents
   would collide or split incorrectly.
2. **Retokenize unresolved language with a default whitespace heuristic** —
   rejected because missing metadata must not silently change span bounds
   (ADR 0004).
3. **Span-grounded units with language as profile only** — accepted.

## Consequences

Operators can bind Korean and English exact spans without collapsing them.
Later concept-dictionary and invariance work (ADR 0004 / #84) can consume these
units without inheriting language-as-identity.

## Failure and recovery

Empty, malformed, private-use, or unknown-region language tags fail closed.
Offering a language tag as
identity fails closed. Recovery supplies a valid exact span and optional
profile; it does not rewrite historical artifacts.

## Security, privacy, and governance impact

Language tags and native lexical spans can be identifying. Purpose-bound access
under ADR 0009 applies. Documents remain untrusted input.

## Compatibility and migration

Standalone crate with no persistence schema. No database object names. Future
concept-alignment versions must not change span identity without a superseding
ADR.

## Verification

Integration tests bind a realistic Korean report sentence and an English
counterpart, prove distinct identities, prove unresolved vs `ko` keeps the
`측정` byte span, and prove language tags cannot become identity.

## Rollback and supersession

Remove the crate from the workspace if the slice is rejected. Supersede only
with a decision that keeps exact-span identity and explicit language-profile
status.

## Authority links

PRD multilingual measurement; ADR 0004; ADR 0008; issue #168; Phillips & Davis
(2009, RFC 5646).
