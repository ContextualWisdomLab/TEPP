# Embedded image source units

## Scope

This note doctors the `evidence_core` contract for `data:image/<type>;base64,...` payloads that appear in document bodies:

1. each well-formed data URI becomes an `EmbeddedImageUnit` with an exact source span;
2. the declared media type is retained;
3. the original image location is preserved so later object/OCR search can attach to that span;
4. the base64 payload is not lexical inference text.

No OCR/object model is executed here. No database migration is allocated.

## Authoritative sources

Masinter, L. (1998). *The "data" URL scheme* (RFC 2397). RFC Editor. https://doi.org/10.17487/RFC2397

Antol, S., Agrawal, A., Lu, J., Mitchell, M., Batra, D., Zitnick, C. L., & Parikh, D. (2015). VQA: Visual question answering. In *Proceedings of the IEEE International Conference on Computer Vision* (pp. 2425–2433). https://doi.org/10.1109/ICCV.2015.279

## Application

RFC 2397 defines the `data:` URI and the `base64` encoding used in HTML and reports (Masinter, 1998). Visual question answering shows that image meaning is a separate modality from surrounding words (Antol et al., 2015). TEPP therefore keeps the original URI offset as a span and refuses to treat that payload as topic or lexical evidence (Masinter, 1998; Antol et al., 2015).

## Verification

- a PNG data URI between two paragraphs recovers media type `image/png` and the exact URI text;
- `refuse_base64_image_as_lexical_text` denies the full document and allows the surrounding sentence;
- a data URI declaring an implausible image media type (outside the accepted conservative set) fails closed with `ImplausibleImageMediaType`;
- documents without images return `EmptySourceSpan`;
- empty lexical input fails closed.
