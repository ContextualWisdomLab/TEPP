# Span-grounded semantic units

This note traces the first `semantic_core` slice to primary sources. It does not
claim concept alignment, measurement invariance, or a topic estimator.

## Sources

Phillips, A., & Davis, M. (Eds.). (2009). *Tags for identifying languages*
(RFC 5646). Internet Engineering Task Force. https://doi.org/10.17487/RFC5646

The Unicode Consortium. (2023). *The Unicode Standard, Version 15.1.0*.
https://www.unicode.org/versions/Unicode15.1.0/

Mimno, D., Wallach, H. M., Naradowsky, J., Smith, D. A., & McCallum, A. (2009).
Polylingual topic models. In *Proceedings of the 2009 Conference on Empirical
Methods in Natural Language Processing* (pp. 880–889). Association for
Computational Linguistics.

## Application

RFC 5646 licenses a primary language subtag with an optional region as
*metadata*. TEPP stores that label on `LanguageProfile` and refuses it as
`SemanticIdentity`. Unicode scalar/byte spans remain the evidence coordinates
from ADR 0008. Polylingual topic identity is a later shared-latent claim
(ADR 0004 / ADR 0012), not this slice.

Korean `측정` and English `Measurement` in matched report sentences are distinct
units on realistic native text. Unresolved language does not retokenize those
spans.
