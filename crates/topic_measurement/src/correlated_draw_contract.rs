//! Numerical covariance characterization with synthetic identified precisions.

use crate::JointCoordinatePrecision;
use temporal_core::EventTime;
use uuid::Uuid;

/// Recover a non-diagonal analytic covariance, including both correlation signs.
///
/// This is a synthetic precision-to-draw check, not fitted-parameter recovery.
/// Chebyshev plus a six-moment union bound sets family error at most 0.001
/// under the ideal independent Gaussian reference; this deterministic seed is
/// one numerical realization, not a PRNG quality or nominal-coverage proof.
#[test]
fn correlated_precision_draws_recover_analytic_covariance() {
    const COUNT: usize = 100_000;
    const COUNT_F64: f64 = 100_000.0;
    const FAMILY_ERROR: f64 = 0.001;
    const MOMENTS: f64 = 6.0;
    let document_ids = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
    let topic_ids = vec![Uuid::from_u128(11), Uuid::from_u128(12)];
    let event_times = vec![
        EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("time"),
        EventTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("time"),
    ];
    for off_diagonal in [-3.2_f64, 3.2_f64] {
        let precision = JointCoordinatePrecision {
            document_ids: document_ids.clone(),
            topic_ids: topic_ids.clone(),
            event_times: event_times.clone(),
            coordinate_means: vec![0.25, -0.5],
            values: vec![vec![4.0, off_diagonal], vec![off_diagonal, 4.0]],
        };
        let determinant = 16.0 - off_diagonal * off_diagonal;
        assert!(determinant.is_finite());
        assert!(determinant > 0.0);
        let expected_variance = 4.0 / determinant;
        let expected_covariance = -off_diagonal / determinant;
        let draws = precision.draw_joint_gaussian(19, COUNT).expect("draws");
        assert_eq!(draws.document_ids(), document_ids);
        assert_eq!(draws.topic_ids(), topic_ids);
        assert_eq!(draws.event_times(), event_times);
        assert_eq!(draws.draws().len(), COUNT);
        for row in draws.draws() {
            assert_eq!(row.len(), 2);
            assert!(row.iter().all(|value| value.is_finite()));
        }
        let mut moments = [0.0_f64; 3];
        for row in draws.draws() {
            let first = row[0] - 0.25;
            let second = row[1] + 0.5;
            moments[0] += first * first;
            moments[1] += second * second;
            moments[2] += first * second;
        }
        for value in &mut moments {
            *value /= COUNT_F64;
        }
        assert!(moments.iter().all(|value| value.is_finite()));
        let per_moment_error = FAMILY_ERROR / MOMENTS;
        let variance_bound = expected_variance * (2.0 / (COUNT_F64 * per_moment_error)).sqrt();
        let covariance_bound = ((expected_variance * expected_variance
            + expected_covariance * expected_covariance)
            / (COUNT_F64 * per_moment_error))
            .sqrt();
        println!(
            "CORRELATED off_diagonal={off_diagonal} determinant={determinant} expected_variance={expected_variance} expected_covariance={expected_covariance} observed={moments:?} variance_bound={variance_bound} covariance_bound={covariance_bound}"
        );
        assert!((moments[0] - expected_variance).abs() <= variance_bound);
        assert!((moments[1] - expected_variance).abs() <= variance_bound);
        assert!((moments[2] - expected_covariance).abs() <= covariance_bound);
        assert!(moments[2] * expected_covariance > 0.0);
    }
}
