//! Enforce coherence between Monte Carlo replication spread and reported standard error.
//!
//! `MonteCarloSummary` is durable Validation Evidence, so its uncertainty fields cannot be
//! independently user-selected. For multiple replications, the reported standard error must remain
//! coherent with the stored standard deviation and replication count, allowing only the adjacent
//! binary64 rounding neighborhood of the canonical `sd / sqrt(n)` result. Positive spread cannot
//! carry zero uncertainty, zero spread cannot carry positive uncertainty, and a singleton cannot
//! claim an empirical spread. Invalid evidence must fail both domain validation and serde ingress /
//! egress instead of being persisted as a scientifically contradictory receipt.

use validation_core::{MonteCarloSummary, ValidationError};

fn summary(replication_count: usize, standard_deviation: f64, standard_error: f64) -> MonteCarloSummary {
    MonteCarloSummary {
        replication_count,
        mean: 0.5,
        standard_deviation,
        standard_error,
        percentile_lower: -2.0,
        percentile_upper: 3.0,
    }
}

#[test]
fn monte_carlo_summary_rejects_impossible_standard_error_evidence() {
    let understated_for_n = summary(4, 0.5, 0.2);
    assert_eq!(
        understated_for_n.validate(),
        Err(ValidationError::InvalidInput)
    );

    let canonical_standard_error = 0.5 / 4.0_f64.sqrt();
    let adjacent_standard_error = f64::from_bits(canonical_standard_error.to_bits() + 1);
    assert!(summary(4, 0.5, adjacent_standard_error).validate().is_ok());

    let equal_to_sd_with_multiple_replications = summary(4, 0.5, 0.5);
    assert_eq!(
        equal_to_sd_with_multiple_replications.validate(),
        Err(ValidationError::InvalidInput)
    );

    let larger_than_sd = summary(4, 0.5, 1.0);
    assert_eq!(larger_than_sd.validate(), Err(ValidationError::InvalidInput));
    assert!(serde_json::to_string(&larger_than_sd).is_err());

    let false_zero_uncertainty = summary(4, 0.5, 0.0);
    assert_eq!(
        false_zero_uncertainty.validate(),
        Err(ValidationError::InvalidInput)
    );

    let zero_spread_with_positive_uncertainty = summary(4, 0.0, 0.1);
    assert_eq!(
        zero_spread_with_positive_uncertainty.validate(),
        Err(ValidationError::InvalidInput)
    );

    let impossible_singleton_spread = summary(1, 0.5, 0.5);
    assert_eq!(
        impossible_singleton_spread.validate(),
        Err(ValidationError::InvalidInput)
    );

    let payload = r#"{"replication_count":4,"mean":0.5,"standard_deviation":0.5,"standard_error":0.2,"percentile_lower":-2.0,"percentile_upper":3.0}"#;
    assert!(serde_json::from_str::<MonteCarloSummary>(payload).is_err());
}
