//! Exact text and page-layout coordinates for source evidence.

use crate::{DocumentRecord, EvidenceError, EvidenceId};

/// A validated rectangle on a one-based source page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageLocation {
    page_number: u32,
    page_width: f64,
    page_height: f64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl PageLocation {
    /// Validate a page-relative evidence rectangle.
    ///
    /// # Errors
    ///
    /// Returns a page or layout validation error when the page number,
    /// dimensions, offsets, or rectangle bounds are invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        page_number: u32,
        page_width: f64,
        page_height: f64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<Self, EvidenceError> {
        if page_number == 0 {
            return Err(EvidenceError::InvalidPageNumber);
        }
        if !positive_finite(page_width) || !positive_finite(page_height) {
            return Err(EvidenceError::InvalidPageGeometry);
        }
        if !nonnegative_finite(x)
            || !nonnegative_finite(y)
            || !positive_finite(width)
            || !positive_finite(height)
        {
            return Err(EvidenceError::InvalidLayoutBounds);
        }
        if x + width > page_width {
            return Err(EvidenceError::LayoutOutOfBounds);
        }
        if y + height > page_height {
            return Err(EvidenceError::LayoutOutOfBounds);
        }

        Ok(Self {
            page_number,
            page_width,
            page_height,
            x,
            y,
            width,
            height,
        })
    }

    /// Return the one-based page number.
    #[must_use]
    pub const fn page_number(&self) -> u32 {
        self.page_number
    }

    /// Return the page width.
    #[must_use]
    pub const fn page_width(&self) -> f64 {
        self.page_width
    }

    /// Return the page height.
    #[must_use]
    pub const fn page_height(&self) -> f64 {
        self.page_height
    }

    /// Return the rectangle's x-offset from the page origin.
    #[must_use]
    pub const fn x(&self) -> f64 {
        self.x
    }

    /// Return the rectangle's y-offset from the page origin.
    #[must_use]
    pub const fn y(&self) -> f64 {
        self.y
    }

    /// Return the rectangle width.
    #[must_use]
    pub const fn width(&self) -> f64 {
        self.width
    }

    /// Return the rectangle height.
    #[must_use]
    pub const fn height(&self) -> f64 {
        self.height
    }
}

/// An exact source-text span with optional validated page/layout coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceSpan {
    document_id: EvidenceId,
    byte_start: usize,
    byte_end: usize,
    scalar_start: usize,
    scalar_end: usize,
    page_location: Option<PageLocation>,
}

impl SourceSpan {
    /// Validate exact byte and Unicode-scalar coordinates against `document`.
    ///
    /// # Errors
    ///
    /// Returns a span validation error when ranges are empty, reversed,
    /// out-of-bounds, not aligned to UTF-8 code-point boundaries, or disagree
    /// about their Unicode-scalar positions.
    pub fn new(
        document: &DocumentRecord,
        byte_start: usize,
        byte_end: usize,
        scalar_start: usize,
        scalar_end: usize,
        page_location: Option<PageLocation>,
    ) -> Result<Self, EvidenceError> {
        if byte_start == byte_end {
            return Err(EvidenceError::EmptySourceSpan);
        }
        if scalar_start == scalar_end {
            return Err(EvidenceError::EmptySourceSpan);
        }
        if byte_start > byte_end {
            return Err(EvidenceError::InvalidSourceSpanOrder);
        }
        if scalar_start > scalar_end {
            return Err(EvidenceError::InvalidSourceSpanOrder);
        }
        if byte_end > document.byte_length() {
            return Err(EvidenceError::ByteRangeOutOfBounds);
        }
        if scalar_end > document.scalar_length() {
            return Err(EvidenceError::ByteRangeOutOfBounds);
        }
        if !document.text().is_char_boundary(byte_start) {
            return Err(EvidenceError::InvalidUtf8Boundary);
        }
        if !document.text().is_char_boundary(byte_end) {
            return Err(EvidenceError::InvalidUtf8Boundary);
        }

        let actual_scalar_start = document.text()[..byte_start].chars().count();
        let actual_scalar_end =
            actual_scalar_start + document.text()[byte_start..byte_end].chars().count();
        if actual_scalar_start != scalar_start {
            return Err(EvidenceError::CharacterRangeMismatch);
        }
        if actual_scalar_end != scalar_end {
            return Err(EvidenceError::CharacterRangeMismatch);
        }

        Ok(Self {
            document_id: document.id(),
            byte_start,
            byte_end,
            scalar_start,
            scalar_end,
            page_location,
        })
    }

    /// Return the owning document identifier.
    #[must_use]
    pub const fn document_id(&self) -> EvidenceId {
        self.document_id
    }

    /// Return the inclusive byte start.
    #[must_use]
    pub const fn byte_start(&self) -> usize {
        self.byte_start
    }

    /// Return the exclusive byte end.
    #[must_use]
    pub const fn byte_end(&self) -> usize {
        self.byte_end
    }

    /// Return the inclusive Unicode-scalar start.
    #[must_use]
    pub const fn scalar_start(&self) -> usize {
        self.scalar_start
    }

    /// Return the exclusive Unicode-scalar end.
    #[must_use]
    pub const fn scalar_end(&self) -> usize {
        self.scalar_end
    }

    /// Return optional page/layout coordinates.
    #[must_use]
    pub const fn page_location(&self) -> Option<PageLocation> {
        self.page_location
    }

    /// Return the exact UTF-8 text selected from `document`.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::SpanDocumentMismatch`] if `document` is not the
    /// record against which this span was validated.
    pub fn text<'document>(
        &self,
        document: &'document DocumentRecord,
    ) -> Result<&'document str, EvidenceError> {
        if document.id() != self.document_id {
            return Err(EvidenceError::SpanDocumentMismatch);
        }
        Ok(&document.text()[self.byte_start..self.byte_end])
    }
}

fn positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn nonnegative_finite(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}
