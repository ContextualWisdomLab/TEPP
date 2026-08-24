//! CPU `f64` ESEM loading recovery and event-time DSEM lag gates.

use crate::PsychometricFitError;

/// Coordinate system admitted into an ESEM/DSEM fit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FitCoordinateKind {
    /// Additive log-ratio coordinates.
    AdditiveLogRatio,
    /// Orthonormal isometric log-ratio coordinates.
    IsometricLogRatio,
    /// Logistic-normal latent coordinates.
    LogisticNormal,
    /// Raw simplex topic proportions. Forbidden as a fit input.
    RawProportion,
}

impl FitCoordinateKind {
    /// Stable wire name for the coordinate kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AdditiveLogRatio => "alr",
            Self::IsometricLogRatio => "ilr",
            Self::LogisticNormal => "logistic_normal",
            Self::RawProportion => "raw_proportion",
        }
    }

    /// Return whether the coordinate kind may enter a structural fit.
    #[must_use]
    pub const fn admits_structural_fit(self) -> bool {
        !matches!(self, Self::RawProportion)
    }
}

/// Higher-order construct class before reflective ESEM interpretation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FitConstructClass {
    /// Reflective indicators of a common latent factor.
    Reflective,
    /// Formative or composite indicators that define the construct.
    Formative,
    /// Interacting indicators that belong in a network model.
    Network,
    /// Insufficient evidence to classify the construct.
    Unresolved,
}

impl FitConstructClass {
    /// Stable wire name for the construct class.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reflective => "reflective",
            Self::Formative => "formative",
            Self::Network => "network",
            Self::Unresolved => "unresolved",
        }
    }

    /// Return whether reflective ESEM is admissible for this class.
    #[must_use]
    pub const fn admits_esem_fit(self) -> bool {
        matches!(self, Self::Reflective)
    }
}

/// Admit only log-ratio or logistic-normal coordinates into a fit.
///
/// # Errors
///
/// Returns [`PsychometricFitError::RawProportionForbidden`] for raw simplex
/// proportions.
pub fn admit_fit_coordinates(kind: FitCoordinateKind) -> Result<(), PsychometricFitError> {
    if kind.admits_structural_fit() {
        Ok(())
    } else {
        Err(PsychometricFitError::RawProportionForbidden)
    }
}

/// Interpret a classified construct as reflective after a fit.
///
/// A good global fit statistic is not authority to reinterpret a formative or
/// network structure as reflective (ADR 0005).
///
/// # Errors
///
/// Returns [`PsychometricFitError::FormativeReinterpretationForbidden`] for
/// formative or network classes and
/// [`PsychometricFitError::UnresolvedConstruct`] when the class is unresolved.
pub fn interpret_fit_as_reflective(
    classified: FitConstructClass,
    global_fit_acceptable: bool,
) -> Result<FitConstructClass, PsychometricFitError> {
    let _ = global_fit_acceptable;
    match classified {
        FitConstructClass::Reflective => Ok(FitConstructClass::Reflective),
        FitConstructClass::Unresolved => Err(PsychometricFitError::UnresolvedConstruct),
        FitConstructClass::Formative => {
            Err(PsychometricFitError::FormativeReinterpretationForbidden)
        }
        FitConstructClass::Network => Err(PsychometricFitError::FormativeReinterpretationForbidden),
    }
}

/// Recover an ESEM loading matrix by OLS of each indicator on every factor.
///
/// Each inner slice is one variable's observation vector. At most two factors
/// are inverted on this CPU `f64` reference path so the Gram matrix stays
/// explicit. Cross-loadings are retained.
///
/// # Errors
///
/// Returns coordinate, payload, or singularity errors from the OLS path.
pub fn recover_esem_loadings(
    factor_scores: &[Vec<f64>],
    indicators: &[Vec<f64>],
    kind: FitCoordinateKind,
) -> Result<Vec<Vec<f64>>, PsychometricFitError> {
    admit_fit_coordinates(kind)?;
    if factor_scores.len() > 2 {
        return Err(PsychometricFitError::InvalidNumericInput);
    }
    let observation_count = factor_scores.first().map_or(0, Vec::len);
    let mut centered_factors = Vec::new();
    for values in factor_scores {
        if observation_count < 2 || values.len() != observation_count {
            return Err(PsychometricFitError::InvalidNumericInput);
        }
        centered_factors.push(center(values)?);
    }
    if centered_factors.is_empty() {
        return Err(PsychometricFitError::InvalidNumericInput);
    }
    let mut loadings = Vec::new();
    for values in indicators {
        if values.len() != observation_count {
            return Err(PsychometricFitError::InvalidNumericInput);
        }
        let centered_indicator = center(values)?;
        loadings.push(ordinary_least_squares_loadings(
            &centered_factors,
            &centered_indicator,
        )?);
    }
    if loadings.is_empty() {
        return Err(PsychometricFitError::InvalidNumericInput);
    }
    Ok(loadings)
}

/// Root-mean-square error between known-truth and recovered loading matrices.
///
/// # Errors
///
/// Returns [`PsychometricFitError::InvalidNumericInput`] when either matrix is
/// empty or the shapes differ.
pub fn loading_recovery_rmse(
    truth: &[Vec<f64>],
    recovered: &[Vec<f64>],
) -> Result<f64, PsychometricFitError> {
    if truth.is_empty() || truth.len() != recovered.len() {
        return Err(PsychometricFitError::InvalidNumericInput);
    }
    let mut sum_sq = 0.0_f64;
    let mut count = 0_u32;
    for (truth_row, recovered_row) in truth.iter().zip(recovered) {
        if truth_row.is_empty() || truth_row.len() != recovered_row.len() {
            return Err(PsychometricFitError::InvalidNumericInput);
        }
        for (truth_value, recovered_value) in truth_row.iter().zip(recovered_row) {
            let residual = truth_value - recovered_value;
            sum_sq += residual * residual;
            count += 1;
        }
    }
    Ok((sum_sq / f64::from(count)).sqrt())
}

/// Recover a DSEM lagged path only when the predictor precedes the outcome.
///
/// # Errors
///
/// Returns [`PsychometricFitError::ReverseEventTimePath`] when
/// `outcome_event_time` is not strictly later than `predictor_event_time`,
/// and OLS payload or singularity errors otherwise.
pub fn recover_dsem_lagged_path(
    predictor_event_time: i64,
    outcome_event_time: i64,
    predictor: &[f64],
    outcome: &[f64],
) -> Result<f64, PsychometricFitError> {
    if outcome_event_time <= predictor_event_time {
        return Err(PsychometricFitError::ReverseEventTimePath);
    }
    let loadings = recover_esem_loadings(
        &[predictor.to_vec()],
        &[outcome.to_vec()],
        FitCoordinateKind::LogisticNormal,
    )?;
    Ok(loadings[0][0])
}

fn center(values: &[f64]) -> Result<Vec<f64>, PsychometricFitError> {
    require_finite_slice(values)?;
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    require_finite(mean)?;
    let mut centered = Vec::with_capacity(values.len());
    for value in values {
        centered.push(require_finite(value - mean)?);
    }
    Ok(centered)
}

fn ordinary_least_squares_loadings(
    centered_factors: &[Vec<f64>],
    centered_indicator: &[f64],
) -> Result<Vec<f64>, PsychometricFitError> {
    let gram = gram_matrix(centered_factors)?;
    let inverse = invert_gram(&gram)?;
    let mut cross = vec![0.0_f64; centered_factors.len()];
    for (factor_index, factor) in centered_factors.iter().enumerate() {
        let mut total = 0.0_f64;
        for (score, outcome) in factor.iter().zip(centered_indicator) {
            total += score * outcome;
        }
        cross[factor_index] = require_finite(total)?;
    }
    let mut loadings = vec![0.0_f64; centered_factors.len()];
    for (row_index, inverse_row) in inverse.iter().enumerate() {
        let mut total = 0.0_f64;
        for (weight, value) in inverse_row.iter().zip(&cross) {
            total += weight * value;
        }
        loadings[row_index] = require_finite(total)?;
    }
    Ok(loadings)
}

fn gram_matrix(centered_factors: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, PsychometricFitError> {
    let rank = centered_factors.len();
    let mut gram = vec![vec![0.0_f64; rank]; rank];
    for row in 0..rank {
        for column in 0..rank {
            let mut total = 0.0_f64;
            for (left, right) in centered_factors[row].iter().zip(&centered_factors[column]) {
                total += left * right;
            }
            gram[row][column] = require_finite(total)?;
        }
    }
    Ok(gram)
}

fn invert_gram(gram: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, PsychometricFitError> {
    match gram.len() {
        1 => {
            let value = gram[0][0];
            if value <= 0.0 {
                return Err(PsychometricFitError::SingularDesign);
            }
            Ok(vec![vec![require_finite(1.0 / value)?]])
        }
        2 => {
            let a = gram[0][0];
            let b = gram[0][1];
            let c = gram[1][0];
            let d = gram[1][1];
            let determinant = require_finite(a * d - b * c)?;
            if determinant.abs() <= 0.0 {
                return Err(PsychometricFitError::SingularDesign);
            }
            Ok(vec![
                vec![
                    require_finite(d / determinant)?,
                    require_finite(-b / determinant)?,
                ],
                vec![
                    require_finite(-c / determinant)?,
                    require_finite(a / determinant)?,
                ],
            ])
        }
        _ => Err(PsychometricFitError::InvalidNumericInput),
    }
}

fn require_finite_slice(values: &[f64]) -> Result<(), PsychometricFitError> {
    for value in values {
        require_finite(*value)?;
    }
    Ok(())
}

fn require_finite(value: f64) -> Result<f64, PsychometricFitError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PsychometricFitError::InvalidNumericInput)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FitConstructClass, FitCoordinateKind, admit_fit_coordinates, invert_gram,
        loading_recovery_rmse, recover_dsem_lagged_path, recover_esem_loadings, require_finite,
    };
    use crate::PsychometricFitError;

    #[test]
    fn local_branches_cover_fit_gates_and_inversions() {
        cover_coordinate_and_inversion_gates();
        cover_recovery_payload_gates();
        cover_rmse_and_indicator_gates();
    }

    fn cover_coordinate_and_inversion_gates() {
        assert!(FitCoordinateKind::AdditiveLogRatio.admits_structural_fit());
        assert!(!FitCoordinateKind::RawProportion.admits_structural_fit());
        admit_fit_coordinates(FitCoordinateKind::IsometricLogRatio).expect("ilr");
        assert_eq!(
            admit_fit_coordinates(FitCoordinateKind::RawProportion),
            Err(PsychometricFitError::RawProportionForbidden)
        );
        assert_eq!(
            require_finite(f64::INFINITY),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        assert_eq!(
            invert_gram(&[vec![0.0]]),
            Err(PsychometricFitError::SingularDesign)
        );
        assert_eq!(
            invert_gram(&[vec![-1.0]]),
            Err(PsychometricFitError::SingularDesign)
        );
        assert_eq!(
            invert_gram(&[vec![1e-320]]),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        let inverted = invert_gram(&[vec![2.0]]).expect("1x1");
        assert!((inverted[0][0] - 0.5).abs() < f64::EPSILON);
        assert_eq!(
            invert_gram(&[vec![1.0, 0.0], vec![0.0, 0.0]]),
            Err(PsychometricFitError::SingularDesign)
        );
        assert_eq!(
            invert_gram(&[vec![f64::MAX, f64::MAX], vec![f64::MAX, f64::MAX]]),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        let negative = invert_gram(&[vec![1.0, 2.0], vec![2.0, 1.0]]).expect("neg det");
        assert!((negative[0][0] + 1.0 / 3.0).abs() < 1e-12);
        let two = invert_gram(&[vec![1.0, 0.0], vec![0.0, 2.0]]).expect("2x2");
        assert!((two[0][0] - 1.0).abs() < f64::EPSILON);
        assert!((two[1][1] - 0.5).abs() < f64::EPSILON);
        assert_eq!(
            invert_gram(&[vec![1.0], vec![2.0], vec![3.0]]),
            Err(PsychometricFitError::InvalidNumericInput)
        );
    }

    fn cover_recovery_payload_gates() {
        assert_eq!(
            recover_esem_loadings(
                &[vec![1.0, 2.0], vec![3.0]],
                &[vec![1.0, 2.0]],
                FitCoordinateKind::AdditiveLogRatio
            ),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        assert_eq!(
            recover_esem_loadings(
                &[vec![1.0, f64::INFINITY]],
                &[vec![1.0, 2.0]],
                FitCoordinateKind::AdditiveLogRatio
            ),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        assert_eq!(
            recover_esem_loadings(
                &[vec![f64::MAX, f64::MAX]],
                &[vec![1.0, 2.0]],
                FitCoordinateKind::AdditiveLogRatio
            ),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        let collinear = recover_esem_loadings(
            &[vec![1.0, 2.0, 3.0], vec![2.0, 4.0, 6.0]],
            &[vec![1.0, 2.0, 3.0]],
            FitCoordinateKind::AdditiveLogRatio,
        );
        assert_eq!(collinear, Err(PsychometricFitError::SingularDesign));
        assert_eq!(
            recover_dsem_lagged_path(5, 4, &[0.0, 1.0], &[0.0, 1.0]),
            Err(PsychometricFitError::ReverseEventTimePath)
        );
        let forward = recover_dsem_lagged_path(1, 2, &[0.0, 1.0], &[0.0, 2.0]).expect("forward");
        assert!((forward - 2.0).abs() < 1e-12);
        assert_eq!(
            loading_recovery_rmse(&[vec![]], &[vec![]]),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        let rmse = loading_recovery_rmse(&[vec![1.0]], &[vec![1.0]]).expect("zero");
        assert!(rmse.abs() < f64::EPSILON);
        assert!(FitConstructClass::Reflective.admits_esem_fit());
        assert_eq!(FitConstructClass::Network.as_str(), "network");
        assert_eq!(
            super::interpret_fit_as_reflective(FitConstructClass::Reflective, false)
                .expect("fit unused"),
            FitConstructClass::Reflective
        );
        assert_eq!(
            recover_esem_loadings(&[vec![]], &[vec![]], FitCoordinateKind::AdditiveLogRatio),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        assert_eq!(
            recover_esem_loadings(
                &[] as &[Vec<f64>],
                &[vec![1.0, 2.0]],
                FitCoordinateKind::AdditiveLogRatio
            ),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        assert_eq!(
            recover_esem_loadings(
                &[vec![1.0, 2.0]],
                &[] as &[Vec<f64>],
                FitCoordinateKind::AdditiveLogRatio
            ),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        let three = [vec![0.0, 1.0], vec![1.0, 0.0], vec![0.5, 0.5]];
        assert_eq!(
            recover_esem_loadings(
                &three,
                &[vec![1.0, 2.0]],
                FitCoordinateKind::AdditiveLogRatio
            ),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        assert_eq!(
            admit_fit_coordinates(FitCoordinateKind::RawProportion),
            Err(PsychometricFitError::RawProportionForbidden)
        );
    }

    fn cover_rmse_and_indicator_gates() {
        assert_eq!(
            loading_recovery_rmse(&[], &[]),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        assert_eq!(
            loading_recovery_rmse(&[vec![1.0]], &[vec![1.0, 2.0]]),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        assert_eq!(
            loading_recovery_rmse(&[vec![1.0]], &[]),
            Err(PsychometricFitError::InvalidNumericInput)
        );
        assert_eq!(
            recover_esem_loadings(
                &[vec![1.0, 2.0]],
                &[vec![1.0]],
                FitCoordinateKind::AdditiveLogRatio
            ),
            Err(PsychometricFitError::InvalidNumericInput)
        );
    }
}
