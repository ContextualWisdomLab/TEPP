# Paragraph semantic units

## Scope

This note doctors the `evidence_core` meaning-search unit:

1. documents are split on blank lines into paragraph units;
2. each unit is a validated exact byte and Unicode-scalar span;
3. a multi-paragraph document cannot be collapsed into one bag-of-words unit.

This is the chunking contract for later embedding search. It does not run an embedding model and allocates no database migration.

## Authoritative sources

Unicode Consortium. (2024). *Unicode Standard Annex #29: Unicode text segmentation* (Revision 45). https://www.unicode.org/reports/tr29/

Reimers, N., & Gurevych, I. (2019). Sentence-BERT: Sentence embeddings using Siamese BERT-networks. In *Proceedings of the 2019 Conference on Empirical Methods in Natural Language Processing* (pp. 3982–3992). Association for Computational Linguistics. https://doi.org/10.18653/v1/D19-1410

## Application

UAX #29 treats paragraphs as higher-level grapheme/word/sentence containers; TEPP uses an explicit blank-line paragraph boundary as the first meaning unit so later sentence/DOM/sender units can be added without rewriting spans (Unicode Consortium, 2024). Reimers and Gurevych (2019) show that embedding search is useful only when the encoded unit is a coherent meaning span, not a whole-document bag. TEPP therefore refuses to treat two known paragraphs as one document vector (Reimers & Gurevych, 2019).

## Verification

- a two-paragraph Acme report recovers both exact source texts and spans;
- collapsing those units to one row returns `SemanticUnitBagRefused`;
- a single paragraph remains one unit;
- empty unit sets fail closed.
