# Paragraph semantic units

## Scope

This note doctors the `evidence_core` meaning-search unit:

1. documents are split on explicit blank lines into paragraph units;
2. each unit is a validated exact byte and Unicode-scalar span;
3. a multi-paragraph document cannot be collapsed into one document-level unit by supplying a weaker caller-owned count.

This is the first chunking contract for later embedding search. It does not run an embedding model, claim that paragraph boundaries are universally optimal, or allocate a database migration.

## Authoritative sources

Unicode Consortium. (2024). *Unicode Standard Annex #29: Unicode text segmentation* (Revision 45). https://www.unicode.org/reports/tr29/

Reimers, N., & Gurevych, I. (2019). Sentence-BERT: Sentence embeddings using Siamese BERT-networks. In *Proceedings of the 2019 Conference on Empirical Methods in Natural Language Processing* (pp. 3982–3992). Association for Computational Linguistics. https://doi.org/10.18653/v1/D19-1410

## Application

UAX #29 specifies lower-level Unicode text-boundary algorithms and makes clear that higher-level protocols may tailor segmentation. TEPP therefore treats an explicit blank line as a repository-owned paragraph convention while retaining exact source coordinates; later sentence, DOM, sender/recipient, and language-profile units can be added without rewriting the evidence identity (Unicode Consortium, 2024).

Sentence-BERT demonstrates retrieval-oriented embeddings for sentence and short-text units, but it does not establish that blank-line paragraphs are universally optimal or that every whole document is an invalid embedding input (Reimers & Gurevych, 2019). TEPP's narrower product decision is to preserve known paragraph multiplicity at this boundary so later measurement can compare or replace chunking policies without losing the original evidence spans.

## Verification

- a two-paragraph Acme report recovers both exact source texts and spans;
- collapsing those units to one row returns `SemanticUnitBagRefused`;
- a caller cannot weaken the guard with a smaller paragraph count;
- unrelated spans with the correct count return `InvalidWirePayload`;
- a single canonical paragraph remains one unit;
- empty unit sets fail closed.
