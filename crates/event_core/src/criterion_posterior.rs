//! Jeffreys-posterior plausible values for independent binary event evidence.
//!
//! The estimator uses the Bernoulli likelihood and Jeffreys' invariant
//! `Beta(1/2, 1/2)` prior. It does not threshold similarity scores or estimate
//! a criterion from the channels being validated.

use std::f64::consts::PI;

/// Fail-closed criterion posterior errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CriterionPosteriorError {
    /// No independent observations were supplied.
    EmptyObservations,
    /// Successes exceeded admitted trials.
    SuccessesExceedTrials,
    /// Fewer than two plausible values were requested.
    InsufficientDraws,
    /// The bounded inverse-CDF solver did not converge.
    NumericalFailure,
}

/// Independent binary criterion observations for one candidate relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndependentCriterionCounts {
    /// Number of independently observed positive criterion outcomes.
    pub successes: u32,
    /// Total independent criterion observations.
    pub trials: u32,
}

/// Identified Jeffreys posterior for one relation probability.
#[derive(Clone, Debug, PartialEq)]
pub struct CriterionPosterior {
    /// Posterior alpha parameter, `successes + 1/2`.
    pub alpha: f64,
    /// Posterior beta parameter, `failures + 1/2`.
    pub beta: f64,
    /// Posterior mean.
    pub mean: f64,
    /// Posterior variance.
    pub variance: f64,
    /// Deterministic midpoint posterior quantiles in ascending order.
    pub plausible_values: Vec<f64>,
}

/// Fit a Bernoulli criterion probability under Jeffreys' invariant prior.
///
/// Plausible values are deterministic midpoint posterior quantiles. This is a
/// quadrature representation of the identified posterior, not a random seed,
/// rank, threshold, or consumer-selected weight.
///
/// # Errors
///
/// Returns a fail-closed error for invalid counts, fewer than two draws, or a
/// bounded inverse-CDF numerical failure.
pub fn fit_independent_criterion_posterior(
    counts: IndependentCriterionCounts,
    draw_count: usize,
) -> Result<CriterionPosterior, CriterionPosteriorError> {
    if counts.trials == 0 {
        return Err(CriterionPosteriorError::EmptyObservations);
    }
    if counts.successes > counts.trials {
        return Err(CriterionPosteriorError::SuccessesExceedTrials);
    }
    if draw_count < 2 {
        return Err(CriterionPosteriorError::InsufficientDraws);
    }
    let draw_count_u32 =
        u32::try_from(draw_count).map_err(|_| CriterionPosteriorError::NumericalFailure)?;
    let alpha = f64::from(counts.successes) + 0.5;
    let beta = f64::from(counts.trials - counts.successes) + 0.5;
    let total = alpha + beta;
    let mean = alpha / total;
    let variance = alpha * beta / (total * total * (total + 1.0));
    let mut plausible_values = Vec::with_capacity(draw_count);
    for index in 0..draw_count {
        let index_u32 =
            u32::try_from(index).map_err(|_| CriterionPosteriorError::NumericalFailure)?;
        let probability = (f64::from(index_u32) + 0.5) / f64::from(draw_count_u32);
        plausible_values.push(beta_quantile(probability, alpha, beta)?);
    }
    Ok(CriterionPosterior {
        alpha,
        beta,
        mean,
        variance,
        plausible_values,
    })
}

fn beta_quantile(probability: f64, alpha: f64, beta: f64) -> Result<f64, CriterionPosteriorError> {
    let mut lower = 0.0_f64;
    let mut upper = 1.0_f64;
    for _ in 0..96 {
        let midpoint = lower.midpoint(upper);
        let cdf = regularized_beta(midpoint, alpha, beta)?;
        if cdf < probability {
            lower = midpoint;
        } else {
            upper = midpoint;
        }
    }
    let value = lower.midpoint(upper);
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(CriterionPosteriorError::NumericalFailure)
    }
}

fn regularized_beta(x: f64, alpha: f64, beta: f64) -> Result<f64, CriterionPosteriorError> {
    if x <= 0.0 {
        return Ok(0.0);
    }
    if x >= 1.0 {
        return Ok(1.0);
    }
    let log_scale = log_gamma(alpha + beta) - log_gamma(alpha) - log_gamma(beta)
        + alpha * x.ln()
        + beta * (-x).ln_1p();
    let scale = log_scale.exp();
    if !scale.is_finite() {
        return Err(CriterionPosteriorError::NumericalFailure);
    }
    let value = if x < (alpha + 1.0) / (alpha + beta + 2.0) {
        scale * beta_fraction(x, alpha, beta)? / alpha
    } else {
        1.0 - scale * beta_fraction(1.0 - x, beta, alpha)? / beta
    };
    if value.is_finite() {
        Ok(value.clamp(0.0, 1.0))
    } else {
        Err(CriterionPosteriorError::NumericalFailure)
    }
}

fn beta_fraction(x: f64, alpha: f64, beta: f64) -> Result<f64, CriterionPosteriorError> {
    const TINY: f64 = 1.0e-300;
    const EPSILON: f64 = 8.0 * f64::EPSILON;
    let qab = alpha + beta;
    let qap = alpha + 1.0;
    let qam = alpha - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < TINY {
        d = TINY;
    }
    d = 1.0 / d;
    let mut result = d;
    for iteration in 1..=512 {
        let m = f64::from(iteration);
        let twice = 2.0 * m;
        let mut coefficient = m * (beta - m) * x / ((qam + twice) * (alpha + twice));
        d = 1.0 + coefficient * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + coefficient / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        result *= d * c;
        coefficient = -(alpha + m) * (qab + m) * x / ((alpha + twice) * (qap + twice));
        d = 1.0 + coefficient * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + coefficient / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        let delta = d * c;
        result *= delta;
        if (delta - 1.0).abs() <= EPSILON {
            return Ok(result);
        }
    }
    Err(CriterionPosteriorError::NumericalFailure)
}

fn log_gamma(value: f64) -> f64 {
    const COEFFICIENTS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if value < 0.5 {
        return PI.ln() - (PI * value).sin().ln() - log_gamma(1.0 - value);
    }
    let shifted = value - 1.0;
    let mut series = COEFFICIENTS[0];
    for (index, coefficient) in COEFFICIENTS.iter().enumerate().skip(1) {
        let index_u32 = u32::try_from(index).expect("fixed coefficient index fits u32");
        series += coefficient / (shifted + f64::from(index_u32));
    }
    let scale = shifted + 7.5;
    0.5 * (2.0 * PI).ln() + (shifted + 0.5) * scale.ln() - scale + series.ln()
}

#[cfg(test)]
mod tests {
    use super::{
        CriterionPosteriorError, beta_fraction, beta_quantile, log_gamma, regularized_beta,
    };

    #[test]
    fn numerical_boundaries_and_invalid_parameters_fail_closed() {
        assert_eq!(regularized_beta(0.0, 1.0, 1.0), Ok(0.0));
        assert_eq!(regularized_beta(1.0, 1.0, 1.0), Ok(1.0));
        assert_eq!(
            regularized_beta(0.5, f64::NAN, 1.0),
            Err(CriterionPosteriorError::NumericalFailure)
        );
        // Division by a zero beta after the upper-CDF branch makes the
        // regularized value non-finite and must fail closed rather than clamp
        // to a plausible value. (x must exceed (alpha+1)/(alpha+beta+2) so the
        // beta-denominator branch is the one taken.)
        assert_eq!(
            regularized_beta(0.8, 2.0, 0.0),
            Err(CriterionPosteriorError::NumericalFailure)
        );
        assert_eq!(
            beta_fraction(f64::NAN, 1.0, 1.0),
            Err(CriterionPosteriorError::NumericalFailure)
        );
        assert_eq!(
            beta_quantile(0.5, f64::NAN, 1.0),
            Err(CriterionPosteriorError::NumericalFailure)
        );
        assert!(log_gamma(0.25).is_finite());
    }

    #[test]
    fn continued_fraction_tiny_branch_points_are_guarded_not_infinite() {
        // Exercise the TINY clamps inside `beta_fraction` with inputs whose
        // continued-fraction intermediates would otherwise become exact zero
        // or denormal in IEEE-754 f64: entry denominator (d = 1 - qab*x/qap),
        // the first-loop d, the second-loop d, and the second-loop c.
        assert!(beta_fraction(1.0, 1.0, 1.0).is_ok());
        assert!(beta_fraction(0.75, 1.0, 2.0).is_ok());
        assert!(beta_fraction(1.0, 2.0, 2.0).is_ok());
        assert!(beta_fraction(1.0, 2.0, 7.0).is_ok());
    }
}
