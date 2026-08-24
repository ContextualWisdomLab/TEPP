# Unicode canonical identity for corpus splits

## Scope

This note doctors the `corpus_split` Unicode identity contract:

1. a document body is identified by its Unicode Normalization Form C (NFC) sequence;
2. NFC and NFD forms of the same abstract characters are equivalent;
3. canonically equivalent bodies must be co-partitioned and cannot occupy independent train/test folds.

This is a leakage-safe split contract. It does not tokenize, stem, or estimate topics, and it allocates no database migration.

## Authoritative sources

Davis, M., Iancu, L., & Whistler, K. (Eds.). (2024). *Unicode Standard Annex #15: Unicode normalization forms*. Unicode Consortium. https://www.unicode.org/reports/tr15/

Yergeau, F. (2003). *UTF-8, a transformation format of ISO 10646* (RFC 3629). RFC Editor. https://doi.org/10.17487/RFC3629

## Application

UAX #15 states that canonically equivalent strings represent the same abstract character sequence and that NFC is the preferred interchange form (Davis et al., 2024). TEPP therefore compares document bodies after NFC rather than treating composed `é` and decomposed `e` + combining acute as distinct evidence. Hangul syllables and their jamo decompositions are included because they are the same canonical class (Davis et al., 2024). RFC 3629 remains the UTF-8 interchange constraint already used by `evidence_core` spans (Yergeau, 2003).

Independent split membership of NFC/NFD pairs would invent independence between identical evidence and leak identity across knowledge-cutoff partitions.

## Verification

- Latin `café` NFC and NFD recover as equivalent; `cafe` does not;
- Hangul syllable `각` NFC and its jamo NFD recover as equivalent;
- an NFC/NFD pair emits one `CanonicalEquivalent` link and is rejected when assigned to different partitions;
- empty bodies and duplicate document identities fail closed.
