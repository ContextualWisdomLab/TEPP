//! Validated compressed sparse matrices used by the reference estimator.

use crate::TopicMeasurementError;

/// Whether compressed values are grouped by row or by column.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SparseOrientation {
    /// Compressed sparse row storage.
    Row,
    /// Compressed sparse column storage.
    Column,
}

/// A finite numeric matrix in canonical CSR or CSC form.
#[derive(Clone, Debug, PartialEq)]
pub struct SparseMatrix {
    rows: usize,
    columns: usize,
    offsets: Vec<usize>,
    indices: Vec<usize>,
    values: Vec<f64>,
    orientation: SparseOrientation,
}

impl SparseMatrix {
    /// Construct and validate a compressed sparse row matrix.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidSparseMatrix`] for zero
    /// dimensions, malformed offsets, unsorted or repeated inner indices,
    /// out-of-range indices, or non-finite values.
    pub fn from_csr(
        rows: usize,
        columns: usize,
        offsets: Vec<usize>,
        indices: Vec<usize>,
        values: Vec<f64>,
    ) -> Result<Self, TopicMeasurementError> {
        Self::new(
            rows,
            columns,
            offsets,
            indices,
            values,
            SparseOrientation::Row,
        )
    }

    /// Construct and validate a compressed sparse column matrix.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidSparseMatrix`] under the same
    /// canonical-storage rules as [`Self::from_csr`].
    pub fn from_csc(
        rows: usize,
        columns: usize,
        offsets: Vec<usize>,
        indices: Vec<usize>,
        values: Vec<f64>,
    ) -> Result<Self, TopicMeasurementError> {
        Self::new(
            rows,
            columns,
            offsets,
            indices,
            values,
            SparseOrientation::Column,
        )
    }

    fn new(
        rows: usize,
        columns: usize,
        offsets: Vec<usize>,
        indices: Vec<usize>,
        values: Vec<f64>,
        orientation: SparseOrientation,
    ) -> Result<Self, TopicMeasurementError> {
        let outer = match orientation {
            SparseOrientation::Row => rows,
            SparseOrientation::Column => columns,
        };
        let inner = match orientation {
            SparseOrientation::Row => columns,
            SparseOrientation::Column => rows,
        };
        if rows == 0
            || columns == 0
            || outer
                .checked_add(1)
                .is_none_or(|offset_count| offsets.len() != offset_count)
            || offsets.first() != Some(&0)
            || offsets.last().copied() != Some(indices.len())
            || indices.len() != values.len()
            || values.iter().any(|value| !value.is_finite())
        {
            return Err(TopicMeasurementError::InvalidSparseMatrix);
        }
        for bounds in offsets.windows(2) {
            if bounds[0] > bounds[1] || bounds[1] > indices.len() {
                return Err(TopicMeasurementError::InvalidSparseMatrix);
            }
            let mut previous = None;
            for &index in &indices[bounds[0]..bounds[1]] {
                if index >= inner || previous.is_some_and(|value| index <= value) {
                    return Err(TopicMeasurementError::InvalidSparseMatrix);
                }
                previous = Some(index);
            }
        }
        Ok(Self {
            rows,
            columns,
            offsets,
            indices,
            values,
            orientation,
        })
    }

    /// Return the matrix row count.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }

    /// Return the matrix column count.
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.columns
    }

    /// Return the number of explicitly stored finite values.
    #[must_use]
    pub const fn nonzero_count(&self) -> usize {
        self.values.len()
    }

    /// Return the compressed orientation.
    #[must_use]
    pub const fn orientation(&self) -> SparseOrientation {
        self.orientation
    }

    pub(crate) fn row_entries(&self) -> Vec<Vec<(usize, f64)>> {
        let mut rows = vec![Vec::new(); self.rows];
        match self.orientation {
            SparseOrientation::Row => {
                for (row, bounds) in self.offsets.windows(2).enumerate() {
                    for index in bounds[0]..bounds[1] {
                        rows[row].push((self.indices[index], self.values[index]));
                    }
                }
            }
            SparseOrientation::Column => {
                for (column, bounds) in self.offsets.windows(2).enumerate() {
                    for index in bounds[0]..bounds[1] {
                        rows[self.indices[index]].push((column, self.values[index]));
                    }
                }
            }
        }
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::{SparseMatrix, SparseOrientation};
    use crate::TopicMeasurementError;

    #[test]
    fn csr_and_csc_produce_the_same_rows() {
        let csr = SparseMatrix::from_csr(2, 3, vec![0, 2, 3], vec![0, 2, 1], vec![1.0, 2.0, 3.0])
            .expect("csr");
        let csc =
            SparseMatrix::from_csc(2, 3, vec![0, 1, 2, 3], vec![0, 1, 0], vec![1.0, 3.0, 2.0])
                .expect("csc");
        assert_eq!(csr.rows(), 2);
        assert_eq!(csr.columns(), 3);
        assert_eq!(csr.nonzero_count(), 3);
        assert_eq!(csr.orientation(), SparseOrientation::Row);
        assert_eq!(csc.orientation(), SparseOrientation::Column);
        assert_eq!(csr.row_entries(), csc.row_entries());
    }

    #[test]
    fn malformed_sparse_storage_fails_closed() {
        assert_eq!(
            SparseMatrix::from_csr(usize::MAX, 2, Vec::new(), Vec::new(), Vec::new()),
            Err(TopicMeasurementError::InvalidSparseMatrix)
        );
        let error = Err(TopicMeasurementError::InvalidSparseMatrix);
        assert_eq!(SparseMatrix::from_csr(0, 1, vec![0], vec![], vec![]), error);
        assert_eq!(
            SparseMatrix::from_csr(1, 0, vec![0, 0], vec![], vec![]),
            error
        );
        assert_eq!(SparseMatrix::from_csr(1, 1, vec![0], vec![], vec![]), error);
        assert_eq!(
            SparseMatrix::from_csr(1, 1, vec![1, 1], vec![], vec![]),
            error
        );
        assert_eq!(
            SparseMatrix::from_csr(1, 1, vec![0, 2], vec![0], vec![1.0]),
            error
        );
        assert_eq!(
            SparseMatrix::from_csr(1, 1, vec![0, 1], vec![0], vec![]),
            error
        );
        assert_eq!(
            SparseMatrix::from_csr(1, 1, vec![0, 1], vec![1], vec![1.0]),
            error
        );
        assert_eq!(
            SparseMatrix::from_csr(1, 2, vec![0, 2], vec![1, 1], vec![1.0, 2.0]),
            error
        );
        assert_eq!(
            SparseMatrix::from_csr(1, 1, vec![0, 1], vec![0], vec![f64::NAN]),
            error
        );
        assert_eq!(
            SparseMatrix::from_csr(2, 1, vec![0, 2, 1], vec![0], vec![1.0]),
            error
        );
        assert_eq!(
            SparseMatrix::from_csr(3, 2, vec![0, 2, 1, 3], vec![0, 1, 1], vec![1.0; 3]),
            error
        );
    }
}
